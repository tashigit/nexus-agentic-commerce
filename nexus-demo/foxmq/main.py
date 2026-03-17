import asyncio
import logging
import random
import time

from nexus_sdk.engine import FoxMQEngine
from nexus_sdk.types import (
    Bid,
    Role,
    EventBid,
    EventDeal,
    EventSettlement,
    EventMetrics,
    EventMeshUpdate,
    EventAgentFailed,
    EventAgentHealed,
)

from agents import spawn_agents, generate_slot
from auction import ClearingHouse, resolve_auction, compute_mesh
from ws_server import WSServer

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("main")

AGENT_COUNT = 50
AUCTION_INTERVAL_SEC = 0.05   # 50ms
METRICS_INTERVAL_SEC = 0.5    # 500ms
MESH_INTERVAL_SEC = 1.0       # 1s

class AuctionState:
    def __init__(self, agents):
        self.agents = agents
        self.clearing = ClearingHouse(200.0)
        self.total_deals = 0
        self.total_bids = 0
        self.latency_sum = 0.0
        self.surge_multiplier = 1.0
        self.recent_deals = []

        self.coordinator = FoxMQEngine(agent_id="coordinator_001")

async def run_auction_round(state: AuctionState, ws: WSServer):
    publishers = [i for i, a in enumerate(state.agents) if a.role == Role.publisher and a.active]
    if not publishers:
        # print("No active publishers!")
        return

    auctions_per_round = min(int(max(1, state.surge_multiplier)), len(publishers))

    for _ in range(auctions_per_round):
        pub_idx = random.choice(publishers)
        agent_pub = state.agents[pub_idx]
        slot = generate_slot(agent_pub)

        trust_threshold = agent_pub.mandate.trust_threshold
        pending_bids = []

        for agent in state.agents:
            if agent.role == Role.advertiser and agent.active:
                if random.random() < 0.4:
                    amount = random.uniform(slot.min_price, max(agent.mandate.max_bid, slot.min_price + 1.0))
                    if agent.balance >= amount:
                        now_ms = int(time.time() * 1000)
                        bid_event = EventBid(
                            from_=agent.id,
                            slot=slot,
                            amount=amount,
                            timestamp=now_ms
                        )
                        await ws.broadcast_event(bid_event.model_dump_json(by_alias=True))

                        pending_bids.append(Bid(
                            id=f"bid-{agent.id[:6]}-{now_ms}",
                            bidder=agent.id,
                            slot_id=slot.id,
                            amount=amount,
                            timestamp_ms=now_ms
                        ))
                        state.total_bids += 1

        if not pending_bids:
            continue

        # Route bids through FoxMQ broker and collect them in broker-delivered order
        ordered = await state.coordinator.run_batch_cycle(pending_bids, slot.id)
        ordered_bids = [o.bid for o in ordered]

        if not ordered_bids:
            continue

        trust_map = {a.id: a.trust_score for a in state.agents}
        result = resolve_auction(ordered_bids, slot, trust_map, trust_threshold)

        if result:
            deal, latency_ms = result
            
            # Update balances
            buyer = next((a for a in state.agents if a.id == deal.buyer), None)
            seller = next((a for a in state.agents if a.id == deal.seller), None)
            
            if buyer:
                buyer.balance -= deal.amount
            if seller:
                seller.balance += deal.amount
                
            state.total_deals += 1
            state.latency_sum += latency_ms
            state.recent_deals.append(deal)
            
            event_deal = EventDeal(
                buyer=deal.buyer,
                seller=deal.seller,
                amount=deal.amount,
                latency_ms=latency_ms,
                slot=slot
            )
            await ws.broadcast_event(event_deal.model_dump_json(by_alias=True))
            
            settlement_res = state.clearing.record_deal(deal)
            if settlement_res:
                event_set = EventSettlement(
                    between=settlement_res["between"],
                    net_amount=settlement_res["net_amount"],
                    tx_count=settlement_res["tx_count"]
                )
                await ws.broadcast_event(event_set.model_dump_json(by_alias=True))

async def loop_auction(state: AuctionState, ws: WSServer):
    while True:
        await run_auction_round(state, ws)
        # Decay surge
        if state.surge_multiplier > 1.0:
            state.surge_multiplier *= 0.995
            if state.surge_multiplier < 1.1:
                state.surge_multiplier = 1.0
                
        await asyncio.sleep(AUCTION_INTERVAL_SEC)

async def loop_metrics(state: AuctionState, ws: WSServer):
    last_deal_count = 0
    while True:
        await asyncio.sleep(METRICS_INTERVAL_SEC)
        
        deals_in_window = state.total_deals - last_deal_count
        tps = deals_in_window / METRICS_INTERVAL_SEC
        avg_latency = state.latency_sum / state.total_deals if state.total_deals > 0 else 0.0
        active_agents = sum(1 for a in state.agents if a.active)
        
        event = EventMetrics(
            tps=tps,
            avg_latency_ms=avg_latency,
            total_deals=state.total_deals,
            total_settled=state.clearing.total_settled,
            active_agents=active_agents
        )
        await ws.broadcast_event(event.model_dump_json(by_alias=True))
        last_deal_count = state.total_deals

async def loop_mesh(state: AuctionState, ws: WSServer):
    while True:
        await asyncio.sleep(MESH_INTERVAL_SEC)
        nodes, edges = compute_mesh(state.agents, state.recent_deals)
        event = EventMeshUpdate(nodes=nodes, edges=edges)
        await ws.broadcast_event(event.model_dump_json(by_alias=True))
        
        if len(state.recent_deals) > 200:
            state.recent_deals = state.recent_deals[100:]

async def main():
    print("=== Nexus-HFC FoxMQ Demo ===")
    print(f"Spawning {AGENT_COUNT} agents...")

    agent_list = spawn_agents(AGENT_COUNT)
    state = AuctionState(agent_list)
    loop = asyncio.get_running_loop()
    state.coordinator.connect(loop=loop)

    # Wait for FoxMQ connection to establish
    await asyncio.sleep(1)

    publishers = sum(1 for a in agent_list if a.role == Role.publisher)
    advertisers = sum(1 for a in agent_list if a.role == Role.advertiser)
    print(f"  {publishers} publishers, {advertisers} advertisers")

    # Start WebSocket Server
    ws = WSServer(host="0.0.0.0", port=3001, initial_agents=agent_list)

    def handle_ws_command(cmd_str: str):
        if cmd_str == "surge":
            print("[cmd] Market Surge!")
            state.surge_multiplier = 5.0
        elif cmd_str == "attack":
            print("[cmd] Network Attack!")
            for agent in state.agents:
                if random.random() < 0.3 and agent.active:
                    agent.active = False
                    agent.trust_score = max(0.0, agent.trust_score - 0.3)
                    event = EventAgentFailed(agent=agent.id)
                    asyncio.create_task(ws.broadcast_event(event.model_dump_json(by_alias=True)))
        elif cmd_str == "heal":
            print("[cmd] Healing network...")
            for agent in state.agents:
                if not agent.active:
                    agent.active = True
                    agent.trust_score = min(1.0, agent.trust_score + 0.2)
                    event = EventAgentHealed(agent=agent.id)
                    asyncio.create_task(ws.broadcast_event(event.model_dump_json(by_alias=True)))
            state.surge_multiplier = 1.0

    ws.on_command = handle_ws_command

    print("Starting simulation loops...\n")
    
    await asyncio.gather(
        ws.start(),
        loop_auction(state, ws),
        loop_metrics(state, ws),
        loop_mesh(state, ws)
    )

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Exiting...")

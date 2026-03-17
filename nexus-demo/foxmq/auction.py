import math
import time
from typing import List, Dict, Tuple, Optional

from nexus_sdk.types import AgentInfo, AdSlot, Deal, Bid, MeshNode, MeshEdge

class ClearingHouse:
    def __init__(self, threshold: float):
        self.ledger: Dict[Tuple[str, str], float] = {}
        self.tx_counts: Dict[Tuple[str, str], int] = {}
        self.threshold = threshold
        self.total_settled = 0

    def _normalize_pair(self, a: str, b: str) -> Tuple[str, str]:
        return (a, b) if a <= b else (b, a)

    def record_deal(self, deal: Deal) -> Optional[Dict]:
        pair = self._normalize_pair(deal.buyer, deal.seller)
        is_forward = (pair[0] == deal.buyer)
        
        entry = self.ledger.get(pair, 0.0)
        if is_forward:
            entry += deal.amount
        else:
            entry -= deal.amount
            
        self.ledger[pair] = entry
        
        count = self.tx_counts.get(pair, 0)
        self.tx_counts[pair] = count + 1
        
        if abs(entry) >= self.threshold:
            net = entry
            tx_count = self.tx_counts[pair]
            
            self.ledger[pair] = 0.0
            self.tx_counts[pair] = 0
            self.total_settled += 1
            
            from_agent, to_agent = (pair[0], pair[1]) if net > 0.0 else (pair[1], pair[0])
            
            return {
                "between": [from_agent, to_agent],
                "net_amount": abs(net),
                "tx_count": tx_count,
            }
            
        return None

def resolve_auction(
    ordered_bids: List[Bid],
    slot: AdSlot,
    trust_scores: Dict[str, float],
    trust_threshold: float
) -> Optional[Tuple[Deal, float]]:
    
    # In FoxMQ demo, ordered_bids arrive via MQTT. We just pick the highest valid bid.
    now_ms = int(time.time() * 1000)
    
    valid_bids = []
    for bid in ordered_bids:
        score = trust_scores.get(bid.bidder, 0.0)
        if score >= trust_threshold and bid.amount >= slot.min_price:
            valid_bids.append(bid)
            
    if not valid_bids:
        # print(f"No valid bids! total bids: {len(ordered_bids)}, min_price: {slot.min_price}, threshold: {trust_threshold}")
        return None
        
    winner = max(valid_bids, key=lambda b: b.amount)
    latency_ms = now_ms - winner.timestamp_ms
    
    deal = Deal(
        id=f"deal-{slot.id}-{now_ms}",
        buyer=winner.bidder,
        seller=slot.publisher,
        slot_id=slot.id,
        amount=winner.amount,
        latency_ms=latency_ms,
        timestamp_ms=now_ms
    )
    
    return deal, latency_ms

def compute_mesh(agents: List[AgentInfo], recent_deals: List[Deal]) -> Tuple[List[MeshNode], List[MeshEdge]]:
    nodes = []
    for i, a in enumerate(agents):
        angle = (i / len(agents)) * 2 * math.pi
        radius = 300.0
        nodes.append(MeshNode(
            id=a.id,
            role=a.role,
            active=a.active,
            x=400.0 + radius * math.cos(angle),
            y=350.0 + radius * math.sin(angle)
        ))
        
    edge_weights: Dict[Tuple[str, str], float] = {}
    
    # Take at most 100 most recent deals
    for deal in list(reversed(recent_deals))[:100]:
        key = tuple(sorted([deal.buyer, deal.seller]))
        edge_weights[key] = edge_weights.get(key, 0.0) + deal.amount
        
    edges = [
        MeshEdge(**{"from": k[0], "to": k[1], "weight": w}) 
        for k, w in edge_weights.items()
    ]
        
    return nodes, edges

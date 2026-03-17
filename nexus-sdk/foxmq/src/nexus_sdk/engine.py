import asyncio
import json
import logging
import time
from typing import Callable, List, Optional, Dict, Any
import paho.mqtt.client as mqtt

from .types import Bid, Event

logger = logging.getLogger("nexus_sdk.engine")


class OrderedBid:
    __slots__ = ("bid", "sequence", "consensus_time_ms")

    def __init__(self, bid: Bid, sequence: int, consensus_time_ms: float):
        self.bid = bid
        self.sequence = sequence
        self.consensus_time_ms = consensus_time_ms


class FoxMQEngine:
    def __init__(self, agent_id: str,
                 broker_host: str = "127.0.0.1",
                 broker_port: int = 1883,
                 username: Optional[str] = None,
                 password: Optional[str] = None):
        self.agent_id = agent_id
        self.broker_host = broker_host
        self.broker_port = broker_port

        self.client = mqtt.Client(
            client_id=f"nexus_{agent_id}",
            protocol=mqtt.MQTTv5
        )

        if username and password:
            self.client.username_pw_set(username, password)

        self.client.on_connect = self._on_connect
        self.client.on_message = self._on_message
        self.client.on_disconnect = self._on_disconnect

        self.connected = False
        self._loop: Optional[asyncio.AbstractEventLoop] = None
        # Per-slot queues for collecting bids back from the broker
        self._bid_queues: Dict[str, asyncio.Queue] = {}

    def connect(self, loop: Optional[asyncio.AbstractEventLoop] = None):
        self._loop = loop or asyncio.get_event_loop()
        logger.info(f"Connecting to FoxMQ at {self.broker_host}:{self.broker_port}")
        will_payload = json.dumps({"agent": self.agent_id})
        self.client.will_set(f"swarm/fail/{self.agent_id}", will_payload, qos=1, retain=False)

        self.client.connect(self.broker_host, self.broker_port, keepalive=60)
        self.client.loop_start()

    def disconnect(self):
        self.client.loop_stop()
        self.client.disconnect()

    def _on_connect(self, client, userdata, flags, reason_code, properties):
        if reason_code == 0:
            logger.info("Connected to FoxMQ broker successfully")
            self.connected = True
            self.client.subscribe("swarm/bids/#", qos=1)
        else:
            logger.error(f"Failed to connect: {reason_code}")

    def _on_disconnect(self, client, userdata, reason_code, properties):
        logger.warning(f"Disconnected from broker: {reason_code}")
        self.connected = False

    def _on_message(self, client, userdata, msg):
        """Called from MQTT background thread — route bids into the asyncio queue."""
        try:
            if msg.topic.startswith("swarm/bids/"):
                slot_id = msg.topic.rsplit("/", 1)[-1]
                queue = self._bid_queues.get(slot_id)
                if queue and self._loop:
                    bid = Bid.model_validate_json(msg.payload)
                    self._loop.call_soon_threadsafe(queue.put_nowait, bid)
        except Exception as e:
            logger.error(f"Error processing message on {msg.topic}: {e}")

    async def run_batch_cycle(self, bids: List[Bid], slot_id: str) -> List[OrderedBid]:
        """Publish bids to FoxMQ, wait for them to arrive back through the
        broker subscription, and return them in broker-delivered order.
        Mirrors VertexEngine::run_batch_cycle."""
        if not bids or not self.connected:
            return []

        expected = len(bids)
        queue: asyncio.Queue[Bid] = asyncio.Queue()
        self._bid_queues[slot_id] = queue
        start = time.monotonic()

        # Publish all bids to the broker
        topic = f"swarm/bids/{slot_id}"
        for bid in bids:
            self.client.publish(topic, bid.model_dump_json(), qos=1)

        # Collect ordered bids as they arrive back from the subscription
        ordered: List[OrderedBid] = []
        timeout = 0.5  # 500ms absolute cap, matching Vertex

        while len(ordered) < expected:
            remaining = timeout - (time.monotonic() - start)
            if remaining <= 0:
                break
            try:
                bid = await asyncio.wait_for(queue.get(), timeout=remaining)
                ordered.append(OrderedBid(
                    bid=bid,
                    sequence=len(ordered),
                    consensus_time_ms=(time.monotonic() - start) * 1000.0,
                ))
            except asyncio.TimeoutError:
                break

        del self._bid_queues[slot_id]
        return ordered

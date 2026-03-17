import json
import logging
import time
from typing import Callable, Optional, Dict, Any
import paho.mqtt.client as mqtt

from .types import Bid, Event

logger = logging.getLogger("nexus_sdk.engine")

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
        
        self._on_event_callback: Optional[Callable[[Dict[str, Any]], None]] = None
        self._on_bid_callback: Optional[Callable[[Bid], None]] = None
        
        self.connected = False

    def set_event_callback(self, cb: Callable[[Dict[str, Any]], None]):
        """Callback for all events (Deal, Metrics, MeshUpdate, etc.)"""
        self._on_event_callback = cb
        
    def set_bid_callback(self, cb: Callable[[Bid], None]):
        """Callback specifically for bids from other agents"""
        self._on_bid_callback = cb

    def connect(self):
        logger.info(f"Connecting to FoxMQ at {self.broker_host}:{self.broker_port}")
        # Set Last Will for agent failure detection
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
            # Subscribe to the entire swarm topics
            self.client.subscribe("swarm/event", qos=1)
            self.client.subscribe("swarm/bids/#", qos=1)
        else:
            logger.error(f"Failed to connect: {reason_code}")

    def _on_disconnect(self, client, userdata, reason_code, properties):
        logger.warning(f"Disconnected from broker: {reason_code}")
        self.connected = False

    def _on_message(self, client, userdata, msg):
        try:
            payload_str = msg.payload.decode("utf-8")
            data = json.loads(payload_str)
            
            if msg.topic == "swarm/event":
                if self._on_event_callback:
                    self._on_event_callback(data)
            elif msg.topic.startswith("swarm/bids/"):
                if self._on_bid_callback:
                    # data is expected to be a serialized sequence containing sequence # and bid
                    # For simplification, assume payload is purely the bid json for now.
                    bid = Bid.model_validate(data)
                    self._on_bid_callback(bid)
                    
        except Exception as e:
            logger.error(f"Error processing message on {msg.topic}: {e}")

    def publish_event(self, event_type: str, payload: dict):
        """Publish a global event (e.g. from the central auctioneer/coordinator if used)"""
        # Inject standard event type flag 
        payload["type"] = event_type
        self.client.publish("swarm/event", json.dumps(payload), qos=1)

    def publish_bid(self, bid: Bid):
        """Publish a bid for a specific slot to be totally ordered by FoxMQ."""
        # QoS 2 ensures exactly once delivery in ordered stream
        topic = f"swarm/bids/{bid.slot_id}"
        self.client.publish(topic, bid.model_dump_json(), qos=2)

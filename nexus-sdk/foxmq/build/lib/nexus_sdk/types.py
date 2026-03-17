from enum import Enum
from typing import Dict, List, Optional, Union, Tuple
from pydantic import BaseModel, Field, ConfigDict

AgentId = str

class Role(str, Enum):
    publisher = "publisher"
    advertiser = "advertiser"

class Mandate(BaseModel):
    max_bid: float
    trust_threshold: float
    daily_spend_cap: float

class AdSlot(BaseModel):
    id: str
    publisher: AgentId
    category: str
    min_price: float

class Bid(BaseModel):
    id: str
    bidder: AgentId
    slot_id: str
    amount: float
    timestamp_ms: int

class Ask(BaseModel):
    slot: AdSlot
    timestamp_ms: int

class Deal(BaseModel):
    id: str
    buyer: AgentId
    seller: AgentId
    slot_id: str
    amount: float
    latency_ms: float
    timestamp_ms: int

class Settlement(BaseModel):
    between: Tuple[AgentId, AgentId]
    net_amount: float
    tx_count: int
    timestamp_ms: int

class AgentInfo(BaseModel):
    id: AgentId
    role: Role
    trust_score: float
    mandate: Mandate
    balance: float
    active: bool

class MeshNode(BaseModel):
    id: AgentId
    role: Role
    active: bool
    x: float
    y: float

class MeshEdge(BaseModel):
    model_config = ConfigDict(populate_by_name=True)
    from_: AgentId = Field(alias="from", serialization_alias="from", validation_alias="from")
    to: AgentId
    weight: float

# Event subtypes
class EventAgentCreated(BaseModel):
    type: str = "agent_created"
    agent: AgentInfo

class EventBid(BaseModel):
    model_config = ConfigDict(populate_by_name=True)
    type: str = "bid"
    from_: AgentId = Field(alias="from", serialization_alias="from", validation_alias="from")
    slot: AdSlot
    amount: float
    timestamp: int

class EventDeal(BaseModel):
    type: str = "deal"
    buyer: AgentId
    seller: AgentId
    amount: float
    latency_ms: float
    slot: AdSlot

class EventSettlement(BaseModel):
    type: str = "settlement"
    between: List[AgentId]
    net_amount: float
    tx_count: int

class EventMetrics(BaseModel):
    type: str = "metrics"
    tps: float
    avg_latency_ms: float
    total_deals: int
    total_settled: int
    active_agents: int

class EventMeshUpdate(BaseModel):
    type: str = "mesh_update"
    nodes: List[MeshNode]
    edges: List[MeshEdge]

class EventAgentFailed(BaseModel):
    type: str = "agent_failed"
    agent: AgentId

class EventAgentHealed(BaseModel):
    type: str = "agent_healed"
    agent: AgentId

Event = Union[
    EventAgentCreated,
    EventBid,
    EventDeal,
    EventSettlement,
    EventMetrics,
    EventMeshUpdate,
    EventAgentFailed,
    EventAgentHealed
]

class Command(BaseModel):
    cmd: str

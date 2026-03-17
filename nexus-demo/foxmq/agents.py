import random
import uuid
from typing import List

from nexus_sdk.types import AgentInfo, Role, Mandate, AdSlot

CATEGORIES = [
    "sports", "tech", "finance", "gaming", "health", "travel", "food", "music",
]

def generate_agent_id() -> str:
    return uuid.uuid4().hex[:12]

def spawn_agents(count: int) -> List[AgentInfo]:
    agents = []
    
    for i in range(count):
        # Roughly 40% publishers, 60% advertisers
        if i < count * 2 / 5:
            role = Role.publisher
            mandate = Mandate(
                max_bid=0.0,
                trust_threshold=random.uniform(0.3, 0.7),
                daily_spend_cap=0.0
            )
        else:
            role = Role.advertiser
            mandate = Mandate(
                max_bid=random.uniform(5.0, 100.0),
                trust_threshold=random.uniform(0.3, 0.7),
                daily_spend_cap=random.uniform(500.0, 5000.0)
            )
            
        agents.append(AgentInfo(
            id=generate_agent_id(),
            role=role,
            trust_score=1.0,
            mandate=mandate,
            balance=10000.0 if role == Role.advertiser else 0.0,
            active=True
        ))
        
    return agents

def generate_slot(publisher: AgentInfo) -> AdSlot:
    cat = random.choice(CATEGORIES)
    slot_id = f"slot-{publisher.id[:6]}-{random.randint(1000, 9999)}"
    
    return AdSlot(
        id=slot_id,
        publisher=publisher.id,
        category=cat,
        min_price=random.uniform(1.0, 20.0)
    )

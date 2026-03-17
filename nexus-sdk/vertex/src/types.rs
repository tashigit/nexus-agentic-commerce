use serde::{Deserialize, Serialize};
use std::fmt;

pub type AgentId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Publisher,
    Advertiser,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Publisher => write!(f, "publisher"),
            Role::Advertiser => write!(f, "advertiser"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mandate {
    pub max_bid: f64,
    pub trust_threshold: f64,
    pub daily_spend_cap: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdSlot {
    pub id: String,
    pub publisher: AgentId,
    pub category: String,
    pub min_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub id: String,
    pub bidder: AgentId,
    pub slot_id: String,
    pub amount: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ask {
    pub slot: AdSlot,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: String,
    pub buyer: AgentId,
    pub seller: AgentId,
    pub slot_id: String,
    pub amount: f64,
    pub latency_ms: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub between: (AgentId, AgentId),
    pub net_amount: f64,
    pub tx_count: u32,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "agent_created")]
    AgentCreated {
        agent: AgentInfo,
    },
    #[serde(rename = "bid")]
    Bid {
        from: AgentId,
        slot: AdSlot,
        amount: f64,
        timestamp: u64,
    },
    #[serde(rename = "deal")]
    Deal {
        buyer: AgentId,
        seller: AgentId,
        amount: f64,
        latency_ms: f64,
        slot: AdSlot,
    },
    #[serde(rename = "settlement")]
    Settlement {
        between: Vec<AgentId>,
        net_amount: f64,
        tx_count: u32,
    },
    #[serde(rename = "metrics")]
    Metrics {
        tps: f64,
        avg_latency_ms: f64,
        total_deals: u64,
        total_settled: u64,
        active_agents: u32,
    },
    #[serde(rename = "mesh_update")]
    MeshUpdate {
        nodes: Vec<MeshNode>,
        edges: Vec<MeshEdge>,
    },
    #[serde(rename = "agent_failed")]
    AgentFailed {
        agent: AgentId,
    },
    #[serde(rename = "agent_healed")]
    AgentHealed {
        agent: AgentId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub role: Role,
    pub trust_score: f64,
    pub mandate: Mandate,
    pub balance: f64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshNode {
    pub id: AgentId,
    pub role: Role,
    pub active: bool,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshEdge {
    pub from: AgentId,
    pub to: AgentId,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub cmd: String,
}

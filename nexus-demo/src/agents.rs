use nexus_sdk::identity::Agent;
use nexus_sdk::types::{AdSlot, Mandate, Role};
use rand::Rng;

const CATEGORIES: &[&str] = &[
    "sports", "tech", "finance", "gaming", "health", "travel", "food", "music",
];

pub fn spawn_agents(count: usize) -> Vec<Agent> {
    let mut rng = rand::thread_rng();
    let mut agents = Vec::with_capacity(count);

    for i in 0..count {
        // Roughly 40% publishers, 60% advertisers
        let role = if i < count * 2 / 5 {
            Role::Publisher
        } else {
            Role::Advertiser
        };

        let mandate = match role {
            Role::Publisher => Mandate {
                max_bid: 0.0, // publishers don't bid
                trust_threshold: rng.gen_range(0.3..0.7),
                daily_spend_cap: 0.0,
            },
            Role::Advertiser => Mandate {
                max_bid: rng.gen_range(5.0..100.0),
                trust_threshold: rng.gen_range(0.3..0.7),
                daily_spend_cap: rng.gen_range(500.0..5000.0),
            },
        };

        agents.push(Agent::new(role, mandate));
    }

    agents
}

pub fn generate_slot(publisher: &Agent) -> AdSlot {
    let mut rng = rand::thread_rng();
    let cat = CATEGORIES[rng.gen_range(0..CATEGORIES.len())];

    AdSlot {
        id: format!("slot-{}-{}", &publisher.id[..6], rng.gen_range(1000..9999)),
        publisher: publisher.id.clone(),
        category: cat.to_string(),
        min_price: rng.gen_range(1.0..20.0),
    }
}

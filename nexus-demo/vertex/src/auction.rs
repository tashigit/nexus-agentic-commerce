use std::collections::HashMap;

use nexus_sdk::clearing::ClearingHouse;
use nexus_sdk::engine::VertexEngine;
use nexus_sdk::handshake::{now_ms, resolve_auction};
use nexus_sdk::identity::Agent;
use nexus_sdk::types::*;
use rand::Rng;
use tokio::sync::broadcast;

use crate::agents::generate_slot;

pub struct AuctionState {
    pub agents: Vec<Agent>,
    pub clearing: ClearingHouse,
    pub engine: VertexEngine,
    pub total_deals: u64,
    pub total_bids: u64,
    pub latency_sum: f64,
    pub surge_multiplier: f64,
    pub recent_deals: Vec<Deal>,
}

impl AuctionState {
    pub async fn new(agents: Vec<Agent>) -> Self {
        AuctionState {
            agents,
            clearing: ClearingHouse::new(200.0),
            engine: VertexEngine::new().await.unwrap(),
            total_deals: 0,
            total_bids: 0,
            latency_sum: 0.0,
            surge_multiplier: 1.0,
            recent_deals: Vec::new(),
        }
    }
}

/// Run one auction round: pick a random publisher, generate a slot,
/// collect bids from advertisers, run consensus, resolve.
pub async fn run_auction_round(
    state: &mut AuctionState,
    event_tx: &broadcast::Sender<Event>,
) {
    let mut rng = rand::thread_rng();

    // Pick active publishers
    let publishers: Vec<usize> = state
        .agents
        .iter()
        .enumerate()
        .filter(|(_, a)| a.role == Role::Publisher && a.active)
        .map(|(i, _)| i)
        .collect();

    if publishers.is_empty() {
        return;
    }

    // Run multiple concurrent auctions based on surge multiplier
    let auctions_per_round = (state.surge_multiplier as usize).max(1).min(publishers.len());

    for _ in 0..auctions_per_round {
        let pub_idx = publishers[rng.gen_range(0..publishers.len())];
        let slot = generate_slot(&state.agents[pub_idx]);

        // Collect bids from active advertisers
        let mut pending_bids = Vec::new();
        let trust_threshold = state.agents[pub_idx].mandate.trust_threshold;

        for agent in state.agents.iter().filter(|a| {
            a.role == Role::Advertiser && a.active
        }) {
            // Each advertiser bids with some probability
            if rng.gen_bool(0.4) {
                let amount = rng.gen_range(slot.min_price..agent.mandate.max_bid.max(slot.min_price + 1.0));
                if agent.can_bid(amount) {
                    let bid = Bid {
                        id: format!("bid-{}-{}", &agent.id[..6], now_ms()),
                        bidder: agent.id.clone(),
                        slot_id: slot.id.clone(),
                        amount,
                        timestamp_ms: now_ms(),
                    };

                    let _ = event_tx.send(Event::Bid {
                        from: agent.id.clone(),
                        slot: slot.clone(),
                        amount,
                        timestamp: bid.timestamp_ms,
                    });

                    pending_bids.push(bid);
                    state.total_bids += 1;
                }
            }
        }

        if pending_bids.is_empty() {
            continue;
        }

        // Run simulated consensus
        let ordered = state.engine.run_batch_cycle(&mut pending_bids).await;

        // Build trust map
        let trust_map: HashMap<AgentId, f64> = state
            .agents
            .iter()
            .map(|a| (a.id.clone(), a.trust_score))
            .collect();

        // Resolve auction
        if let Some((deal, latency)) = resolve_auction(&ordered, &slot, &trust_map, trust_threshold) {
            // Update balances
            if let Some(buyer) = state.agents.iter_mut().find(|a| a.id == deal.buyer) {
                buyer.spend(deal.amount);
            }
            if let Some(seller) = state.agents.iter_mut().find(|a| a.id == deal.seller) {
                seller.earn(deal.amount);
            }

            state.total_deals += 1;
            state.latency_sum += latency;
            state.recent_deals.push(deal.clone());

            let _ = event_tx.send(Event::Deal {
                buyer: deal.buyer.clone(),
                seller: deal.seller.clone(),
                amount: deal.amount,
                latency_ms: deal.latency_ms,
                slot: slot.clone(),
            });

            // Check clearing
            if let Some(settlement) = state.clearing.record_deal(&deal) {
                let _ = event_tx.send(Event::Settlement {
                    between: vec![settlement.between.0.clone(), settlement.between.1.clone()],
                    net_amount: settlement.net_amount,
                    tx_count: settlement.tx_count,
                });
            }
        }
    }
}

pub fn compute_mesh(agents: &[Agent], recent_deals: &[Deal]) -> (Vec<MeshNode>, Vec<MeshEdge>) {
    let nodes: Vec<MeshNode> = agents
        .iter()
        .enumerate()
        .map(|(i, a)| {
            // Arrange in a circle
            let angle = (i as f64 / agents.len() as f64) * std::f64::consts::TAU;
            let radius = 300.0;
            MeshNode {
                id: a.id.clone(),
                role: a.role,
                active: a.active,
                x: 400.0 + radius * angle.cos(),
                y: 350.0 + radius * angle.sin(),
            }
        })
        .collect();

    // Edges from recent deals
    let mut edge_weights: HashMap<(String, String), f64> = HashMap::new();
    for deal in recent_deals.iter().rev().take(100) {
        let key = if deal.buyer <= deal.seller {
            (deal.buyer.clone(), deal.seller.clone())
        } else {
            (deal.seller.clone(), deal.buyer.clone())
        };
        *edge_weights.entry(key).or_insert(0.0) += deal.amount;
    }

    let edges: Vec<MeshEdge> = edge_weights
        .into_iter()
        .map(|((from, to), weight)| MeshEdge { from, to, weight })
        .collect();

    (nodes, edges)
}

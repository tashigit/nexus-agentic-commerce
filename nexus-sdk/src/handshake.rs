use crate::engine::OrderedBid;
use crate::types::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Resolves an auction: given ordered bids for a slot, picks the highest valid bid.
pub fn resolve_auction(
    ordered_bids: &[OrderedBid],
    slot: &AdSlot,
    trust_scores: &HashMap<AgentId, f64>,
    trust_threshold: f64,
) -> Option<(Deal, f64)> {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Filter bids by trust threshold, then pick highest amount
    let winner = ordered_bids
        .iter()
        .filter(|ob| {
            let score = trust_scores.get(&ob.bid.bidder).copied().unwrap_or(0.0);
            score >= trust_threshold && ob.bid.amount >= slot.min_price
        })
        .max_by(|a, b| a.bid.amount.partial_cmp(&b.bid.amount).unwrap());

    winner.map(|ob| {
        let deal = Deal {
            id: format!("deal-{}-{}", slot.id, now_ms),
            buyer: ob.bid.bidder.clone(),
            seller: slot.publisher.clone(),
            slot_id: slot.id.clone(),
            amount: ob.bid.amount,
            latency_ms: ob.consensus_time_ms,
            timestamp_ms: now_ms,
        };
        (deal, ob.consensus_time_ms)
    })
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

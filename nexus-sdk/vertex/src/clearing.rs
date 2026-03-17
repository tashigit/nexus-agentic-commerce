use std::collections::HashMap;

use crate::types::{AgentId, Deal, Settlement};
use crate::handshake::now_ms;

/// Virtual ledger tracking net debts between agent pairs.
/// Settles in batch when threshold is hit.
pub struct ClearingHouse {
    /// Maps (debtor, creditor) -> net amount owed
    ledger: HashMap<(AgentId, AgentId), f64>,
    /// Number of transactions per pair
    tx_counts: HashMap<(AgentId, AgentId), u32>,
    /// Settlement threshold
    pub threshold: f64,
    pub total_settled: u64,
}

impl ClearingHouse {
    pub fn new(threshold: f64) -> Self {
        ClearingHouse {
            ledger: HashMap::new(),
            tx_counts: HashMap::new(),
            threshold,
            total_settled: 0,
        }
    }

    /// Record a deal in the ledger. Returns a Settlement if threshold is hit.
    pub fn record_deal(&mut self, deal: &Deal) -> Option<Settlement> {
        let pair = normalize_pair(&deal.buyer, &deal.seller);
        let is_forward = pair.0 == deal.buyer;

        let entry = self.ledger.entry(pair.clone()).or_insert(0.0);
        if is_forward {
            *entry += deal.amount; // buyer owes seller
        } else {
            *entry -= deal.amount;
        }

        let count = self.tx_counts.entry(pair.clone()).or_insert(0);
        *count += 1;

        if entry.abs() >= self.threshold {
            let net = *entry;
            let tx_count = *count;
            *entry = 0.0;
            *count = 0;
            self.total_settled += 1;

            let (from, to) = if net > 0.0 {
                (pair.0.clone(), pair.1.clone())
            } else {
                (pair.1.clone(), pair.0.clone())
            };

            Some(Settlement {
                between: (from, to),
                net_amount: net.abs(),
                tx_count,
                timestamp_ms: now_ms(),
            })
        } else {
            None
        }
    }

    pub fn pending_pairs(&self) -> usize {
        self.ledger.values().filter(|v| v.abs() > 0.0).count()
    }

    pub fn max_pending_ratio(&self) -> f64 {
        self.ledger
            .values()
            .map(|v| v.abs() / self.threshold)
            .fold(0.0_f64, f64::max)
    }
}

fn normalize_pair(a: &AgentId, b: &AgentId) -> (AgentId, AgentId) {
    if a <= b {
        (a.clone(), b.clone())
    } else {
        (b.clone(), a.clone())
    }
}

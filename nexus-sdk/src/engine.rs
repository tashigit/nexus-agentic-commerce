use std::time::{Duration, Instant};

use crate::types::Bid;

/// Trait representing a consensus engine (Vertex or simulated).
pub trait ConsensusEngine: Send + Sync {
    /// Submit a bid for ordering. Returns when consensus is reached.
    fn submit_bid(
        &self,
        bid: Bid,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64, EngineError>> + Send>>;
}

#[derive(Debug)]
pub enum EngineError {
    Timeout,
    ChannelClosed,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Timeout => write!(f, "consensus timeout"),
            EngineError::ChannelClosed => write!(f, "channel closed"),
        }
    }
}

impl std::error::Error for EngineError {}

/// In-memory simulated consensus engine mimicking Vertex's 26ms BFT.
/// Fair ordering via timestamp sorting within each batch window.
pub struct SimulatedEngine {
    batch_window: Duration,
}

#[derive(Debug, Clone)]
pub struct OrderedBid {
    pub bid: Bid,
    pub sequence: u64,
    pub consensus_time_ms: f64,
}

impl SimulatedEngine {
    pub fn new() -> Self {
        SimulatedEngine {
            batch_window: Duration::from_millis(26),
        }
    }

    /// Collect bids within the batch window, sort by timestamp (fair ordering),
    /// assign sequence numbers, and return them.
    pub async fn run_batch_cycle(&self, pending: &mut Vec<Bid>) -> Vec<OrderedBid> {
        let start = Instant::now();

        // Simulate the ~26ms consensus window
        let jitter = rand::random::<f64>() * 6.0; // 0-6ms jitter
        let target = self.batch_window + Duration::from_micros((jitter * 1000.0) as u64);

        if start.elapsed() < target {
            tokio::time::sleep(target - start.elapsed()).await;
        }

        let elapsed = start.elapsed();

        // Fair ordering: sort by timestamp
        pending.sort_by_key(|b| b.timestamp_ms);

        let mut ordered = Vec::with_capacity(pending.len());
        for (i, bid) in pending.drain(..).enumerate() {
            ordered.push(OrderedBid {
                bid,
                sequence: i as u64,
                consensus_time_ms: elapsed.as_secs_f64() * 1000.0,
            });
        }

        ordered
    }
}

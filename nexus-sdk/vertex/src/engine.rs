use std::time::Instant;
use std::sync::Arc;
use crate::types::Bid;
use tashi_vertex::{Context, Engine, KeySecret, Message, Options, Peers, Socket, Transaction};

/// Simulated/Wrapper representation of an ordered bid
#[derive(Debug, Clone)]
pub struct OrderedBid {
    pub bid: Bid,
    pub sequence: u64,
    pub consensus_time_ms: f64,
}

pub struct VertexEngine {
    engine: Arc<Engine>,
    _context: Arc<Context>,
}

impl VertexEngine {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let secret = KeySecret::generate();
        let mut peers = Peers::new()?;
        
        // Randomize the port to avoid conflicts
        let port = 10000 + (rand::random::<u16>() % 20000);
        let addr = format!("127.0.0.1:{}", port);
        
        peers.insert(&addr, &secret.public(), Default::default())?;

        let context = Context::new()?;
        let socket = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            Socket::bind(&context, &addr)
        ).await??;

        let engine = Engine::start(&context, socket, Options::default(), &secret, peers)?;

        Ok(VertexEngine {
            engine: Arc::new(engine),
            _context: Arc::new(context),
        })
    }

    pub async fn run_batch_cycle(&self, pending: &mut Vec<Bid>) -> Vec<OrderedBid> {
        let num_bids = pending.len();
        if num_bids == 0 {
            return vec![];
        }

        let mut ordered = Vec::with_capacity(num_bids);
        let mut sequence = 0;
        let start = Instant::now();
        let expected = num_bids;

        for bid in pending.drain(..) {
            if let Ok(data) = serde_json::to_vec(&bid) {
                let mut tx = Transaction::allocate(data.len());
                tx.copy_from_slice(&data);
                let _ = self.engine.send_transaction(tx);
            }
        }

        // We know we sent exactly `expected` bids, so await them through consensus
        let abs_timeout = std::time::Duration::from_millis(500); 
        while ordered.len() < expected && start.elapsed() < abs_timeout {
            if let Ok(Ok(Some(msg))) = tokio::time::timeout(std::time::Duration::from_millis(20), self.engine.recv_message()).await {
                match msg {
                    Message::Event(event) => {
                        for tx in event.transactions() {
                            if let Ok(bid) = serde_json::from_slice::<Bid>(tx) {
                                ordered.push(OrderedBid {
                                    bid,
                                    sequence,
                                    consensus_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                                });
                                sequence += 1;
                            }
                        }
                    }
                    Message::SyncPoint(_) => {}
                }
            }
        }
        
        ordered
    }
}

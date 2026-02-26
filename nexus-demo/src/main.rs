mod agents;
mod auction;
mod ws_server;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use nexus_sdk::types::{AgentInfo, Event};
use rand::Rng;
use tokio::sync::{broadcast, mpsc};

use crate::auction::{compute_mesh, run_auction_round, AuctionState};

const AGENT_COUNT: usize = 50;
const AUCTION_INTERVAL_MS: u64 = 50; // Base interval between auction rounds
const METRICS_INTERVAL_MS: u64 = 500;
const MESH_INTERVAL_MS: u64 = 1000;

#[tokio::main]
async fn main() {
    println!("=== Nexus-HFC Demo ===");
    println!("Spawning {} agents...", AGENT_COUNT);

    let agent_list = agents::spawn_agents(AGENT_COUNT);

    let (event_tx, _event_rx) = broadcast::channel::<Event>(10_000);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(100);

    // Snapshot agent info for replaying to new WS clients
    let initial_agents: Arc<Vec<AgentInfo>> = Arc::new(
        agent_list.iter().map(|a| a.info()).collect()
    );

    let publishers = agent_list.iter().filter(|a| a.role == nexus_sdk::types::Role::Publisher).count();
    let advertisers = agent_list.iter().filter(|a| a.role == nexus_sdk::types::Role::Advertiser).count();
    println!("  {} publishers, {} advertisers", publishers, advertisers);

    let mut state = AuctionState::new(agent_list);

    // Start WebSocket server
    let ws_addr: SocketAddr = "0.0.0.0:3001".parse().unwrap();
    let event_tx_ws = event_tx.clone();
    let initial_agents_ws = initial_agents.clone();
    tokio::spawn(async move {
        ws_server::run_ws_server(ws_addr, event_tx_ws, cmd_tx, initial_agents_ws).await;
    });

    println!("Starting auction loop...\n");

    let mut auction_interval = tokio::time::interval(Duration::from_millis(AUCTION_INTERVAL_MS));
    let mut metrics_interval = tokio::time::interval(Duration::from_millis(METRICS_INTERVAL_MS));
    let mut mesh_interval = tokio::time::interval(Duration::from_millis(MESH_INTERVAL_MS));

    let mut last_deal_count: u64 = 0;
    let mut tps_window_start = tokio::time::Instant::now();

    loop {
        tokio::select! {
            _ = auction_interval.tick() => {
                run_auction_round(&mut state, &event_tx).await;
            }

            _ = metrics_interval.tick() => {
                let elapsed = tps_window_start.elapsed().as_secs_f64();
                let deals_in_window = state.total_deals - last_deal_count;
                let tps = if elapsed > 0.0 { deals_in_window as f64 / elapsed } else { 0.0 };
                let avg_latency = if state.total_deals > 0 {
                    state.latency_sum / state.total_deals as f64
                } else {
                    0.0
                };
                let active = state.agents.iter().filter(|a| a.active).count() as u32;

                let _ = event_tx.send(Event::Metrics {
                    tps,
                    avg_latency_ms: avg_latency,
                    total_deals: state.total_deals,
                    total_settled: state.clearing.total_settled,
                    active_agents: active,
                });

                last_deal_count = state.total_deals;
                tps_window_start = tokio::time::Instant::now();
            }

            _ = mesh_interval.tick() => {
                let (nodes, edges) = compute_mesh(&state.agents, &state.recent_deals);
                let _ = event_tx.send(Event::MeshUpdate { nodes, edges });

                // Trim old deals to keep mesh relevant
                if state.recent_deals.len() > 200 {
                    state.recent_deals.drain(..100);
                }
            }

            Some(cmd) = cmd_rx.recv() => {
                match cmd.as_str() {
                    "surge" => {
                        println!("[cmd] Market Surge!");
                        state.surge_multiplier = 5.0;
                        // Reset after 10 seconds
                        // Decays gradually via the 0.995 multiplier in the main loop
                    }
                    "attack" => {
                        println!("[cmd] Network Attack!");
                        let mut rng = rand::thread_rng();
                        // Fail ~30% of agents
                        for agent in state.agents.iter_mut() {
                            if rng.gen_bool(0.3) && agent.active {
                                agent.active = false;
                                agent.degrade_trust(0.3);
                                let _ = event_tx.send(Event::AgentFailed {
                                    agent: agent.id.clone(),
                                });
                            }
                        }
                    }
                    "heal" => {
                        println!("[cmd] Healing network...");
                        for agent in state.agents.iter_mut() {
                            if !agent.active {
                                agent.active = true;
                                agent.restore_trust(0.2);
                                let _ = event_tx.send(Event::AgentHealed {
                                    agent: agent.id.clone(),
                                });
                            }
                        }
                        state.surge_multiplier = 1.0;
                    }
                    _ => {
                        eprintln!("[cmd] unknown command: {}", cmd);
                    }
                }
            }
        }

        // Decay surge multiplier
        if state.surge_multiplier > 1.0 {
            state.surge_multiplier *= 0.995;
            if state.surge_multiplier < 1.1 {
                state.surge_multiplier = 1.0;
            }
        }
    }
}

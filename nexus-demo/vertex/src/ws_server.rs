use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use nexus_sdk::types::{AgentInfo, Command, Event};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;

pub async fn run_ws_server(
    addr: SocketAddr,
    event_rx: broadcast::Sender<Event>,
    cmd_tx: mpsc::Sender<String>,
    initial_agents: Arc<Vec<AgentInfo>>,
) {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind WebSocket server");
    println!("[ws] listening on ws://{}", addr);

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[ws] accept error: {}", e);
                continue;
            }
        };

        let event_rx = event_rx.subscribe();
        let cmd_tx = cmd_tx.clone();
        let agents = initial_agents.clone();
        tokio::spawn(handle_connection(stream, peer, event_rx, cmd_tx, agents));
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer: SocketAddr,
    mut event_rx: broadcast::Receiver<Event>,
    cmd_tx: mpsc::Sender<String>,
    initial_agents: Arc<Vec<AgentInfo>>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[ws] handshake error from {}: {}", peer, e);
            return;
        }
    };

    println!("[ws] new connection from {}", peer);
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // Replay initial agent state to the new client
    for agent in initial_agents.iter() {
        let event = Event::AgentCreated { agent: agent.clone() };
        if let Ok(json) = serde_json::to_string(&event) {
            if ws_tx.send(Message::Text(json.into())).await.is_err() {
                return;
            }
        }
    }

    // Forward live events to WebSocket client
    let send_task = tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    if let Ok(json) = serde_json::to_string(&event) {
                        if ws_tx.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[ws] {} lagged by {} messages", peer, n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Receive commands from client
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            if let Message::Text(text) = msg {
                if let Ok(cmd) = serde_json::from_str::<Command>(&text) {
                    let _ = cmd_tx.send(cmd.cmd).await;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    println!("[ws] {} disconnected", peer);
}

use anyhow::Context;
use argh::FromArgs;
use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use tasklens_sync_protocol::{ClientMessage, ServerMessage};
use tokio::sync::broadcast;

mod db;

/// Command line arguments for the sync server.
#[derive(FromArgs)]
struct Args {
    /// the path to the sqlite database file
    #[argh(option, short = 'd', default = "\"updates.db\".to_string()")]
    database_path: String,

    /// the port to listen on
    #[argh(option, short = 'p', default = "3000")]
    port: u16,

    /// the host to bind to
    #[argh(option, default = "String::from(\"127.0.0.1\")")]
    host: String,

    /// enable debug logging
    #[argh(switch)]
    debug: bool,
}

// Application State
#[derive(Clone)]
struct AppState {
    db: db::DbPool,
    // Broadcast channel for notifying connected clients of new updates.
    // Capacity of 100 should be sufficient for MVP bursts.
    // NOTE: DashMap<String, broadcast::Sender> would be better for scaling (per-room channels),
    // but for MVP a single bus with filtering is simpler and sufficient.
    tx: broadcast::Sender<ServerMessage>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    let log_level = if args.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Initialize Database
    let db_pool = db::init_pool(&args.database_path).expect("Failed to initialize database");
    tracing::info!("Database initialized at {}", args.database_path);

    // Initialize Broadcast Channel
    let (tx, _rx) = broadcast::channel(100);

    let app_state = AppState { db: db_pool, tx };

    let app = Router::new()
        .route("/sync", get(ws_handler))
        .with_state(app_state);

    let addr = format!("{}:{}", args.host, args.port);
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to {}", addr))?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // 1. Handshake Phase
    // The client MUST send Hello as the first message.
    let mut client_id = String::new();
    let mut discovery_key = String::new();

    while let Some(Ok(msg)) = receiver.next().await {
        if let axum::extract::ws::Message::Text(text) = msg
            && let Ok(ClientMessage::Hello {
                client_id: cid,
                discovery_key: dkey,
                last_sequence,
            }) = serde_json::from_str(&text)
        {
            tracing::debug!("Received Hello: {:?}", text);
            client_id = cid;
            discovery_key = dkey;

            tracing::info!(%client_id, %discovery_key, "Client connected. Replaying from seq {}", last_sequence);

            // Replay missed messages from DB
            match db::get_changes_since(&state.db, &discovery_key, last_sequence) {
                Ok(changes) => {
                    for change in changes {
                        let json = serde_json::to_string(&change).unwrap();
                        tracing::debug!("Replaying change to {}: {}", client_id, json);
                        if let Err(e) = sender
                            .send(axum::extract::ws::Message::Text(json.into()))
                            .await
                        {
                            tracing::warn!(
                                "Failed to send replayed message to {}: {:?}",
                                client_id,
                                e
                            );
                            return; // Client disconnected
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to fetch changes: {:?}", e);
                    return;
                }
            }
            break; // Handshake complete
        }
        // If we get anything else first, ignore or disconnect.
        // For robustness, we just wait for Hello.
    }

    if client_id.is_empty() {
        tracing::warn!("Client disconnected before handshake");
        return;
    }

    // 2. Live Sync Phase
    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            // A. Receive from Client (Push)
            Some(Ok(msg)) = receiver.next() => {
                tracing::debug!("Received message from {}: {:?}", client_id, msg);
                if let axum::extract::ws::Message::Text(text) = msg
                    && let Ok(ClientMessage::SubmitChange { discovery_key: target_dkey, payload }) = serde_json::from_str(&text) {
                        // Validate target room (optional but good practice)
                        if target_dkey != discovery_key {
                             tracing::warn!(%client_id, "Client tried to push to wrong discovery_key");
                             continue;
                        }

                        // Persist to DB
                        match db::append_update(&state.db, &discovery_key, &client_id, &payload) {
                            Ok(new_seq) => {
                                // Broadcast to others
                                let notification = ServerMessage::ChangeOccurred {
                                    sequence_id: new_seq,
                                    discovery_key: discovery_key.clone(),
                                    source_client_id: client_id.clone(),
                                    payload
                                };
                                tracing::debug!("Broadcasting change from {}: {:?}", client_id, notification);
                                if let Err(e) = state.tx.send(notification) {
                                    tracing::error!("Failed to broadcast change from {}: {:?}", client_id, e);
                                }
                            }
                            Err(e) => tracing::error!("Failed to persist update: {:?}", e),
                        }
                    }
            }

            // B. Receive from Broadcast (Pull)
            Ok(msg) = rx.recv() => {
                match msg {
                     ServerMessage::ChangeOccurred { discovery_key: ref msg_dkey, ref source_client_id, .. } => {
                        // Filter 1: Must be for this room
                        if msg_dkey != &discovery_key {
                            continue;
                        }
                        // Filter 2: Don't echo back to sender
                        if source_client_id == &client_id {
                            continue;
                        }

                        // Forward to client
                        let json = serde_json::to_string(&msg).unwrap();
                        tracing::debug!("Forwarding to {}: {}", client_id, json);
                        if let Err(e) = sender.send(axum::extract::ws::Message::Text(json.into())).await {
                            tracing::warn!("Failed to forward message to {}: {:?}", client_id, e);
                            break;
                        }
                     }
                }
            }

            else => break,
        }
    }

    tracing::info!(%client_id, "Client disconnected");
}

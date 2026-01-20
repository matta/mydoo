use crate::{ClientMessage, ServerMessage};
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::prelude::*;
use gloo_net::websocket::{Message, futures::WebSocket};

/// A pure client for the TaskLens Sync Protocol.
///
/// This client handles the WebSocket transport, handshake, and message passing.
/// It is ignorant of the content of the payload (encryption, automerge, etc).
#[derive(Clone)]
pub struct SyncProtocolClient {
    tx_outbound: UnboundedSender<Message>,
}

impl SyncProtocolClient {
    /// Connects to the Sync Server.
    ///
    /// Returns a tuple of (Client, InboundStream).
    /// The inbound stream yields `ServerMessage`s from the server.
    pub async fn connect(
        url: &str,
        client_id: String,
        discovery_key: String,
        last_sequence: i64,
    ) -> Result<(Self, UnboundedReceiver<ServerMessage>), String> {
        let ws = WebSocket::open(url).map_err(|e| format!("WS connection failed: {}", e))?;

        let (mut write, mut read) = ws.split();
        let (tx_outbound, mut rx_outbound) = mpsc::unbounded::<Message>();
        let (tx_inbound, rx_inbound) = mpsc::unbounded::<ServerMessage>();

        // 1. Send Hello immediately
        let hello = ClientMessage::Hello {
            client_id: client_id.clone(),
            discovery_key: discovery_key.clone(),
            last_sequence,
        };
        let hello_json = serde_json::to_string(&hello).map_err(|e| e.to_string())?;
        write
            .send(Message::Text(hello_json))
            .await
            .map_err(|e| format!("Failed to send Hello: {}", e))?;

        // 2. Spawn Write Loop
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(msg) = rx_outbound.next().await {
                if let Err(e) = write.send(msg).await {
                    // Log error or break? For now, we just stop writing.
                    // On read failure, the connection is considered dead.
                    let _ = e;
                    break;
                }
            }
        });

        // 3. Spawn Read Loop
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text)
                            && tx_inbound.unbounded_send(server_msg).is_err()
                        {
                            break;
                        }
                    }
                    Ok(Message::Bytes(_)) => {} // Ignore bytes
                    Err(_) => break,            // Error means disconnect
                }
            }
        });

        Ok((Self { tx_outbound }, rx_inbound))
    }

    /// Sends a payload to the joined channel.
    pub fn send(&self, discovery_key: String, payload: Vec<u8>) {
        let msg = ClientMessage::SubmitChange {
            discovery_key,
            payload,
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = self.tx_outbound.unbounded_send(Message::Text(json));
        }
    }
}

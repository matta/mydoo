#[cfg(target_arch = "wasm32")]
mod implementation {
    use crate::crypto;
    use anyhow::{Context, Result, anyhow};
    use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
    use futures::{SinkExt, StreamExt};
    use gloo_net::websocket::{Message, futures::WebSocket};
    use gloo_timers::future::TimeoutFuture;
    use rand::Rng;
    use tasklens_sync_protocol::{ClientMessage, ServerMessage};

    pub struct SyncService {
        pub ws: Option<WebSocket>,
        pub sync_id: String,
        pub client_id: String,
        pub master_key: [u8; 32],
    }

    impl SyncService {
        pub fn new(master_key: [u8; 32]) -> Self {
            let sync_id = crypto::derive_sync_id(&master_key);
            // We need a stable client_id. Usually this is per-session to avoid echo issues.
            // Let's generate one random UUID for this session.
            let client_id = uuid::Uuid::new_v4().to_string();

            Self {
                ws: None,
                sync_id,
                client_id,
                master_key,
            }
        }

        pub async fn connect(&mut self) -> Result<()> {
            let loc = web_sys::window().unwrap().location();
            let host = loc.host().unwrap();
            let protocol = if loc.protocol().unwrap() == "https:" {
                "wss"
            } else {
                "ws"
            };

            let url = if host.contains("8080") {
                // Phase 4: Point to the Mac Mini sync server for local development
                "ws://mac-mini.lan:8080/sync".to_string()
            } else {
                format!("{}://{}/sync", protocol, host)
            };

            let ws = WebSocket::open(&url).map_err(|e| anyhow!("WS Error: {:?}", e))?;
            self.ws = Some(ws);

            // 1. Send Hello
            // We can pass `last_sequence: 0` for now, assuming full replay or we can track it later.
            let hello = ClientMessage::Hello {
                client_id: self.client_id.clone(),
                sync_id: self.sync_id.clone(),
                last_sequence: 0,
            };

            self.send(hello).await.context("Failed to send Hello")?;

            Ok(())
        }

        pub async fn send(&mut self, msg: ClientMessage) -> Result<()> {
            if let Some(ws) = &mut self.ws {
                let json = serde_json::to_string(&msg)?;
                ws.send(Message::Text(json))
                    .await
                    .map_err(|e| anyhow!("WS Send Error: {:?}", e))?;
            }
            Ok(())
        }

        pub async fn start_loop<S>(
            &mut self,
            rx_local_changes: &mut S,
            mut tx_remote_changes: futures::channel::mpsc::UnboundedSender<Vec<u8>>,
        ) where
            S: futures::stream::Stream<Item = Vec<u8>> + futures::stream::FusedStream + Unpin,
        {
            let mut ws = if let Some(ws) = self.ws.take() {
                ws.fuse()
            } else {
                return;
            };

            loop {
                futures::select! {
                   local_change = rx_local_changes.next() => {
                       if let Some(data) = local_change {
                            match crypto::encrypt_change(&data, &self.master_key) {
                                Ok(blob) => {
                                    let msg = ClientMessage::SubmitChange {
                                        sync_id: self.sync_id.clone(),
                                        payload: blob,
                                    };
                                    let json_res = serde_json::to_string(&msg);
                                    if let Ok(json) = json_res
                                        && let Err(e) = ws.send(Message::Text(json)).await
                                    {
                                        tracing::error!("WS Send Error: {:?}", e);
                                        break;
                                    }
                                }
                                Err(e) => tracing::error!("Encryption failed: {:?}", e),
                            }
                       } else {
                           break;
                       }
                   }

                   msg = ws.next() => {
                       match msg {
                           Some(Ok(Message::Text(text))) => {
                                                           if let Ok(ServerMessage::ChangeOccurred { payload, .. }) = serde_json::from_str::<ServerMessage>(&text) {
                                                               match crypto::decrypt_change(&payload, &self.master_key) {
                                                                   Ok(plaintext) => {
                                                                       if let Err(e) = tx_remote_changes.send(plaintext).await {
                                                                            tracing::error!("Failed to forward change: {:?}", e);
                                                                            break;
                                                                       }
                                                                   }
                                                                   Err(e) => tracing::error!("Decryption failed: {:?}", e),
                                                               }
                                                           }                       }
                           Some(Ok(Message::Bytes(_))) => tracing::warn!("Unexpected binary msg"),
                           Some(Err(e)) => {
                               tracing::error!("WS Error: {:?}", e);
                               break;
                           }
                           None => {
                               tracing::info!("WS Closed");
                               break;
                           }
                       }
                   }
                }
            }
            // Restore WS (though it might be closed/errored)
            self.ws = Some(ws.into_inner());
        }
    }

    pub async fn run_sync_loop(
        rx_local: UnboundedReceiver<Vec<u8>>,
        tx_remote: UnboundedSender<Vec<u8>>,
        master_key: impl Fn() -> Option<[u8; 32]>,
        mut get_full_state: impl FnMut() -> Vec<u8> + 'static,
    ) {
        let mut failure_count = 0;
        const BASE_DELAY_MS: u64 = 1000;
        const MAX_DELAY_MS: u64 = 30_000;
        const MIN_STABLE_CONN_TIME_MS: f64 = 5_000.0;

        let mut current_key: Option<[u8; 32]> = None;
        // Fuse the receiver so we can iterate it in the loop
        let mut rx_local = rx_local.fuse();

        loop {
            // 1. Check/Wait for Key
            let desired_key = master_key();
            if current_key != desired_key {
                current_key = desired_key;
                failure_count = 0; // Reset backoff on key change
                if current_key.is_some() {
                    tracing::info!("Key set. Starting Sync Loop.");
                } else {
                    tracing::info!("Key cleared. Stopping Sync Loop.");
                }
            }

            if current_key.is_none() {
                TimeoutFuture::new(200).await;
                continue;
            }

            let key = current_key.unwrap();

            // 2. Backoff if needed
            if failure_count > 0 {
                let exp_delay = BASE_DELAY_MS.saturating_mul(2u64.pow(failure_count.min(6)));
                let delay = exp_delay.min(MAX_DELAY_MS);

                // Add jitter (0-20% of delay)
                let jitter = rand::thread_rng().gen_range(0..=(delay / 5));
                let total_delay = delay + jitter;

                tracing::warn!(
                    "Connection failure #{}. Backing off for {}ms...",
                    failure_count,
                    total_delay
                );
                TimeoutFuture::new(total_delay as u32).await;
            }

            // Check key again after sleep in case it changed
            if master_key() != Some(key) {
                continue;
            }

            // 3. Connect
            let mut svc = SyncService::new(key);
            tracing::info!("Attempting to connect...");

            match svc.connect().await {
                Err(e) => {
                    tracing::error!("Connect failed: {:?}", e);
                    failure_count += 1;
                    continue;
                }
                Ok(_) => {
                    tracing::info!("Connected to Sync Server.");
                }
            }

            // 4. Initial Sync State
            let full_save = get_full_state();
            let encrypted_payload = match crypto::encrypt_change(&full_save, &key) {
                Ok(payload) => payload,
                Err(e) => {
                    tracing::error!("Failed to encrypt initial state: {:?}", e);
                    failure_count += 1;
                    continue;
                }
            };
            if let Err(e) = svc
                .send(ClientMessage::SubmitChange {
                    sync_id: svc.sync_id.clone(),
                    payload: encrypted_payload,
                })
                .await
            {
                tracing::error!("Failed to push initial state: {:?}", e);
                // If we can't send hello/initial state, count as failure
                failure_count += 1;
                continue;
            }

            // 5. Run Loop
            // We use web_sys::window().performance() to time the session stability
            let perf = web_sys::window().unwrap().performance().unwrap();
            let start_time = perf.now();

            let loop_fut = Box::pin(svc.start_loop(&mut rx_local, tx_remote.clone()));

            let check_key_fut = Box::pin(async {
                loop {
                    TimeoutFuture::new(500).await;
                    if master_key() != Some(key) {
                        return;
                    }
                }
            });

            use futures::future::{Either, select};
            match select(loop_fut, check_key_fut).await {
                Either::Left((_, _)) => {
                    // Service finished (disconnected)
                    let duration = perf.now() - start_time;
                    if duration < MIN_STABLE_CONN_TIME_MS {
                        tracing::warn!(
                            "Sync loop ended quickly ({:.0}ms). Flapping detected.",
                            duration
                        );
                        failure_count += 1;
                    } else {
                        tracing::warn!("Sync loop ended after {:.0}ms. Reconnecting...", duration);
                        failure_count = 0;
                        // Optional small delay after a long session to prevent tight loops on persistent server crashes
                        TimeoutFuture::new(1000).await;
                    }
                }
                Either::Right((_, _)) => {
                    // Key changed!
                    tracing::info!("Master key changed. Disconnecting...");
                    failure_count = 0;
                    // Loop will catch key change at top
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod stub {
    use anyhow::Result;
    use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
    // use crate::crypto; // Unused in stub
    use tasklens_sync_protocol::ClientMessage;

    pub struct SyncService {
        pub sync_id: String,
        pub client_id: String,
        pub master_key: [u8; 32],
    }

    impl SyncService {
        pub fn new(master_key: [u8; 32]) -> Self {
            Self {
                sync_id: "stub".to_string(),
                client_id: "stub".to_string(),
                master_key,
            }
        }
        pub async fn connect(&mut self) -> Result<()> {
            Ok(())
        }
        pub async fn send(&mut self, _msg: ClientMessage) -> Result<()> {
            Ok(())
        }

        pub async fn start_loop<S>(
            &mut self,
            _rx_local_changes: &mut S,
            _tx_remote_changes: UnboundedSender<Vec<u8>>,
        ) where
            S: futures::stream::Stream<Item = Vec<u8>> + futures::stream::FusedStream + Unpin,
        {
            futures::future::pending::<()>().await;
        }
    }

    pub async fn run_sync_loop(
        _rx_local: UnboundedReceiver<Vec<u8>>,
        _tx_remote: UnboundedSender<Vec<u8>>,
        _master_key: impl Fn() -> Option<[u8; 32]>,
        mut _get_full_state: impl FnMut() -> Vec<u8> + 'static,
    ) {
        futures::future::pending::<()>().await;
    }
}

#[cfg(target_arch = "wasm32")]
pub use implementation::*;

#[cfg(not(target_arch = "wasm32"))]
pub use stub::*;

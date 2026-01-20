#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Disconnected,
    Connecting,
    Connected,
    Syncing,
    Error(String),
}

#[cfg(target_arch = "wasm32")]
mod implementation {
    use crate::crypto;
    use anyhow::{Context, Result, anyhow};
    use futures::StreamExt;
    use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
    use gloo_timers::future::TimeoutFuture;
    use rand::Rng;
    use tasklens_sync_protocol::client::SyncProtocolClient;

    use super::SyncStatus;

    /// The Bridge between the Application (Store) and the Protocol (SyncProtocolClient).
    ///
    /// Responsibilities:
    /// 1. Identity Translation: DocId <-> DiscoveryKey
    /// 2. Encryption: Plaintext <-> EncryptedBlob
    pub struct DocSyncBridge {
        pub client: SyncProtocolClient,
        pub discovery_key: String,
        pub encryption_key: [u8; 32],
    }

    impl DocSyncBridge {
        pub async fn connect(
            url: &str,
            doc_id: [u8; 32],
            client_id: String,
        ) -> Result<(
            Self,
            UnboundedReceiver<tasklens_sync_protocol::ServerMessage>,
        )> {
            // 1. Derive Keys
            // In our model, DocId IS the MasterKey.
            let encryption_key = doc_id;
            let discovery_key = crypto::derive_sync_id(&doc_id);

            // 2. Connect via Pure Client
            let (client, rx) =
                SyncProtocolClient::connect(url, client_id, discovery_key.clone(), 0)
                    .await
                    .map_err(|e| anyhow!("Protocol Error: {}", e))?;

            Ok((
                Self {
                    client,
                    discovery_key,
                    encryption_key,
                },
                rx,
            ))
        }

        /// Encrypts and sends a local change to the server.
        pub fn send_change(&self, plaintext: &[u8]) -> Result<()> {
            let blob = crypto::encrypt_change(plaintext, &self.encryption_key)?;
            let payload = serde_json::to_vec(&blob).expect("Failed to serialize EncryptedBlob");
            self.client.send(self.discovery_key.clone(), payload);
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run_sync_loop(
        rx_local: UnboundedReceiver<Vec<u8>>,
        tx_remote: UnboundedSender<Vec<u8>>,
        status_tx: UnboundedSender<SyncStatus>,
        credentials_provider: impl Fn() -> Option<[u8; 32]>, // Returns DocId
        url_provider: impl Fn() -> Option<String>,
        mut get_full_state: impl FnMut() -> Vec<u8> + 'static,
    ) {
        let mut failure_count = 0;
        const BASE_DELAY_MS: u64 = 1000;
        const MAX_DELAY_MS: u64 = 30_000;
        const MIN_STABLE_CONN_TIME_MS: f64 = 5_000.0;

        // Session ID is stable for the lifetime of this loop (tab)
        let client_id = uuid::Uuid::new_v4().to_string();

        let mut current_doc_id: Option<[u8; 32]> = None;
        // Fuse the receiver so we can iterate it in the loop
        let mut rx_local = rx_local.fuse();

        loop {
            // 1. Check/Wait for DocId
            let desired_doc_id = credentials_provider();
            if current_doc_id != desired_doc_id {
                current_doc_id = desired_doc_id;
                failure_count = 0; // Reset backoff on key change
                if current_doc_id.is_some() {
                    tracing::info!("DocID set. Starting Sync Loop.");
                } else {
                    tracing::info!("DocID cleared. Stopping Sync Loop.");
                    let _ = status_tx.unbounded_send(SyncStatus::Disconnected);
                }
            }

            if current_doc_id.is_none() {
                TimeoutFuture::new(200).await;
                continue;
            }

            let doc_id = current_doc_id.unwrap();

            // 2. Check/Wait for URL
            let desired_url = url_provider();
            if desired_url.is_none() {
                let _ = status_tx.unbounded_send(SyncStatus::Disconnected);
                TimeoutFuture::new(500).await;
                continue;
            }
            let url = desired_url.unwrap();

            // 3. Backoff if needed
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
                let _ = status_tx
                    .unbounded_send(SyncStatus::Error(format!("Retrying in {}ms", total_delay)));
                TimeoutFuture::new(total_delay as u32).await;
            }

            // Check keys again after sleep
            if credentials_provider() != Some(doc_id) || url_provider() != Some(url.clone()) {
                continue;
            }

            // 4. Connect Bridge
            tracing::info!("Attempting to connect to {}...", url);
            let _ = status_tx.unbounded_send(SyncStatus::Connecting);

            let (bridge, mut rx_server) =
                match DocSyncBridge::connect(&url, doc_id, client_id.clone()).await {
                    Ok(res) => res,
                    Err(e) => {
                        tracing::error!("Connect failed: {:?}", e);
                        let _ = status_tx.unbounded_send(SyncStatus::Error(format!("{:?}", e)));
                        failure_count += 1;
                        continue;
                    }
                };

            tracing::info!("Connected to Sync Server.");
            let _ = status_tx.unbounded_send(SyncStatus::Connected);

            // 5. Initial State Push
            let full_save = get_full_state();
            if let Err(e) = bridge.send_change(&full_save) {
                tracing::error!("Failed to encrypt/send initial state: {:?}", e);
                failure_count += 1;
                continue;
            }

            // 6. Run Loop
            let perf = web_sys::window().unwrap().performance().unwrap();
            let start_time = perf.now();

            let bridge_ref = &bridge;

            // A. Server -> Store (Decrypt)
            let inbound_fut = async {
                while let Some(msg) = rx_server.next().await {
                    if let tasklens_sync_protocol::ServerMessage::ChangeOccurred {
                        payload, ..
                    } = msg
                    {
                        // Deserialize EncryptedBlob
                        if let Ok(blob) = serde_json::from_slice::<
                            tasklens_sync_protocol::EncryptedBlob,
                        >(&payload)
                        {
                            // Decrypt
                            if let Ok(plaintext) =
                                crypto::decrypt_change(&blob, &bridge_ref.encryption_key)
                            {
                                if let Err(e) = tx_remote.unbounded_send(plaintext) {
                                    tracing::error!("Failed to forward change to store: {:?}", e);
                                    break;
                                }
                            } else {
                                tracing::error!("Decryption failed for incoming message");
                            }
                        }
                    }
                }
                "Inbound Stream Ended"
            };

            // B. Store -> Server (Encrypt)
            let outbound_fut = async {
                while let Some(local_change) = rx_local.next().await {
                    if let Err(e) = bridge_ref.send_change(&local_change) {
                        tracing::error!("Failed to send local change: {:?}", e);
                        break;
                    }
                }
                "Outbound Stream Ended"
            };

            // C. Watchdog
            let check_creds_fut = async {
                loop {
                    TimeoutFuture::new(500).await;
                    if credentials_provider() != Some(doc_id) || url_provider() != Some(url.clone())
                    {
                        break;
                    }
                }
                "Credentials Changed"
            };

            use futures::future::{Either, select};
            // Run until any future exits
            let loop_res = select(
                Box::pin(inbound_fut),
                select(Box::pin(outbound_fut), Box::pin(check_creds_fut)),
            )
            .await;

            // Determine why we exited
            let reason = match loop_res {
                Either::Left((r, _)) => r,
                Either::Right((Either::Left((r, _)), _)) => r,
                Either::Right((Either::Right((r, _)), _)) => r,
            };

            if reason == "Credentials Changed" {
                tracing::info!("Credentials changed. Restarting loop.");
                failure_count = 0;
            } else {
                // Disconnected
                let duration = perf.now() - start_time;
                if duration < MIN_STABLE_CONN_TIME_MS {
                    tracing::warn!("Sync loop ended quickly ({:.0}ms). Flapping.", duration);
                    failure_count += 1;
                } else {
                    tracing::warn!("Sync loop ended after {:.0}ms. Reconnecting...", duration);
                    failure_count = 0;
                    TimeoutFuture::new(1000).await;
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod stub {
    use super::SyncStatus;
    use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

    #[allow(clippy::too_many_arguments)]
    pub async fn run_sync_loop(
        _rx_local: UnboundedReceiver<Vec<u8>>,
        _tx_remote: UnboundedSender<Vec<u8>>,
        _status_tx: UnboundedSender<SyncStatus>,
        _credentials_provider: impl Fn() -> Option<[u8; 32]>,
        _url_provider: impl Fn() -> Option<String>,
        mut _get_full_state: impl FnMut() -> Vec<u8> + 'static,
    ) {
        futures::future::pending::<()>().await;
    }
}

#[cfg(target_arch = "wasm32")]
pub use implementation::*;

#[cfg(not(target_arch = "wasm32"))]
pub use stub::*;

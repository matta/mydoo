use dioxus::prelude::*;
use futures::StreamExt;
use futures::channel::mpsc;
use gloo_storage::{LocalStorage, Storage};
use tasklens_store::crypto;
pub use tasklens_store::network::SyncStatus;
use tasklens_store::network::run_sync_loop;
use tasklens_store::store::AppStore;

pub const SYNC_SERVER_URL_KEY: &str = "tasklens_sync_server_url";

pub fn use_sync_client(mut store: Signal<AppStore>) -> Signal<SyncStatus> {
    let mut status = use_signal(|| SyncStatus::Disconnected);
    let mut tx_local_signal = use_signal(|| None::<mpsc::UnboundedSender<Vec<u8>>>);

    use_future(move || async move {
        let (tx_local, rx_local) = mpsc::unbounded::<Vec<u8>>();
        let (tx_remote, mut rx_remote) = mpsc::unbounded::<Vec<u8>>();
        let (tx_status, mut rx_status) = mpsc::unbounded::<SyncStatus>();
        tx_local_signal.set(Some(tx_local));

        // Background loop for network sync
        spawn(async move {
            run_sync_loop(
                rx_local,
                tx_remote,
                tx_status,
                || crypto::load_key().ok().flatten(),
                || LocalStorage::get::<String>(SYNC_SERVER_URL_KEY).ok(),
                move || {
                    // Initial state push - currently not strictly required as server
                    // handles catch-up, but good to have.
                    // Vec::new()
                    // To get full state, we'd need access to the store here,
                    // which is tricky across threads without a lock.
                    // For now, we rely on incremental changes.
                    Vec::new()
                },
            )
            .await;
        });

        // Remote -> Store
        spawn(async move {
            let mut store = store;
            while let Some(changes) = rx_remote.next().await {
                tracing::info!("Received remote changes: {} bytes", changes.len());
                if let Err(e) = store.write().import_changes(changes) {
                    tracing::error!("Failed to import remote changes in sync hook: {:?}", e);
                }
            }
        });

        // Status -> Signal
        spawn(async move {
            while let Some(new_status) = rx_status.next().await {
                status.set(new_status);
            }
        });
    });

    // Store -> Local (Observer)
    // We use a polling strategy to detect local changes.
    // This avoids potential infinite loops where writing to the store (after saving incremental changes)
    // triggers the effect again.
    use_future(move || async move {
        let mut last_synced_heads: Vec<automerge::ChangeHash> = Vec::new();

        loop {
            // Check every 500ms
            gloo_timers::future::TimeoutFuture::new(500).await;

            if let Some(tx) = tx_local_signal() {
                let mut s = store.write();
                // Use get_changes_since to avoid conflict with persistence which clears the
                // internal incremental buffer via save().
                if let Some(changes) = s.get_changes_since(&last_synced_heads) {
                    tracing::info!("Pushing local changes to sync: {} bytes", changes.len());
                    let _ = tx.unbounded_send(changes);

                    // Update our tracker to the current heads of the doc so we only
                    // send subsequent changes next time.
                    last_synced_heads = s.heads.clone();
                }
            }
        }
    });

    status
}

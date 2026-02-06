use dioxus::prelude::*;
use futures::StreamExt;
use tasklens_store::store::AppStore;

fn format_heads(heads: &[automerge::ChangeHash]) -> String {
    if heads.is_empty() {
        String::new()
    } else {
        heads
            .iter()
            .map(|h| format!("{:?}", h))
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Hook to manage background persistence tracking for the active Automerge document.
///
/// This hook spawns a long-running future that monitors the `AppStore` for document changes
/// and persistence events. It updates two types of head tracking signals:
///
/// * `MemoryHeads`: Represents the "optimistic" state of the document in memory immediately
///   after a mutation occurs in the store.
/// * `PersistedHeads`: Represents the "confirmed" state of the document once the storage
///   engine (Samod) has successfully written and flushed the changes to disk.
///
/// The hook implementation includes logic to detect document swaps (e.g., when a user imports
/// a new file or creates a new document). It will restart its internal subscription loops
/// whenever the `current_id` in the store diverges from the one it is currently tracking.
///
/// # Arguments
///
/// * `store` - Access to the global `AppStore` containing the document handle and repo.
/// * `memory_heads` - A wrapper around a string signal that tracks memory heads.
/// * `persisted_heads` - A wrapper around a string signal that tracks persisted heads.
pub fn use_persistence(
    store: Signal<AppStore>,
    mut memory_heads: crate::MemoryHeads,
    mut persisted_heads: crate::PersistedHeads,
) {
    // Spawn a task to track both in-memory changes and persisted state
    use_future(move || async move {
        use futures::future::FutureExt;
        tracing::debug!("use_persistence: tracker initialized");

        let mut current_id = None;

        loop {
            // 1. Wait for store to have a handle AND it's a different document than before
            let (handle, id) = loop {
                let s = store.read();
                if let (Some(h), Some(id)) = (s.handle.clone(), s.current_id)
                    && Some(id) != current_id
                {
                    break (h, id);
                }
                drop(s);
                crate::utils::async_utils::sleep(100).await;
            };

            tracing::info!("use_persistence: tracking new document {}", id);
            current_id = Some(id);

            // Set initial heads from the current document state
            let initial_heads = handle.with_document(|doc| format_heads(&doc.get_heads()));
            memory_heads.set(initial_heads.clone());
            persisted_heads.set(initial_heads);

            // 2. Subscribe to both changes and persisted events reactively
            let mut changes = handle.changes().fuse();
            let mut persisted_stream = handle.persisted().fuse();

            // 3. Inner loop for this document
            loop {
                // Check if the handle has changed in the store
                if store.read().current_id != current_id {
                    tracing::debug!(
                        "use_persistence: document changed in store, restarting tracker"
                    );
                    break; // Restart outer loop for new handle
                }

                futures::select! {
                    changed = changes.next() => if let Some(changed) = changed {
                        let heads_str = format_heads(&changed.new_heads);
                        tracing::debug!(
                            "use_persistence: document changed, new heads: {}",
                            heads_str
                        );
                        memory_heads.set(heads_str);
                    },
                    persisted = persisted_stream.next() => if let Some(persisted) = persisted {
                        let heads_str = format_heads(&persisted.persisted_heads);
                        tracing::debug!("use_persistence: document persisted, heads: {}", heads_str);
                        persisted_heads.set(heads_str);
                    },
                    _ = crate::utils::async_utils::sleep(500).fuse() => {
                        // Periodic wakeup to check if document ID changed
                    },
                    complete => {
                        tracing::debug!("use_persistence: streams completed for doc {}", id);
                        break;
                    }
                }
            }
        }
    });
}

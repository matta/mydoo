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

pub fn use_persistence(
    store: Signal<AppStore>,
    mut memory_heads: crate::MemoryHeads,
    mut persisted_heads: crate::PersistedHeads,
) {
    // Spawn a task to track both in-memory changes and persisted state
    use_future(move || async move {
        tracing::debug!("use_persistence: tracker initialized");

        // Wait for store to have a handle
        let handle = loop {
            let handle_opt = store.read().handle.clone();
            if let Some(handle) = handle_opt {
                break handle;
            }
            crate::utils::async_utils::sleep(100).await;
        };

        // Set initial heads from the current document state
        let initial_heads = handle.with_document(|doc| format_heads(&doc.get_heads()));
        memory_heads.set(initial_heads.clone());
        persisted_heads.set(initial_heads);

        // Subscribe to both changes and persisted events reactively
        let mut changes = handle.changes().fuse();
        let mut persisted = handle.persisted().fuse();

        loop {
            futures::select! {
                change = changes.next() => {
                    if let Some(changed) = change {
                        let heads_str = format_heads(&changed.new_heads);
                        tracing::debug!(
                            "use_persistence: document changed, new heads: {}",
                            heads_str
                        );
                        memory_heads.set(heads_str);
                    } else {
                        break;
                    }
                },
                persist = persisted.next() => {
                    if let Some(persisted_event) = persist {
                        let heads_str = format_heads(&persisted_event.persisted_heads);
                        tracing::debug!(
                            "use_persistence: document persisted, heads: {}",
                            heads_str
                        );
                        persisted_heads.set(heads_str);
                    } else {
                        break;
                    }
                }
            }
        }
    });
}

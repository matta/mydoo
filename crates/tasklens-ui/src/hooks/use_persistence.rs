use dioxus::prelude::*;
use tasklens_store::store::AppStore;

pub fn use_persistence(
    store: Signal<AppStore>,
    mut memory_heads: crate::MemoryHeads,
    mut persisted_heads: crate::PersistedHeads,
) {
    // Removed use_context calls as we now accept signals directly to avoid context resolution issues
    // in the root component.

    // Spawn a task that polls document heads and updates the signals
    use_future(move || async move {
        tracing::debug!("use_persistence: hook initialized");
        loop {
            // Poll every 100ms
            crate::utils::async_utils::sleep(100).await;

            let handle_opt = store.read().handle.clone();

            if let Some(handle) = handle_opt {
                // Get the current heads from the document
                let heads = handle.with_document(|doc| {
                    let heads_vec = doc.get_heads();
                    if heads_vec.is_empty() {
                        String::new()
                    } else {
                        heads_vec
                            .iter()
                            .map(|h| format!("{:?}", h))
                            .collect::<Vec<_>>()
                            .join(",")
                    }
                });

                let current_mem = memory_heads.read().clone();

                // Update memory heads and persisted heads
                if current_mem != heads {
                    // TODO: It is silly: there is no functional difference
                    // between memory_heads and persisted_heads. We can get rid
                    // of one of them. The original point was to track the
                    // in-memory doc and the persisted doc so we could update UI
                    // when pending changes were comitted.
                    memory_heads.set(heads.clone());
                    persisted_heads.set(heads);
                }
            }
        }
    });
}

use dioxus::prelude::*;
use tasklens_store::store::AppStore;

pub fn use_persistence(
    store: Signal<AppStore>,
    mut memory_heads: Signal<String>,
    mut persisted_heads: Signal<String>,
) {
    // Removed use_context calls as we now accept signals directly to avoid context resolution issues
    // in the root component.

    // Spawn a task that polls document heads and updates the signals
    use_future(move || async move {
        tracing::debug!("use_persistence: hook initialized");
        loop {
            // Poll every 100ms
            gloo_timers::future::TimeoutFuture::new(100).await;

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
                    memory_heads.set(heads.clone());
                    persisted_heads.set(heads);
                }
            }
        }
    });
}

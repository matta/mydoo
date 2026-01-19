use automerge::ChangeHash;
use dioxus::prelude::*;
use tasklens_store::store::AppStore;

use crate::{MemoryHeads, PersistedHeads};

pub fn use_persistence(mut store: Signal<AppStore>) {
    let mh_context = try_use_context::<MemoryHeads>();
    let ph_context = try_use_context::<PersistedHeads>();

    // Track the last saved heads.
    let mut last_saved_heads = use_signal(Vec::<ChangeHash>::new);
    // Track the last saved document ID.
    let mut last_saved_doc_id = use_signal(|| None::<String>);

    // 1. Observable State Update (MemoryHeads)
    // Synchronously derive MemoryHeads for E2E tests using cached heads.
    // This avoids render loops because it only uses store.read().
    let memory_heads_str = use_memo(move || {
        let s = store.read();
        format_state(&s.current_id.to_string(), &s.heads)
    });

    use_effect(move || {
        if let Some(mut mh) = mh_context {
            mh.0.set(memory_heads_str());
        }
    });

    // 2. Async Persistence Side Effect
    use_effect(move || {
        let (current_heads, current_id, id_str) = {
            let s = store.read();
            (
                s.heads.clone(),
                s.current_id.clone(),
                s.current_id.to_string(),
            )
        };

        // Check if we need to save
        let should_save = {
            let last_heads = last_saved_heads.read();
            let last_id_str = last_saved_doc_id.read();
            current_heads != *last_heads || Some(id_str.clone()) != *last_id_str
        };

        if should_save {
            // Update tracking signals synchronously to prevent re-entry loops
            // during the re-render triggered by store.write().doc.save() below.
            *last_saved_heads.write() = current_heads.clone();
            *last_saved_doc_id.write() = Some(id_str.clone());

            let bytes = store.write().doc.save();
            let doc_id_for_save = current_id.clone();
            let id_str_for_save = id_str.clone();
            let heads_for_save = current_heads.clone();

            spawn(async move {
                tracing::info!("Starting auto-save for {}...", id_str_for_save);
                match AppStore::save_doc_data_async(&doc_id_for_save, bytes).await {
                    Ok(_) => {
                        tracing::info!(
                            "Auto-saved doc {} with heads {:?}",
                            id_str_for_save,
                            heads_for_save
                        );
                        AppStore::save_active_doc_id(&doc_id_for_save);

                        if let Some(mut ph) = ph_context {
                            ph.0.set(format_state(&id_str_for_save, &heads_for_save));
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to auto-save doc {}: {:?}", id_str_for_save, e);
                    }
                }
            });
        }
    });

    // Format helper
    fn format_state(id: &str, heads: &[ChangeHash]) -> String {
        let heads_str = heads
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!("{}:{}", id, heads_str)
    }
}

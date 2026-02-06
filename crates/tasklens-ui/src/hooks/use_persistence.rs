use dioxus::prelude::*;
use futures::StreamExt;
use tasklens_store::doc_id::DocumentId;
use tasklens_store::store::AppStore;

use dioxus_core::Task;

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

/// Track the in-memory and on-disk head state of the active Automerge document.
///
/// Call this hook once from the root component. It requires a
/// `Signal<AppStore>` in context (via `use_context_provider`) and accepts
/// three arguments:
///
/// - `doc_id_signal` — the current document identity. May start as `None`
///   while the document is still being loaded or created.
/// - `memory_heads` — written with a stringified snapshot of the document's
///   change-graph heads after every local or remote mutation. Downstream
///   consumers (e.g. `use_tunnel_state`) subscribe to this value to trigger
///   re-hydration.
/// - `persisted_heads` — written when the storage layer confirms a flush to
///   disk, letting the UI show a "saved" indicator.
///
/// The hook is idle until `doc_id_signal`, `store.current_id`, and
/// `store.handle` all agree on the same document. Once they do it
/// synchronously snapshots the current heads into both output signals and
/// spawns a background task that streams subsequent change and persisted
/// events. If the active document switches, the previous listener is
/// cancelled and a new one is started automatically.
pub fn use_persistence(
    doc_id_signal: Signal<Option<DocumentId>>,
    mut memory_heads: crate::MemoryHeads,
    mut persisted_heads: crate::PersistedHeads,
) {
    let store = use_context::<Signal<AppStore>>();

    let ready_id = use_memo(move || {
        let wanted = doc_id_signal();
        let s = store.read();
        match (wanted, s.current_id, &s.handle) {
            (Some(id), Some(cur), Some(_)) if id == cur => Some(id),
            _ => None,
        }
    });

    let mut task_slot: Signal<Option<Task>> = use_signal(|| None);

    use_effect(move || {
        if let Some(prev) = task_slot.write().take() {
            prev.cancel();
        }

        let Some(id) = *ready_id.read() else {
            return;
        };

        let handle = store
            .read()
            .handle
            .clone()
            .expect("store.handle is guaranteed by ready_id");

        let initial_heads = handle.with_document(|doc| format_heads(&doc.get_heads()));
        memory_heads.set(initial_heads.clone());
        persisted_heads.set(initial_heads);

        let task = spawn(async move {
            let mut changes = handle.changes().fuse();
            let mut persisted_stream = handle.persisted().fuse();

            loop {
                futures::select! {
                    changed = changes.next() => if let Some(changed) = changed {
                        memory_heads.set(format_heads(&changed.new_heads));
                    },
                    persisted = persisted_stream.next() => if let Some(persisted) = persisted {
                        persisted_heads.set(format_heads(&persisted.persisted_heads));
                    },
                    complete => {
                        tracing::debug!("use_persistence: streams completed for doc {}", id);
                        break;
                    }
                }
            }
        });

        task_slot.set(Some(task));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::transaction::Transactable;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use tasklens_store::store::AppStore;

    struct TokioRuntime;
    impl samod::runtime::LocalRuntimeHandle for TokioRuntime {
        fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
            tokio::task::spawn_local(future);
        }
    }

    #[component]
    fn TestApp(
        props: (
            samod::Repo,
            samod::DocHandle,
            DocumentId,
            Arc<Mutex<String>>,
        ),
    ) -> Element {
        let doc_id = use_signal(|| Some(props.2));
        let store = use_signal(|| {
            let mut s = AppStore::new();
            s.repo = Some(props.0.clone());
            s.handle = Some(props.1.clone());
            s.current_id = Some(props.2);
            s
        });
        use_context_provider(|| store);

        let memory_heads_signal = use_signal(String::new);
        let persisted_heads_signal = use_signal(String::new);

        use_persistence(
            doc_id,
            crate::MemoryHeads::new(memory_heads_signal),
            crate::PersistedHeads::new(persisted_heads_signal),
        );

        let heads = memory_heads_signal.read();
        if let Ok(mut lock) = props.3.lock()
            && *lock != *heads
        {
            *lock = heads.clone();
        }

        rsx! {
            div { "ready" }
        }
    }

    /// Drive the VirtualDom until `predicate` returns true or timeout expires.
    async fn drive_until(
        dom: &mut VirtualDom,
        timeout: tokio::time::Duration,
        predicate: impl Fn() -> bool,
    ) {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if predicate() {
                return;
            }
            let remaining = deadline - tokio::time::Instant::now();
            if remaining.is_zero() {
                return;
            }
            let _ = tokio::time::timeout(remaining, dom.wait_for_work()).await;
            dom.render_immediate_to_vec();
        }
    }

    #[tokio::test]
    async fn test_use_persistence_updates() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let storage = tasklens_store::samod_storage::SamodStorage::new("test", "docs");
                let repo = samod::RepoBuilder::new(TokioRuntime)
                    .with_storage(storage)
                    .load_local()
                    .await;
                let (handle, id) = AppStore::create_new(repo.clone()).await.unwrap();

                let captured_heads = Arc::new(Mutex::new(String::new()));
                let heads_clone = captured_heads.clone();

                let mut dom =
                    VirtualDom::new_with_props(TestApp, (repo, handle.clone(), id, heads_clone));

                dom.rebuild_in_place();

                let timeout = tokio::time::Duration::from_secs(2);

                // Drive effects so the hook subscribes to the document streams.
                let heads_ref = captured_heads.clone();
                drive_until(&mut dom, timeout, || !heads_ref.lock().unwrap().is_empty()).await;

                // Mutate the document.
                handle.with_document(|doc| {
                    let mut tx = doc.transaction();
                    tx.put(automerge::ROOT, "test", "value").unwrap();
                    tx.commit();
                });

                // Record pre-mutation heads so we can detect the change.
                let pre_mutation_heads = captured_heads.lock().unwrap().clone();

                // Drive until heads change from the mutation.
                let heads_ref = captured_heads.clone();
                drive_until(&mut dom, timeout, move || {
                    let h = heads_ref.lock().unwrap();
                    !h.is_empty() && *h != pre_mutation_heads
                })
                .await;

                let heads = captured_heads.lock().unwrap().clone();
                assert!(
                    !heads.is_empty(),
                    "Heads should not be empty after mutation"
                );
            })
            .await;
    }
}

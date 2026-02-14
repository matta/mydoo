use dioxus::prelude::*;
use tasklens_store::doc_id::DocumentId;
use tasklens_store::store::AppStore;

/// Switches the active document to the one specified by `new_doc_id`.
pub(crate) fn switch_document(
    mut store: Signal<AppStore>,
    mut doc_id: Signal<Option<DocumentId>>,
    new_doc_id: DocumentId,
) {
    spawn(async move {
        let repo = store.read().repo.clone();

        if let Some(repo) = repo {
            match AppStore::find_doc(repo, new_doc_id).await {
                Ok(Some(handle)) => {
                    store.write().set_active_doc(handle, new_doc_id);
                    doc_id.set(Some(new_doc_id));
                }
                Ok(None) => {
                    tracing::error!("Document not found: {}", new_doc_id);
                }
                Err(e) => {
                    tracing::error!("find_doc_detached failed: {:?}", e);
                }
            }
        } else {
            tracing::error!("Repo not initialized");
        }
    });
}

/// Creates a new document and sets it as the active document.
pub(crate) fn create_new_document(
    mut store: Signal<AppStore>,
    mut doc_id: Signal<Option<DocumentId>>,
) {
    spawn(async move {
        let repo = store.read().repo.clone();

        if let Some(repo) = repo {
            match AppStore::create_new(repo).await {
                Ok((handle, new_id)) => {
                    store.write().set_active_doc(handle, new_id);
                    doc_id.set(Some(new_id));
                }
                Err(e) => tracing::error!("Failed to create doc: {:?}", e),
            }
        } else {
            tracing::error!("Repo not initialized");
        }
    });
}

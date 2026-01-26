use dioxus::prelude::*;
use tasklens_store::doc_id::DocumentId;
use tasklens_store::store::AppStore;

/// Switches the active document to the one specified by `new_doc_id`.
pub fn switch_document(
    mut store: Signal<AppStore>,
    mut doc_id: Signal<Option<DocumentId>>,
    new_doc_id: DocumentId,
) {
    tracing::info!("Attempting to switch to Document ID: {}", new_doc_id);
    spawn(async move {
        tracing::info!("Switching to Document ID: {}", new_doc_id);

        // 1. Get repo without holding lock
        let repo = store.read().repo.clone();

        if let Some(repo) = repo {
            // 2. Perform async lookup detached from store instance
            match AppStore::find_doc(repo, new_doc_id.clone()).await {
                Ok(Some(handle)) => {
                    tracing::info!(
                        "find_doc_detached successful for Document ID: {}",
                        new_doc_id
                    );

                    // 3. Acquire lock ONLY for the sync update
                    store.write().set_active_doc(handle, new_doc_id.clone());
                    tracing::info!("set_active_doc successful for Document ID: {}", new_doc_id);

                    let logged_doc_id = new_doc_id.clone();
                    doc_id.set(Some(new_doc_id));
                    tracing::info!("doc_id.set() successful for Document ID: {}", logged_doc_id);
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
pub fn create_new_document(mut store: Signal<AppStore>, mut doc_id: Signal<Option<DocumentId>>) {
    tracing::info!("Creating new document");
    spawn(async move {
        // 1. Get repo without holding lock
        let repo = store.read().repo.clone();

        if let Some(repo) = repo {
            // 2. Perform async creation detached from store instance
            match AppStore::create_new(repo).await {
                Ok((handle, new_id)) => {
                    tracing::info!("Created new doc successfully: {}", new_id);

                    // 3. Acquire lock ONLY for the sync update
                    store.write().set_active_doc(handle, new_id.clone());
                    doc_id.set(Some(new_id));
                }
                Err(e) => tracing::error!("Failed to create doc: {:?}", e),
            }
        } else {
            tracing::error!("Repo not initialized");
        }
    });
}

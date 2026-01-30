use crate::doc_id::DocumentId;
#[cfg(target_arch = "wasm32")]
use crate::doc_id::TaskLensUrl;
#[cfg(target_arch = "wasm32")]
use crate::storage::ActiveDocStorage;
use anyhow::{Result, anyhow};
use tasklens_core::{Action, types::TunnelState};

use crate::adapter;

/// A manager for Automerge documents and persistence.
///
/// This struct implements a Repo-like pattern, managing the current
/// document and providing methods for document lifecycle management.
#[derive(Clone, Debug)]
pub struct AppStore {
    /// The ID of the currently loaded document.
    pub current_id: Option<DocumentId>,
    pub handle: Option<samod::DocHandle>,
    pub repo: Option<samod::Repo>,
}

impl AppStore {
    /// Creates a new AppStore with a fresh document.
    pub fn new() -> Self {
        Self {
            current_id: None,
            handle: None,
            repo: None,
        }
    }

    /// initialize with a specific repo (useful for tests)
    pub fn with_repo(repo: samod::Repo) -> Self {
        Self {
            current_id: None,
            handle: None,
            repo: Some(repo),
        }
    }

    /// Creates a new document using the provided repo.
    pub async fn create_new(repo: samod::Repo) -> Result<(samod::DocHandle, DocumentId)> {
        // Create new document
        let handle = repo.create(automerge::Automerge::new());
        let handle = handle
            .await
            .map_err(|e| anyhow!("Failed to create doc: {:?}", e))?;
        let id = DocumentId::from(handle.document_id());

        // Initialize with default state
        handle.with_document(|doc| {
            if let Err(e) = adapter::init_state(doc, &id) {
                tracing::error!("Failed to initialize state: {}", e);
            }
        });

        Ok((handle, id))
    }

    /// Finds a document using the provided repo.
    pub async fn find_doc(repo: samod::Repo, id: DocumentId) -> Result<Option<samod::DocHandle>> {
        let handle = repo.find(id.into());
        let handle = handle
            .await
            .map_err(|e| anyhow!("Failed to find doc: {:?}", e))?;
        Ok(handle)
    }

    /// Updates the store to track the provided document.
    pub fn set_active_doc(&mut self, handle: samod::DocHandle, id: DocumentId) {
        self.handle = Some(handle);
        self.current_id = Some(id);
        #[cfg(target_arch = "wasm32")]
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(id));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save_active_doc_id(id: DocumentId) {
        tracing::info!("Saving active doc id: {}", id);
        crate::storage::ActiveDocStorage::save_active_url(&TaskLensUrl::from(id));
    }

    /// Imports a document from a byte array.
    /// This is detached from the store instance to avoid holding locks during async operations.
    pub async fn import_doc(
        repo: samod::Repo,
        bytes: Vec<u8>,
    ) -> Result<(samod::DocHandle, DocumentId)> {
        let doc = automerge::Automerge::load(&bytes)?;

        #[cfg(target_arch = "wasm32")]
        {
            // Try to extract existing ID from metadata to preserve identity
            let target_id = adapter::hydrate_tunnel_state(&doc)
                .ok()
                .and_then(|state| state.metadata)
                .and_then(|meta| meta.automerge_url)
                .and_then(|url_str| url_str.parse::<TaskLensUrl>().ok())
                .map(|url| url.document_id);

            if let Some(id) = target_id {
                match Self::inject_existing_doc(repo.clone(), id, &doc).await {
                    Ok((handle, id)) => return Ok((handle, id)),
                    Err(e) => {
                        tracing::error!(
                            "Manual injection failed for ID {}, falling back to new document creation: {:?}",
                            id,
                            e
                        );
                    }
                }
            }
        }

        let handle = repo
            .create(doc)
            .await
            .map_err(|e| anyhow!("Failed to create (import) doc: {:?}", e))?;
        let id = DocumentId::from(handle.document_id());

        #[cfg(target_arch = "wasm32")]
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(id));

        Ok((handle, id))
    }

    /// Exports the current document to a byte array.
    pub fn export_save(&self) -> Vec<u8> {
        if let Some(handle) = &self.handle {
            handle.with_document(|doc| doc.save())
        } else {
            Vec::new()
        }
    }

    /// Reconciles a Rust struct with the current document.
    pub fn expensive_reconcile<T: autosurgeon::Reconcile + 'static>(
        &mut self,
        data: &T,
    ) -> Result<(), autosurgeon::ReconcileError> {
        if let Some(handle) = &mut self.handle {
            handle.with_document(|doc| {
                adapter::expensive_reconcile(doc, data).map_err(|e| match e {
                    adapter::AdapterError::Reconcile(re) => re,
                    _ => autosurgeon::ReconcileError::Automerge(automerge::AutomergeError::Fail),
                })
            })
        } else {
            Err(autosurgeon::ReconcileError::Automerge(
                automerge::AutomergeError::InvalidObjId("root".to_string()),
            ))
        }
    }

    /// Hydrates a Rust struct from the current document.
    pub fn store_hydrate_tunnel_state(&self) -> Result<TunnelState> {
        if let Some(handle) = &self.handle {
            handle.with_document(|doc| adapter::hydrate_tunnel_state(doc).map_err(|e| anyhow!(e)))
        } else {
            Err(anyhow!("No handle available"))
        }
    }

    /// Dispatches an action to modify the application state.
    ///
    /// This method is the primary entry point for state mutations. It implements a
    /// functional core pattern using Autosurgeon's hydration and reconciliation capabilities:
    ///
    /// 1. **Mutate**: The `Action` is applied to the `TunnelState` via specialized handlers.
    /// 2. **Reconcile**: Handlers use surgical reconciliation for efficiency.
    ///    operations, ensuring history and conflict resolution are preserved.
    ///
    /// # Arguments
    ///
    /// * `action` - The `Action` to perform, encapsulating the intent of the mutation
    ///   (e.g., creating a task, updating a field, moving a task).
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the action was successfully applied and reconciled, or an `Error`
    /// if hydration or reconciliation failed.
    pub fn dispatch(&mut self, action: Action) -> Result<()> {
        let handle = self.handle.as_mut().ok_or_else(|| anyhow!("No handle"))?;
        handle
            .with_document(|doc| adapter::dispatch(doc, action))
            .map_err(|e| anyhow!(e))
    }

    /// Static handler for dispatch that works with a handle directly.
    pub fn dispatch_with_handle(handle: &mut samod::DocHandle, action: Action) -> Result<()> {
        handle
            .with_document(|doc| adapter::dispatch(doc, action))
            .map_err(|e| anyhow!(e))
    }

    /// Explicitly inject a document into the underlying WASM storage with a known ID.
    /// This bypasses `repo.create()` which forces a new random ID.
    #[cfg(target_arch = "wasm32")]
    async fn inject_existing_doc(
        repo: samod::Repo,
        id: DocumentId,
        doc: &automerge::Automerge,
    ) -> Result<(samod::DocHandle, DocumentId)> {
        // This code is a hack to work around the fact that we can't create a
        // document with a known ID. TODO: Remove this hack when we figure out
        // how to create a document with a known ID. See also
        // https://github.com/alexjg/samod/issues/60
        //
        // The problem is that `repo.create()` always generates a new random ID,
        // but we need to create a document with a specific ID (the one passed
        // in as an argument).
        //
        // The solution is to manually inject the document into storage and then
        // find it via `repo.find()`.

        use crate::samod_storage::SamodStorage;
        use samod::storage::{LocalStorage, StorageKey};

        let hash = samod_core::CompactionHash::new(&doc.get_heads());
        let key = StorageKey::snapshot_path(&id.into(), &hash);

        let storage = SamodStorage::new("tasklens_samod", "documents");
        storage.put(key, doc.save()).await;

        // Now find it via Repo (which should look in storage)
        match Self::find_doc(repo, id).await {
            Ok(Some(handle)) => Ok((handle, id)),
            Ok(None) => Err(anyhow!(
                "Document {} not found after manual storage injection",
                id
            )),
            Err(e) => Err(anyhow!(
                "Error finding document {} after manual injection: {:?}",
                id,
                e
            )),
        }
    }
}

// Legacy methods removed

impl Default for AppStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
// tests_async uses tokio for its test harness and async runtime (LocalSet),
// which is currently excluded from WASM builds.
#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests_async;

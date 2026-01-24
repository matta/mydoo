pub use crate::actions::{Action, TaskUpdates};
use crate::doc_id::{DocumentId, TaskLensUrl};
#[cfg(target_arch = "wasm32")]
use crate::storage::ActiveDocStorage;
use anyhow::{Result, anyhow};
use tasklens_core::types::TunnelState;

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
        let id = DocumentId::from(handle.document_id().clone());

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
        self.current_id = Some(id.clone());
        #[cfg(target_arch = "wasm32")]
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(id));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save_active_doc_id(id: &DocumentId) {
        crate::storage::ActiveDocStorage::save_active_url(&TaskLensUrl::from(id.clone()));
    }

    /// Imports a document from a byte array.
    /// This is detached from the store instance to avoid holding locks during async operations.
    pub async fn import_doc(
        repo: samod::Repo,
        bytes: Vec<u8>,
    ) -> Result<(samod::DocHandle, DocumentId)> {
        let doc = automerge::Automerge::load(&bytes)?;

        // Try to extract existing ID from metadata to preserve identity
        let target_id = adapter::hydrate::<TunnelState>(&doc)
            .ok()
            .and_then(|state| state.metadata)
            .and_then(|meta| meta.automerge_url)
            .and_then(|url_str| url_str.parse::<TaskLensUrl>().ok())
            .map(|url| url.document_id);

        if let Some(id) = target_id {
            tracing::info!("Importing document with existing ID: {}", id);

            #[cfg(target_arch = "wasm32")]
            {
                use crate::samod_storage::SamodStorage;
                use samod::storage::LocalStorage;

                // Manually inject into storage to bypass Repo::create generating a new ID
                let storage = SamodStorage::new("tasklens_samod", "documents");
                // Samod keys are typically the string representation of the ID
                if let Ok(key) = samod::storage::StorageKey::from_parts(vec![id.to_string()]) {
                    storage.put(key, bytes.clone()).await;

                    // Now find it via Repo (which should look in storage)
                    if let Ok(Some(handle)) = Self::find_doc(repo.clone(), id.clone()).await {
                        return Ok((handle, id));
                    }
                }
            }
        }

        let handle = repo
            .create(doc)
            .await
            .map_err(|e| anyhow!("Failed to create (import) doc: {:?}", e))?;
        let id = DocumentId::from(handle.document_id().clone());

        #[cfg(target_arch = "wasm32")]
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(id.clone()));

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
            handle.with_document(|doc| adapter::expensive_reconcile(doc, data))
        } else {
            Err(autosurgeon::ReconcileError::Automerge(
                automerge::AutomergeError::InvalidObjId("root".to_string()),
            ))
        }
    }

    /// Hydrates a Rust struct from the current document.
    pub fn hydrate<T: autosurgeon::Hydrate + 'static>(&self) -> Result<T> {
        if let Some(handle) = &self.handle {
            handle.with_document(|doc| adapter::hydrate(doc))
        } else {
            Err(anyhow!("No handle available"))
        }
    }

    /// A "total hack" repair utility that fixes tasks with "DoDonee" status,
    /// changing them to "Done". This should be called if hydration fails
    /// because of an "unexpected DoDonee" error.
    pub fn repair_dodonee(&mut self) -> Result<()> {
        let handle = self.handle.as_mut().ok_or_else(|| anyhow!("No handle"))?;
        handle.with_document(adapter::repair_dodonee)
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
        handle.with_document(|doc| adapter::dispatch(doc, action))
    }

    /// Static handler for dispatch that works with a handle directly.
    pub fn dispatch_with_handle(handle: &mut samod::DocHandle, action: Action) -> Result<()> {
        handle.with_document(|doc| adapter::dispatch(doc, action))
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
#[cfg(test)]
mod tests_async;

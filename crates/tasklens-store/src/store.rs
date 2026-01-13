use anyhow::Result;
use autosurgeon::{hydrate, reconcile};
use samod::runtime::RuntimeHandle;
use samod::storage::InMemoryStorage;
use samod::{DocHandle, Repo};
use std::collections::HashMap;
use tasklens_core::types::{ExtraFields, TunnelState};

#[derive(Clone, Debug)]
pub struct AppStore {
    repo: Repo,
    root_handle: Option<DocHandle>,
}

impl AppStore {
    pub async fn new<R: RuntimeHandle>(runtime: R) -> Result<Self> {
        // TODO: Configure IndexedDB storage adapter using samod's features or automerge_repo traits.
        // Currently defaulting to InMemoryStorage as per available documentation.
        let repo = Repo::builder(runtime)
            .with_storage(InMemoryStorage::default())
            .load()
            .await;

        Ok(Self {
            repo,
            root_handle: None,
        })
    }

    /// Initialize the store by creating a new document or loading an existing one.
    /// In a real app, this might accept a DocumentId.
    pub async fn init(&mut self) -> Result<()> {
        let handle = self
            .repo
            .create(automerge::Automerge::new())
            .await
            .map_err(|_| anyhow::anyhow!("Failed to create document"))?;

        // Seed with initial state
        let mut result = Ok(());
        handle.with_document(|doc| {
            let initial_state = TunnelState {
                tasks: HashMap::new(),
                places: HashMap::new(),
                root_task_ids: Vec::new(),
                extra_fields: ExtraFields::default(),
            };
            let mut tx = doc.transaction();
            // We must reconcile against the transaction
            let res = reconcile(&mut tx, &initial_state)
                .map_err(|e| anyhow::anyhow!("Init reconciliation failed: {}", e));
            if res.is_ok() {
                tx.commit();
            }
            result = res;
        });
        result?;

        self.root_handle = Some(handle);
        Ok(())
    }

    pub fn get_state(&self) -> Result<TunnelState> {
        let handle = self
            .root_handle
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AppStore not initialized"))?;

        // Assuming with_document returns the closure result, otherwise we need capture
        let mut state = Err(anyhow::anyhow!("Hydration failed internal"));
        handle.with_document(|doc| {
            state = hydrate(doc).map_err(|e| anyhow::anyhow!("Hydration failed: {}", e));
        });
        state
    }

    pub fn dispatch(&self, _action: ()) {
        // TODO: Implement dispatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::LocalPool;

    #[test]
    fn test_store_init() {
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();

        pool.run_until(async move {
            let mut store = AppStore::new(spawner).await.unwrap();
            store.init().await.unwrap();
            let state = store.get_state().unwrap();
            assert!(state.tasks.is_empty());
            assert!(state.extra_fields.0.is_empty());
        });
    }
}

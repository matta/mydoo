#[cfg(target_arch = "wasm32")]
mod wasm_storage {
    use crate::doc_id::{DocumentId, TaskLensUrl};
    use anyhow::{Result, anyhow};
    use gloo_storage::{LocalStorage, Storage};
    use wasm_bindgen::JsValue;

    const ACTIVE_DOC_STORAGE_KEY: &str = "tasklens_active_doc_id";

    pub struct IndexedDbStorage;

    impl IndexedDbStorage {
        /// Loads the persisted state from the browser's IndexedDB.
        pub async fn load_from_db(doc_id: &DocumentId) -> Result<Option<Vec<u8>>> {
            let key_str = format!("doc:{}:root", doc_id);
            tracing::info!("Storage: Loading from key: {}", key_str);

            let db = Self::build_db().await.map_err(|e| anyhow!("{:?}", e))?;
            let db = scopeguard::guard(db, |db| db.close());

            let transaction = db
                .transaction(&["automerge"], rexie::TransactionMode::ReadOnly)
                .map_err(|e| anyhow!("{:?}", e))?;
            let store = transaction
                .store("automerge")
                .map_err(|e| anyhow!("{:?}", e))?;

            let key = JsValue::from_str(&key_str);
            let val_opt = store.get(key).await.map_err(|e| anyhow!("{:?}", e))?;

            if let Some(val) = val_opt {
                if val.is_undefined() || val.is_null() {
                    tracing::info!("Storage: Value is undefined/null for key: {}", key_str);
                    return Ok(None);
                }
                let bytes: Vec<u8> =
                    serde_wasm_bindgen::from_value(val).map_err(|e| anyhow!("{:?}", e))?;
                tracing::info!("Storage: Loaded {} bytes for key: {}", bytes.len(), key_str);
                Ok(Some(bytes))
            } else {
                tracing::info!("Storage: No value found for key: {}", key_str);
                Ok(None)
            }
        }

        /// Persists the current state to the browser's IndexedDB.
        pub async fn save_to_db(doc_id: &DocumentId, bytes: Vec<u8>) -> Result<()> {
            let key_str = format!("doc:{}:root", doc_id);
            tracing::info!("Storage: Saving {} bytes to key: {}", bytes.len(), key_str);

            let db = Self::build_db().await.map_err(|e| anyhow!("{:?}", e))?;
            let db = scopeguard::guard(db, |db| db.close());

            let transaction = db
                .transaction(&["automerge"], rexie::TransactionMode::ReadWrite)
                .map_err(|e| anyhow!("{:?}", e))?;
            let store = transaction
                .store("automerge")
                .map_err(|e| anyhow!("{:?}", e))?;

            let js_bytes = serde_wasm_bindgen::to_value(&bytes).map_err(|e| anyhow!("{:?}", e))?;

            let key = JsValue::from_str(&key_str);
            store
                .put(&js_bytes, Some(&key))
                .await
                .map_err(|e| anyhow!("{:?}", e))?;

            transaction.done().await.map_err(|e| anyhow!("{:?}", e))?;

            tracing::info!("Storage: Successfully saved to key: {}", key_str);
            Ok(())
        }

        /// Deletes a document from the database.
        pub async fn delete_doc(doc_id: &DocumentId) -> Result<()> {
            let db = Self::build_db().await.map_err(|e| anyhow!("{:?}", e))?;
            let db = scopeguard::guard(db, |db| db.close());

            let transaction = db
                .transaction(&["automerge"], rexie::TransactionMode::ReadWrite)
                .map_err(|e| anyhow!("{:?}", e))?;
            let store = transaction
                .store("automerge")
                .map_err(|e| anyhow!("{:?}", e))?;

            let key = JsValue::from_str(&format!("doc:{}:root", doc_id));
            store.delete(key).await.map_err(|e| anyhow!("{:?}", e))?;

            transaction.done().await.map_err(|e| anyhow!("{:?}", e))?;
            Ok(())
        }

        /// Builds the Rexie database definition.
        async fn build_db() -> Result<rexie::Rexie, rexie::Error> {
            tracing::info!("Building IndexedDB database...");
            let res = rexie::Rexie::builder("tasklens_db")
                .version(1)
                .add_object_store(rexie::ObjectStore::new("automerge"))
                .build()
                .await;
            match &res {
                Ok(_) => tracing::info!("IndexedDB build successful"),
                Err(e) => tracing::error!("IndexedDB build failed: {:?}", e),
            }
            res
        }
    }

    /// Side-channel storage for the active document preference.
    pub struct ActiveDocStorage;

    impl ActiveDocStorage {
        pub fn load_active_url() -> Option<TaskLensUrl> {
            LocalStorage::get::<String>(ACTIVE_DOC_STORAGE_KEY)
                .ok()
                .and_then(|s| s.parse().ok())
        }

        pub fn save_active_url(url: &TaskLensUrl) {
            let _ = LocalStorage::set(ACTIVE_DOC_STORAGE_KEY, url.to_string());
        }

        pub fn clear_active_url() {
            LocalStorage::delete(ACTIVE_DOC_STORAGE_KEY);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_storage::{ActiveDocStorage, IndexedDbStorage};

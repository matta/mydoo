#[cfg(target_arch = "wasm32")]
mod wasm_storage {
    use anyhow::Result;
    use wasm_bindgen::JsValue;

    pub struct IndexedDbStorage;

    impl IndexedDbStorage {
        /// Loads the persisted state from the browser's IndexedDB.
        pub async fn load_from_db() -> Result<Option<Vec<u8>>> {
            let db = Self::build_db()
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
            let transaction = db
                .transaction(&["automerge"], rexie::TransactionMode::ReadOnly)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
            let store = transaction
                .store("automerge")
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let key = JsValue::from_str("root");
            let val_opt = store
                .get(key)
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            if let Some(val) = val_opt {
                if val.is_undefined() || val.is_null() {
                    return Ok(None);
                }
                let bytes: Vec<u8> =
                    serde_wasm_bindgen::from_value(val).map_err(|e| anyhow::anyhow!("{:?}", e))?;
                Ok(Some(bytes))
            } else {
                Ok(None)
            }
        }

        /// Persists the current state to the browser's IndexedDB.
        pub async fn save_to_db(bytes: Vec<u8>) -> Result<()> {
            let db = Self::build_db()
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
            let transaction = db
                .transaction(&["automerge"], rexie::TransactionMode::ReadWrite)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
            let store = transaction
                .store("automerge")
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let js_bytes =
                serde_wasm_bindgen::to_value(&bytes).map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let key = JsValue::from_str("root");
            store
                .put(&js_bytes, Some(&key))
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            transaction
                .done()
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
            Ok(())
        }

        /// Builds the Rexie database definition.
        async fn build_db() -> Result<rexie::Rexie, rexie::Error> {
            rexie::Rexie::builder("tasklens_db")
                .version(1)
                .add_object_store(rexie::ObjectStore::new("automerge"))
                .build()
                .await
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_storage::IndexedDbStorage;

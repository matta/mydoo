#[cfg(target_arch = "wasm32")]
use anyhow::anyhow;
#[cfg(target_arch = "wasm32")]
use rexie::{Rexie, TransactionMode};
use samod::storage::{LocalStorage, StorageKey};
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct SamodStorage {
    db_name: String,
    store_name: String,
}

#[cfg(target_arch = "wasm32")]
impl SamodStorage {
    pub fn new(db_name: &str, store_name: &str) -> Self {
        Self {
            db_name: db_name.to_string(),
            store_name: store_name.to_string(),
        }
    }

    async fn get_db(&self) -> Result<Rexie, anyhow::Error> {
        let res = Rexie::builder(&self.db_name)
            .version(1)
            .add_object_store(rexie::ObjectStore::new(&self.store_name))
            .build()
            .await;

        match res {
            Ok(db) => Ok(db),
            Err(e) => Err(anyhow!("Failed to open DB: {:?}", e)),
        }
    }

    fn key_to_string(key: &StorageKey) -> String {
        // Join components with /
        let parts: Vec<&str> = key.into_iter().map(|s| s.as_str()).collect();
        format!("/{}", parts.join("/"))
    }

    fn string_to_key(s: &str) -> StorageKey {
        // Remove leading /
        let s = s.strip_prefix('/').unwrap_or(s);
        let parts: Vec<&str> = s.split('/').collect();
        StorageKey::from_parts(parts).expect("Invalid key parts generated from string")
    }
}

#[cfg(target_arch = "wasm32")]
impl LocalStorage for SamodStorage {
    async fn load(&self, key: StorageKey) -> Option<Vec<u8>> {
        let key_str = Self::key_to_string(&key);
        let db = match self.get_db().await {
            Ok(db) => db,
            Err(e) => {
                tracing::error!("Storage load error: {:?}", e);
                return None;
            }
        };

        let tx = match db.transaction(&[&self.store_name], TransactionMode::ReadOnly) {
            Ok(tx) => tx,
            Err(e) => {
                tracing::error!("Storage tx error: {:?}", e);
                return None;
            }
        };

        let check_res = async {
            let store = tx.store(&self.store_name).map_err(|e| anyhow!("{:?}", e))?;
            let js_key = JsValue::from_str(&key_str);
            let val = store.get(js_key).await.map_err(|e| anyhow!("{:?}", e))?;
            if val.is_none()
                || val.as_ref().unwrap().is_undefined()
                || val.as_ref().unwrap().is_null()
            {
                return Ok(None);
            }
            let bytes: Vec<u8> =
                serde_wasm_bindgen::from_value(val.unwrap()).map_err(|e| anyhow!("{:?}", e))?;
            Ok::<_, anyhow::Error>(Some(bytes))
        };

        match check_res.await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Storage get error: {:?}", e);
                None
            }
        }
    }

    async fn put(&self, key: StorageKey, data: Vec<u8>) -> () {
        let key_str = Self::key_to_string(&key);
        let db = match self.get_db().await {
            Ok(db) => db,
            Err(e) => {
                tracing::error!("Storage put error: {:?}", e);
                return;
            }
        };
        let db = scopeguard::guard(db, |db| db.close());

        let tx = match db.transaction(&[&self.store_name], TransactionMode::ReadWrite) {
            Ok(tx) => tx,
            Err(e) => {
                tracing::error!("Storage tx error: {:?}", e);
                return;
            }
        };

        let run = async {
            let store = tx.store(&self.store_name).map_err(|e| anyhow!("{:?}", e))?;
            let js_key = JsValue::from_str(&key_str);
            let js_val = serde_wasm_bindgen::to_value(&data).map_err(|e| anyhow!("{:?}", e))?;
            store
                .put(&js_val, Some(&js_key))
                .await
                .map_err(|e| anyhow!("{:?}", e))?;
            tx.done().await.map_err(|e| anyhow!("{:?}", e))?;
            Ok::<(), anyhow::Error>(())
        };

        if let Err(e) = run.await {
            tracing::error!("Storage put operation failed: {:?}", e);
        }
    }

    async fn delete(&self, key: StorageKey) -> () {
        let key_str = Self::key_to_string(&key);
        let db = match self.get_db().await {
            Ok(db) => db,
            Err(e) => {
                tracing::error!("Storage delete error: {:?}", e);
                return;
            }
        };
        let db = scopeguard::guard(db, |db| db.close());

        let tx = match db.transaction(&[&self.store_name], TransactionMode::ReadWrite) {
            Ok(tx) => tx,
            Err(e) => {
                tracing::error!("Storage tx error: {:?}", e);
                return;
            }
        };

        let run = async {
            let store = tx.store(&self.store_name).map_err(|e| anyhow!("{:?}", e))?;
            let js_key = JsValue::from_str(&key_str);
            store.delete(js_key).await.map_err(|e| anyhow!("{:?}", e))?;
            tx.done().await.map_err(|e| anyhow!("{:?}", e))?;
            Ok::<(), anyhow::Error>(())
        };

        if let Err(e) = run.await {
            tracing::error!("Storage delete operation failed: {:?}", e);
        }
    }

    async fn load_range(&self, prefix: StorageKey) -> HashMap<StorageKey, Vec<u8>> {
        let prefix_str = Self::key_to_string(&prefix);
        let mut results = HashMap::new();

        let db = match self.get_db().await {
            Ok(db) => db,
            Err(e) => {
                tracing::error!("Storage load_range error: {:?}", e);
                return results;
            }
        };
        let db = scopeguard::guard(db, |db| db.close());

        let tx = match db.transaction(&[&self.store_name], TransactionMode::ReadOnly) {
            Ok(tx) => tx,
            Err(e) => {
                tracing::error!("Storage tx error: {:?}", e);
                return results;
            }
        };

        let run = async {
            let store = tx.store(&self.store_name).map_err(|e| anyhow!("{:?}", e))?;

            let keys = store
                .get_all_keys(None, None)
                .await
                .map_err(|e| anyhow!("{:?}", e))?;

            for key_val in keys {
                if let Some(s) = key_val.as_string()
                    && s.starts_with(&prefix_str)
                {
                    let val = store.get(key_val).await.map_err(|e| anyhow!("{:?}", e))?;
                    if let Some(v) = val
                        && !v.is_undefined()
                        && !v.is_null()
                    {
                        let bytes: Vec<u8> =
                            serde_wasm_bindgen::from_value(v).map_err(|e| anyhow!("{:?}", e))?;
                        results.insert(Self::string_to_key(&s), bytes);
                    }
                }
            }
            Ok::<_, anyhow::Error>(())
        };

        if let Err(e) = run.await {
            tracing::error!("Storage load_range operation failed: {:?}", e);
        }

        results
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
#[allow(clippy::type_complexity)]
pub struct SamodStorage {
    data: std::sync::Arc<std::sync::Mutex<HashMap<StorageKey, Vec<u8>>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl SamodStorage {
    pub fn new(_db_name: &str, _store_name: &str) -> Self {
        Self {
            data: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn key_to_string(key: &StorageKey) -> String {
        let parts: Vec<&str> = key.into_iter().map(|s| s.as_str()).collect();
        format!("/{}", parts.join("/"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl LocalStorage for SamodStorage {
    async fn load(&self, key: StorageKey) -> Option<Vec<u8>> {
        let data = self.data.lock().unwrap();
        data.get(&key).cloned()
    }

    async fn put(&self, key: StorageKey, val: Vec<u8>) -> () {
        let mut data = self.data.lock().unwrap();
        data.insert(key, val);
    }

    async fn delete(&self, key: StorageKey) -> () {
        let mut data = self.data.lock().unwrap();
        data.remove(&key);
    }

    async fn load_range(&self, prefix: StorageKey) -> HashMap<StorageKey, Vec<u8>> {
        let data = self.data.lock().unwrap();
        let mut results = HashMap::new();
        // Naive implementation, key prefix matching
        // Since StorageKey is a list of strings, we need to check if it "starts with" the prefix parts
        // This is a bit tricky with strict equality, but assuming standard layout:

        for (k, v) in data.iter() {
            // We can convert to string keys for prefix matching if we follow the same convention as wasm
            // Or just implement prefix matching on the vector of strings
            let k_str = Self::key_to_string(k);
            let prefix_str = Self::key_to_string(&prefix);

            if k_str.starts_with(&prefix_str) {
                results.insert(k.clone(), v.clone());
            }
        }
        results
    }
}

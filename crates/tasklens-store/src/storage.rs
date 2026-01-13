use futures::Future;
use samod::storage::{LocalStorage, StorageKey};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
mod wasm_storage {
    use super::*;
    use wasm_bindgen::JsValue;
    use web_sys::console;

    #[derive(Clone, Debug)]
    pub struct IndexedDbStorage {
        db_name: String,
    }

    impl IndexedDbStorage {
        pub fn new(db_name: &str) -> Self {
            Self {
                db_name: db_name.to_string(),
            }
        }

        async fn get_db(&self) -> Result<rexie::Rexie, rexie::Error> {
            rexie::Rexie::builder(&self.db_name)
                .version(1)
                .add_object_store(rexie::ObjectStore::new("samod_storage"))
                .build()
                .await
        }
    }

    impl LocalStorage for IndexedDbStorage {
        fn load(&self, key: StorageKey) -> impl Future<Output = Option<Vec<u8>>> {
            let key_str = key.to_string();
            let this = self.clone();
            async move {
                let db = match this.get_db().await {
                    Ok(db) => db,
                    Err(e) => {
                        console::error_1(&format!("IndexedDB load error (get_db): {:?}", e).into());
                        return None;
                    }
                };
                let tx = match db.transaction(&["samod_storage"], rexie::TransactionMode::ReadOnly)
                {
                    Ok(tx) => tx,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB load error (transaction): {:?}", e).into(),
                        );
                        return None;
                    }
                };
                let store = match tx.store("samod_storage") {
                    Ok(store) => store,
                    Err(e) => {
                        console::error_1(&format!("IndexedDB load error (store): {:?}", e).into());
                        return None;
                    }
                };
                let val = match store.get(JsValue::from_str(&key_str)).await {
                    Ok(v) => v,
                    Err(e) => {
                        console::error_1(&format!("IndexedDB load error (get): {:?}", e).into());
                        return None;
                    }
                };

                if let Some(v) = val {
                    if v.is_undefined() || v.is_null() {
                        return None;
                    }
                    match serde_wasm_bindgen::from_value(v) {
                        Ok(bytes) => Some(bytes),
                        Err(e) => {
                            console::error_1(
                                &format!("IndexedDB load error (deserialization): {:?}", e).into(),
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            }
        }

        fn load_range(
            &self,
            prefix: StorageKey,
        ) -> impl Future<Output = HashMap<StorageKey, Vec<u8>>> {
            let prefix_str = prefix.to_string();
            let this = self.clone();
            async move {
                let mut results = HashMap::new();
                let db = match this.get_db().await {
                    Ok(db) => db,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB load_range error (get_db): {:?}", e).into(),
                        );
                        return results;
                    }
                };
                let tx = match db.transaction(&["samod_storage"], rexie::TransactionMode::ReadOnly)
                {
                    Ok(tx) => tx,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB load_range error (transaction): {:?}", e).into(),
                        );
                        return results;
                    }
                };
                let store = match tx.store("samod_storage") {
                    Ok(store) => store,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB load_range error (store): {:?}", e).into(),
                        );
                        return results;
                    }
                };

                let range = match rexie::KeyRange::bound(
                    &JsValue::from_str(&prefix_str),
                    &JsValue::from_str(&(prefix_str.clone() + "\u{ffff}")),
                    Some(false),
                    Some(false),
                ) {
                    Ok(r) => r,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB load_range error (KeyRange): {:?}", e).into(),
                        );
                        return results;
                    }
                };

                let all = match store.get_all(Some(range.clone()), None).await {
                    Ok(all) => all,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB load_range error (get_all): {:?}", e).into(),
                        );
                        return results;
                    }
                };

                let keys_val = match store.get_all_keys(Some(range), None).await {
                    Ok(k) => k,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB load_range error (get_all_keys): {:?}", e).into(),
                        );
                        return results;
                    }
                };

                for (i, k_js) in keys_val.iter().enumerate() {
                    let Some(v_js) = all.get(i) else { continue };
                    let Ok(k_str) = serde_wasm_bindgen::from_value::<String>(k_js.clone()) else {
                        continue;
                    };
                    let Ok(v_bytes) = serde_wasm_bindgen::from_value::<Vec<u8>>(v_js.clone())
                    else {
                        continue;
                    };

                    let k_vec = k_str.split('/').map(|s| s.to_string()).collect::<Vec<_>>();
                    if let Ok(sk) = StorageKey::from_parts(k_vec) {
                        results.insert(sk, v_bytes);
                    }
                }

                results
            }
        }

        fn put(&self, key: StorageKey, data: Vec<u8>) -> impl Future<Output = ()> {
            let key_str = key.to_string();
            let this = self.clone();
            async move {
                let db = match this.get_db().await {
                    Ok(db) => db,
                    Err(e) => {
                        console::error_1(&format!("IndexedDB put error (get_db): {:?}", e).into());
                        return;
                    }
                };
                let tx = match db.transaction(&["samod_storage"], rexie::TransactionMode::ReadWrite)
                {
                    Ok(tx) => tx,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB put error (transaction): {:?}", e).into(),
                        );
                        return;
                    }
                };
                let store = match tx.store("samod_storage") {
                    Ok(store) => store,
                    Err(e) => {
                        console::error_1(&format!("IndexedDB put error (store): {:?}", e).into());
                        return;
                    }
                };

                let js_data = match serde_wasm_bindgen::to_value(&data) {
                    Ok(d) => d,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB put error (serialization): {:?}", e).into(),
                        );
                        return;
                    }
                };

                if let Err(e) = store
                    .put(&js_data, Some(&JsValue::from_str(&key_str)))
                    .await
                {
                    console::error_1(&format!("IndexedDB put error (put): {:?}", e).into());
                }
                if let Err(e) = tx.done().await {
                    console::error_1(&format!("IndexedDB put error (done): {:?}", e).into());
                }
            }
        }

        fn delete(&self, key: StorageKey) -> impl Future<Output = ()> {
            let key_str = key.to_string();
            let this = self.clone();
            async move {
                let db = match this.get_db().await {
                    Ok(db) => db,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB delete error (get_db): {:?}", e).into(),
                        );
                        return;
                    }
                };
                let tx = match db.transaction(&["samod_storage"], rexie::TransactionMode::ReadWrite)
                {
                    Ok(tx) => tx,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB delete error (transaction): {:?}", e).into(),
                        );
                        return;
                    }
                };
                let store = match tx.store("samod_storage") {
                    Ok(store) => store,
                    Err(e) => {
                        console::error_1(
                            &format!("IndexedDB delete error (store): {:?}", e).into(),
                        );
                        return;
                    }
                };

                if let Err(e) = store.delete(JsValue::from_str(&key_str)).await {
                    console::error_1(&format!("IndexedDB delete error (delete): {:?}", e).into());
                }
                if let Err(e) = tx.done().await {
                    console::error_1(&format!("IndexedDB delete error (done): {:?}", e).into());
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_storage::IndexedDbStorage;

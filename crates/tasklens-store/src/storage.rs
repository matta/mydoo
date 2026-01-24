#[cfg(target_arch = "wasm32")]
mod wasm_storage {
    use crate::doc_id::{DocumentId, TaskLensUrl};
    use anyhow::{Result, anyhow};
    use gloo_storage::{LocalStorage, Storage};
    use wasm_bindgen::JsValue;

    const ACTIVE_DOC_STORAGE_KEY: &str = "tasklens_active_doc_id";

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
pub use wasm_storage::ActiveDocStorage;

#[cfg(not(target_arch = "wasm32"))]
pub struct ActiveDocStorage;

#[cfg(not(target_arch = "wasm32"))]
impl ActiveDocStorage {
    pub fn load_active_url() -> Option<crate::doc_id::TaskLensUrl> {
        None
    }
    pub fn save_active_url(_url: &crate::doc_id::TaskLensUrl) {}
    pub fn clear_active_url() {}
}

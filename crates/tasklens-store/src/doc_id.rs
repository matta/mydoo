//! Document ID management for multi-document support.
//!
//! This module provides utilities for generating, storing, and retrieving document IDs.
//! Unlike the React implementation which uses automerge-repo URLs, the Rust implementation
//! uses cryptographically random document IDs that:
//! - Identify the local Automerge document in IndexedDB
//! - When combined with the master key, determine the sync channel

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_imports))]
use anyhow::{Result, anyhow};
use rand::RngCore;

/// Key used for storing the document ID in LocalStorage.
#[allow(dead_code)] // Used in WASM-only functions
const DOC_ID_STORAGE_KEY: &str = "tasklens_doc_id";

/// Generates a new cryptographically random document ID.
///
/// # Returns
///
/// A 64-character hex string (32 bytes of randomness).
///
/// # Example
///
/// ```rust,ignore
/// let doc_id = tasklens_store::doc_id::generate_doc_id();
/// println!("New document ID: {}", doc_id);
/// // Output: "a1b2c3d4e5f6..."
/// ```
#[allow(dead_code)]
pub fn generate_doc_id() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Saves the document ID to LocalStorage.
///
/// # Arguments
///
/// * `doc_id` - The document ID to persist.
///
/// # Returns
///
/// * `Ok(())` on success.
/// * `Err` if storage fails.
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
pub fn save_doc_id(doc_id: &str) -> Result<()> {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::set(DOC_ID_STORAGE_KEY, doc_id)
        .map_err(|e| anyhow!("Failed to save document ID: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn save_doc_id(_doc_id: &str) -> Result<()> {
    tracing::warn!("save_doc_id is not supported on non-wasm targets");
    Ok(())
}

/// Loads the document ID from LocalStorage.
///
/// # Returns
///
/// * `Ok(Some(String))` - If a document ID exists.
/// * `Ok(None)` - If no document ID is found (first run).
/// * `Err` - If storage access fails.
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
pub fn load_doc_id() -> Result<Option<String>> {
    use gloo_storage::{LocalStorage, Storage};
    match LocalStorage::get::<String>(DOC_ID_STORAGE_KEY) {
        Ok(doc_id) => Ok(Some(doc_id)),
        Err(gloo_storage::errors::StorageError::KeyNotFound(_)) => Ok(None),
        Err(e) => Err(anyhow!("Failed to load document ID: {}", e)),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn load_doc_id() -> Result<Option<String>> {
    Ok(None)
}

/// Clears the document ID from LocalStorage.
///
/// Call this when creating a new document or resetting the application.
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
pub fn clear_doc_id() {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::delete(DOC_ID_STORAGE_KEY);
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn clear_doc_id() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_doc_id() {
        let id1 = generate_doc_id();
        let id2 = generate_doc_id();

        // Should be 64 hex characters (32 bytes)
        assert_eq!(id1.len(), 64);
        assert_eq!(id2.len(), 64);

        // Should be different
        assert_ne!(id1, id2);

        // Should be valid hex
        assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(id2.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

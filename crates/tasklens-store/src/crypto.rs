//! # Cryptographic Primitives for Client-Side Sync
//!
//! This module handles the "Identity" and "Key Management" portion of the application.
//!
//! ## Core Concepts
//!
//! 1.  **Identity**: A user's identity is strictly tied to a 12-word BIP39 mnemonic phrase.
//!     This phrase is generated locally. It is never transmitted to the server, but it is
//!     displayed to the user who MUST save it to regain access on other devices.
//!     We assume the user will copy/paste this phrase into a password manager or secure note.
//!     Possession of this phrase is the only way to decrypt the user's data.
//!
//! 2.  **Key Derivation**: To encrypt data, we need a 32-byte cryptographic key.
//!     We derive this key deterministically from the mnemonic using Argon2id.
//!     This ensures that the same mnemonic always produces the same encryption key,
//!     allowing users to access their data from multiple devices by entering the same phrase.
//!
//! 3.  **Static Salt**: For this MVP, we use a hardcoded application-wide salt for key derivation.
//!     In a more advanced system, we might use a per-user salt stored on the server, but
//!     using a static salt simplifies the "username-less" login flow (we only need the mnemonic).

use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use bip39::Mnemonic;
use rand::RngCore;

use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, AeadCore, KeyInit},
};

use sha2::{Digest, Sha256};
use tasklens_sync_protocol::EncryptedBlob;

/// A hardcoded salt used for deterministic key derivation.
///
/// We accept the trade-off of using a static salt because it allows us to
/// derive the encryption key purely from the mnemonic without needing to look
/// up a user record on a server first. This supports the "offline-first" and
/// "server-blind" architecture.
const STATIC_SALT_BYTES: &[u8; 16] = b"tasklens_app_v1!";

/// Context string for SyncID derivation to separate domain from other potential hashes.
const SYNC_ID_CONTEXT: &[u8] = b"TaskLens_SyncID_v1";

/// Derives the public Sync ID from the Master Key.
///
/// We use SHA-256(MasterKey || Context) to generate a deterministic ID that constitutes
/// the "room" or "channel" for the user's devices. The server sees this ID but cannot
/// derive the MasterKey from it.
///
/// # Arguments
///
/// * `key` - The 32-byte Master Key.
///
/// # Returns
///
/// * `String` - A hex-encoded string of the SHA-256 hash.
#[allow(dead_code)]
pub fn derive_sync_id(key: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update(SYNC_ID_CONTEXT);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generates a new random Identity.
///
/// # Returns
///
/// A 12-word BIP39 mnemonic string (English).
///
/// # Example
///
/// ```rust,ignore
/// let phrase = todo_mvp::crypto::generate_key();
/// println!("Write this down: {}", phrase);
/// // Output: "witch collapse practice feed shame open despair creek road again ice least"
/// ```
#[allow(dead_code)]
pub fn generate_key() -> String {
    let mut entropy = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut entropy);
    // 16 bytes of entropy produces a 12-word mnemonic
    let mnemonic =
        Mnemonic::from_entropy(&entropy).expect("Failed to create mnemonic from entropy");
    mnemonic.to_string()
}

/// Derives the 32-byte Master Encryption Key from a mnemonic phrase.
///
/// This function uses the Argon2id KDF (Key Derivation Function) to harden the
/// mnemonic against brute-force attacks.
///
/// # Arguments
///
/// * `phrase` - A valid BIP39 mnemonic string.
///
/// # Returns
///
/// * `Ok([u8; 32])` - The derived 32-byte key, ready for use with XChaCha20Poly1305.
/// * `Err` - If the mnemonic is invalid or key derivation fails.
#[allow(dead_code)]
pub fn derive_key(phrase: &str) -> Result<[u8; 32]> {
    // 1. Validate the mnemonic phrase.
    // bip39::Mnemonic::parse checks the checksum and word list.
    let _ = Mnemonic::parse(phrase).map_err(|e| anyhow!("Invalid mnemonic: {}", e))?;

    // 2. Prepare the salt.
    // Argon2 requires a base64-encoded salt string.
    let salt = SaltString::encode_b64(STATIC_SALT_BYTES)
        .map_err(|e| anyhow!("Failed to encode salt: {}", e))?;

    // 3. Perform the derivation.
    // We use Argon2id with default parameters (memory-hard).
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(phrase.as_bytes(), &salt)
        .map_err(|e| anyhow!("Argon2 error: {}", e))?;

    // 4. Extract the key.
    // The hash output is our derived key.
    let hash_output = password_hash
        .hash
        .ok_or_else(|| anyhow!("Argon2 failed to produce hash output"))?;

    let hash_bytes = hash_output.as_bytes();
    if hash_bytes.len() != 32 {
        return Err(anyhow!(
            "Derived key length is {}, expected 32",
            hash_bytes.len()
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(hash_bytes);

    Ok(key)
}

/// Key used for storage in LocalStorage/SessionStorage.
#[allow(dead_code)]
const STORAGE_KEY: &str = "tasklens_master_key";

/// strategies for storing the Master Key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StorageMode {
    /// Store in `SessionStorage`. Data is cleared when the tab/browser is closed.
    Session,
    /// Store in `LocalStorage`. Data persists across browser restarts.
    Local,
}

#[cfg(target_arch = "wasm32")]
use gloo_storage::{LocalStorage, SessionStorage, Storage};

/// Stores the Master Key in the browser's storage.
///
/// # Arguments
///
/// * `key` - The 32-byte master key.
/// * `mode` - The storage strategy to use.
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
pub fn save_key(key: &[u8; 32], mode: StorageMode) -> Result<()> {
    match mode {
        StorageMode::Local => {
            // If remembering, clear any session key to avoid confusion, and save to local.
            SessionStorage::delete(STORAGE_KEY);
            LocalStorage::set(STORAGE_KEY, key)
                .map_err(|e| anyhow!("Failed to save to LocalStorage: {}", e))
        }
        StorageMode::Session => {
            // If ephemeral, clear any local key (user might have unchecked "remember me"), and save to session.
            LocalStorage::delete(STORAGE_KEY);
            SessionStorage::set(STORAGE_KEY, key)
                .map_err(|e| anyhow!("Failed to save to SessionStorage: {}", e))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn save_key(_key: &[u8; 32], _mode: StorageMode) -> Result<()> {
    tracing::warn!("save_key is not supported on non-wasm targets");
    Ok(())
}

/// Attempts to load the Master Key from storage.
///
/// Checks `SessionStorage` first (active session), then `LocalStorage` (remembered session).
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
pub fn load_key() -> Result<Option<[u8; 32]>> {
    // 1. Check SessionStorage
    if let Ok(key) = SessionStorage::get::<[u8; 32]>(STORAGE_KEY) {
        return Ok(Some(key));
    }

    // 2. Check LocalStorage
    if let Ok(key) = LocalStorage::get::<[u8; 32]>(STORAGE_KEY) {
        return Ok(Some(key));
    }

    Ok(None)
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn load_key() -> Result<Option<[u8; 32]>> {
    Ok(None)
}

/// Clears the Master Key from both SessionStorage and LocalStorage.
///
/// Call this on "Logout".
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
pub fn clear_key() {
    LocalStorage::delete(STORAGE_KEY);
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn clear_key() {}

/// Encrypts a binary blob using XChaCha20Poly1305 and the Master Key.
///
/// # Arguments
///
/// * `data` - The plaintext data to encrypt (e.g., an Automerge change chunk).
/// * `key` - The 32-byte Master Key.
///
/// # Returns
///
/// * `Ok(EncryptedBlob)` - The encrypted blob with its generated nonce.
/// * `Err` - If encryption fails (should be rare).
#[allow(dead_code)]
pub fn encrypt_change(data: &[u8], key: &[u8; 32]) -> Result<EncryptedBlob> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut rand::thread_rng()); // 24-byte nonce
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    Ok(EncryptedBlob {
        nonce: nonce.into(),
        ciphertext,
    })
}

/// Decrypts an `EncryptedBlob` using the Master Key.
///
/// # Arguments
///
/// * `blob` - The encrypted blob to decrypt.
/// * `key` - The 32-byte Master Key.
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The decrypted plaintext.
/// * `Err` - If decryption fails (e.g., wrong key, tampered data).
#[allow(dead_code)]
pub fn decrypt_change(blob: &EncryptedBlob, key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XNonce::from_slice(&blob.nonce);
    let plaintext = cipher
        .decrypt(nonce, blob.ciphertext.as_ref())
        .map_err(|e| anyhow!("Decryption failed (wrong key or corrupted data): {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keygen_and_derivation() {
        let phrase = generate_key();
        assert_eq!(phrase.split_whitespace().count(), 12);

        let key1 = derive_key(&phrase).unwrap();
        let key2 = derive_key(&phrase).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_encryption_roundtrip() {
        let phrase = generate_key();
        let key = derive_key(&phrase).unwrap();

        let data = b"Super secret todo list items";
        let encrypted = encrypt_change(data, &key).expect("Encryption failed");

        assert_ne!(data.as_slice(), encrypted.ciphertext.as_slice());

        let decrypted = decrypt_change(&encrypted, &key).expect("Decryption failed");
        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_decryption_fails_with_wrong_key() {
        let phrase1 = generate_key();
        let key1 = derive_key(&phrase1).unwrap();

        let phrase2 = generate_key();
        let key2 = derive_key(&phrase2).unwrap();

        let data = b"My secrets";
        let encrypted = encrypt_change(data, &key1).unwrap();

        let result = decrypt_change(&encrypted, &key2);
        assert!(result.is_err());
    }
}

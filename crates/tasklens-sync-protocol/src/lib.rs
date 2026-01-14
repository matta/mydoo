use serde::{Deserialize, Serialize};

/// A container for encrypted data, including the unique nonce used for encryption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    /// The 24-byte nonce (number used once) for XChaCha20Poly1305.
    /// We use XChaCha20 because it accepts a large random nonce, removing the need
    /// for nonce counting or state tracking, which is perfect for our syncing model.
    pub nonce: [u8; 24],
    /// The encrypted data (ciphertext + authentication tag).
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Handshake to initiate syncing.
    Hello {
        /// Random UUID identifying this device/session.
        client_id: String,
        /// Public ID derived from the MasterKey (SHA-256).
        /// Acts as the "Room ID" to group devices.
        sync_id: String,
        /// The last sequence ID this client has seen.
        /// 0 if this is a fresh sync.
        last_sequence: i64,
    },
    /// Push a new change to the server.
    SubmitChange {
        /// The room to push to (redundant but explicit).
        sync_id: String,
        /// The encrypted Automerge change.
        payload: EncryptedBlob,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Broadcast a new change to other clients.
    ChangeOccurred {
        /// Monotonic sequence ID from the server DB.
        sequence_id: i64,
        /// The room this change belongs to.
        sync_id: String,
        /// The client who originated this change (to avoid echo).
        source_client_id: String,
        /// The encrypted content.
        payload: EncryptedBlob,
    },
}

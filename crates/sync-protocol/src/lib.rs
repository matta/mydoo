use serde::{Deserialize, Serialize};

/// Encrypted payload for sync messages.
/// Uses XChaCha20-Poly1305 encryption with a 24-byte nonce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    /// XChaCha20 uses a 24-byte nonce
    pub nonce: [u8; 24],
    /// Encrypted payload
    pub ciphertext: Vec<u8>,
}

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Initial handshake from client
    #[serde(rename = "hello")]
    Hello {
        client_id: String,
        sync_id: String,
        last_sequence: i64,
    },
    /// Submit a change to the server
    #[serde(rename = "submit_change")]
    SubmitChange {
        sync_id: String,
        payload: EncryptedBlob,
    },
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Notify client of a change
    #[serde(rename = "change_occurred")]
    ChangeOccurred {
        sequence_id: i64,
        sync_id: String,
        source_client_id: String,
        payload: EncryptedBlob,
    },
}

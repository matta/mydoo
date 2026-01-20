use serde::{Deserialize, Serialize};

/// A container for encrypted data.
/// NOTE: This struct is no longer used directly in the protocol (which uses generic Vec<u8>),
/// but is kept here for reference or client-side usage if needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Handshake to initiate syncing.
    Hello {
        /// Random UUID identifying this device/session.
        client_id: String,
        /// Public ID used to discover/route to the document.
        /// Derived from the Document Secret (e.g. SHA-256).
        discovery_key: String,
        /// The last sequence ID this client has seen.
        /// 0 if this is a fresh sync.
        last_sequence: i64,
    },
    /// Push a new change to the server.
    SubmitChange {
        /// The channel to push to (redundant but explicit).
        discovery_key: String,
        /// The encrypted content (opaque bytes).
        payload: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Broadcast a new change to other clients.
    ChangeOccurred {
        /// Monotonic sequence ID from the server DB.
        sequence_id: i64,
        /// The channel this change belongs to.
        discovery_key: String,
        /// The client who originated this change (to avoid echo).
        source_client_id: String,
        /// The encrypted content (opaque bytes).
        payload: Vec<u8>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_schema_update() {
        let msg = ClientMessage::Hello {
            client_id: "test-client".to_string(),
            discovery_key: "test-key".to_string(),
            last_sequence: 10,
        };
        let json = serde_json::to_string(&msg).unwrap();

        assert!(
            json.contains("discovery_key"),
            "JSON output should contain 'discovery_key', got: {}",
            json
        );
    }
}

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Unique identifier for a TaskLens document.
///
/// Wraps a UUID and serializes to a Base58Check string.
/// Bit-identical format to the Automerge Repo (samod) DocumentId.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentId(Uuid);

impl DocumentId {
    /// Generates a new random document ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_string = bs58::encode(self.0.as_bytes()).with_check().into_string();
        write!(f, "{}", as_string)
    }
}

impl fmt::Debug for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocumentId({})", self)
    }
}

impl FromStr for DocumentId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = bs58::decode(s)
            .with_check(None)
            .into_vec()
            .map_err(|e| anyhow!("Invalid Base58Check: {}", e))?;

        let uuid = Uuid::from_slice(&bytes).map_err(|e| anyhow!("Invalid UUID: {}", e))?;
        Ok(Self(uuid))
    }
}

/// A user-facing locator for a TaskLens document.
///
/// Encapsulates the "tasklens:{DocumentId}" format.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskLensUrl {
    pub document_id: DocumentId,
}

impl fmt::Display for TaskLensUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tasklens:{}", self.document_id)
    }
}

impl fmt::Debug for TaskLensUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TaskLensUrl({})", self)
    }
}

impl FromStr for TaskLensUrl {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let suffix = s
            .strip_prefix("tasklens:")
            .ok_or_else(|| anyhow!("Invalid TaskLens URL: missing 'tasklens:' prefix"))?;

        let document_id = DocumentId::from_str(suffix)?;
        Ok(Self { document_id })
    }
}

impl From<DocumentId> for TaskLensUrl {
    fn from(document_id: DocumentId) -> Self {
        Self { document_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_id_serialization() {
        let id = DocumentId::new();
        let s = id.to_string();
        let id2 = DocumentId::from_str(&s).unwrap();
        assert_eq!(id, id2);

        // Verify it looks like Base58 (no '0', 'O', 'I', 'l')
        assert!(!s.contains('0'));
        assert!(!s.contains('O'));
        assert!(!s.contains('I'));
        assert!(!s.contains('l'));
    }

    #[test]
    fn test_tasklens_url_parsing() {
        let id = DocumentId::new();
        let url = TaskLensUrl::from(id.clone());
        let s = url.to_string();
        assert!(s.starts_with("tasklens:"));

        let url2 = TaskLensUrl::from_str(&s).unwrap();
        assert_eq!(url.document_id, url2.document_id);
    }

    #[test]
    fn test_invalid_id() {
        assert!(DocumentId::from_str("invalid").is_err());
        assert!(DocumentId::from_str("123456789ABCDEF").is_err()); // Wrong length/checksum
    }
}

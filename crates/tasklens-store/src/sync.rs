#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl SyncStatus {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Error(_))
    }

    pub fn is_connecting(&self) -> bool {
        matches!(self, Self::Connecting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_is_connected() {
        assert!(SyncStatus::Connected.is_connected());
        assert!(!SyncStatus::Connecting.is_connected());
        assert!(!SyncStatus::Disconnected.is_connected());
        assert!(!SyncStatus::Error("foo".to_string()).is_connected());
    }

    #[test]
    fn test_sync_status_is_disconnected() {
        assert!(SyncStatus::Disconnected.is_disconnected());
        assert!(SyncStatus::Error("foo".to_string()).is_disconnected());
        assert!(!SyncStatus::Connected.is_disconnected());
        assert!(!SyncStatus::Connecting.is_disconnected());
    }

    #[test]
    fn test_sync_status_is_connecting() {
        assert!(SyncStatus::Connecting.is_connecting());
        assert!(!SyncStatus::Connected.is_connecting());
        assert!(!SyncStatus::Disconnected.is_connecting());
        assert!(!SyncStatus::Error("foo".to_string()).is_connecting());
    }
}

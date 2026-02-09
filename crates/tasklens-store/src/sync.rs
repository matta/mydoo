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

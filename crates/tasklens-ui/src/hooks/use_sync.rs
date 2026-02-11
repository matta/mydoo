use dioxus::prelude::*;
use tasklens_store::store::AppStore;
pub use tasklens_store::sync::SyncStatus;

#[cfg(target_arch = "wasm32")]
pub(crate) const SYNC_SERVER_URL_KEY: &str = "tasklens_sync_server_url";

#[cfg(any(target_arch = "wasm32", test))]
const INITIAL_RETRY_DELAY_MS: u32 = 1_000;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_RETRY_DELAY_MS: u32 = 30_000;

#[cfg(target_arch = "wasm32")]
#[path = "use_sync/wasm.rs"]
mod platform;
#[cfg(not(target_arch = "wasm32"))]
#[path = "use_sync/desktop.rs"]
mod platform;

/// Build and run the sync client for the active target platform.
pub fn use_sync_client(store: Signal<AppStore>) -> Signal<SyncStatus> {
    platform::use_sync_client_impl(store)
}

/// Calculate the next reconnect delay for exponential backoff.
#[cfg(any(target_arch = "wasm32", test))]
fn next_retry_delay_ms(current_delay_ms: u32) -> u32 {
    current_delay_ms.saturating_mul(2).min(MAX_RETRY_DELAY_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_retry_delay_doubles_before_cap() {
        assert_eq!(next_retry_delay_ms(INITIAL_RETRY_DELAY_MS), 2_000);
        assert_eq!(next_retry_delay_ms(8_000), 16_000);
    }

    #[test]
    fn next_retry_delay_clamps_at_cap() {
        assert_eq!(next_retry_delay_ms(20_000), MAX_RETRY_DELAY_MS);
        assert_eq!(next_retry_delay_ms(MAX_RETRY_DELAY_MS), MAX_RETRY_DELAY_MS);
    }

    #[test]
    fn next_retry_delay_handles_overflow_inputs() {
        assert_eq!(next_retry_delay_ms(u32::MAX), MAX_RETRY_DELAY_MS);
    }
}

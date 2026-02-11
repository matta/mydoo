use super::SyncStatus;
use dioxus::prelude::*;
use tasklens_store::store::AppStore;

/// Build the desktop sync hook shell until desktop transport is implemented.
pub(super) fn use_sync_client_impl(_store: Signal<AppStore>) -> Signal<SyncStatus> {
    let status = use_signal(|| SyncStatus::Disconnected);

    use_future(move || async move {
        tracing::warn!("Sync client is not yet implemented for desktop targets.");
    });

    status
}

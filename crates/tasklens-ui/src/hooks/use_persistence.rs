use dioxus::prelude::*;
use tasklens_store::store::AppStore;

pub fn use_persistence(_store: Signal<AppStore>) {
    // Legacy persistence logic removed.
    // Samod handles persistence automatically via the Storage adapter in Repo.

    // TODO: Re-implement MemoryHeads and PersistedHeads updates if needed for E2E tests,
    // by observing Repo/DocHandle changes.
}

use dioxus::prelude::*;
use tasklens_core::types::TunnelState;
use tasklens_store::store::AppStore;

/// A hook that centralizes hydration of TunnelState from the AppStore.
/// It handles error logging and surfaces errors via the `load_error` signal context.
#[tracing::instrument(skip_all)]
pub fn use_tunnel_state() -> Memo<TunnelState> {
    if let Some(state) = try_use_context::<Memo<TunnelState>>() {
        return state;
    }

    let store = use_context::<Signal<AppStore>>();
    let load_error = use_context::<Signal<Option<String>>>();
    let memory_heads = use_context::<crate::MemoryHeads>();

    use_memo(move || compute_tunnel_state(store, load_error, memory_heads))
}

#[tracing::instrument(skip_all)]
fn compute_tunnel_state(
    store: Signal<AppStore>,
    mut load_error: Signal<Option<String>>,
    memory_heads: crate::MemoryHeads,
) -> TunnelState {
    // Subscribe to heads updates to trigger re-hydration when the document changes
    let heads = memory_heads.read();
    info!("compute_tunnel_state Heads: {}", *heads);

    let state = store.read().store_hydrate_tunnel_state();

    match state {
        Ok(state) => {
            // Clear any previous load errors if hydration succeeds
            if load_error.read().is_some() {
                load_error.set(None);
            }
            state
        }
        Err(e) => {
            let err_msg = e.to_string();
            tracing::error!("Failed to hydrate TunnelState: {}", err_msg);
            load_error.set(Some(err_msg));
            TunnelState::default()
        }
    }
}

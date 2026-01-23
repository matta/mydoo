use dioxus::prelude::*;
use tasklens_core::types::TunnelState;
use tasklens_store::store::AppStore;

/// A hook that centralizes hydration of TunnelState from the AppStore.
/// It handles error logging and surfaces errors via the `load_error` signal context.
pub fn use_tunnel_state() -> Memo<TunnelState> {
    let store = use_context::<Signal<AppStore>>();
    let mut load_error = use_context::<Signal<Option<String>>>();
    let memory_heads = use_context::<crate::MemoryHeads>();

    use_memo(move || {
        // Subscribe to heads updates to trigger re-hydration when the document changes
        let _ = memory_heads.0.read();

        match store.read().hydrate::<TunnelState>() {
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
    })
}

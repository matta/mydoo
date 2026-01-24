use dioxus::prelude::*;
use tasklens_core::types::TunnelState;
use tasklens_store::store::AppStore;

/// A hook that centralizes hydration of TunnelState from the AppStore.
/// It handles error logging and surfaces errors via the `load_error` signal context.
pub fn use_tunnel_state() -> Memo<TunnelState> {
    let mut store = use_context::<Signal<AppStore>>();
    let mut load_error = use_context::<Signal<Option<String>>>();
    let memory_heads = use_context::<crate::MemoryHeads>();

    use_memo(move || {
        // Subscribe to heads updates to trigger re-hydration when the document changes
        let _ = memory_heads.0.read();

        let initial_hydrate = store.read().hydrate::<TunnelState>();

        match initial_hydrate {
            Ok(state) => {
                // Clear any previous load errors if hydration succeeds
                if load_error.read().is_some() {
                    load_error.set(None);
                }
                state
            }
            Err(e) if e.to_string().contains("unexpected DoDonee") => {
                tracing::warn!(
                    "Total hack: Hydration failed with 'DoDonee'. Triggering automatic repair."
                );
                let mut s = store.write();
                if let Err(err) = s.repair_dodonee() {
                    tracing::error!("Automatic repair failed: {}", err);
                }

                // Re-hydrate after repair
                match s.hydrate::<TunnelState>() {
                    Ok(state) => {
                        if load_error.read().is_some() {
                            load_error.set(None);
                        }
                        state
                    }
                    Err(retry_e) => {
                        let err_msg = format!("Re-hydration failed after repair: {}", retry_e);
                        tracing::error!("{}", err_msg);
                        load_error.set(Some(err_msg));
                        TunnelState::default()
                    }
                }
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

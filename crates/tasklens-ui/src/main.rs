use dioxus::prelude::*;
use tasklens_core::types::TunnelState;
use tasklens_store::store::AppStore;

fn main() {
    // Initialize panic hook for better WASM error messages
    console_error_panic_hook::set_once();

    // Initialize the tracing subscriber to capture logs in the browser console.
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut state_sig = use_signal(TunnelState::default);
    let mut store_ready = use_signal(|| false);

    // Initialize store
    // NOTE: Reactive updates via subscribe will be implemented in Milestone 2.5
    // when we add the sync service. For now, we just load the initial state.
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        tracing::info!("Initializing AppStore...");

        let mut store = AppStore::new();
        if let Err(e) = store.init() {
            tracing::error!("Failed to init store: {}", e);
            return;
        }

        tracing::info!("AppStore initialized");

        // Load initial state
        match store.get_state() {
            Ok(state) => {
                state_sig.set(state);
                store_ready.set(true);
                tracing::info!("Initial state loaded");
            }
            Err(e) => {
                tracing::error!("Failed to get state: {}", e);
            }
        }
    });

    rsx! {
        div {
            h1 { "TaskLens Migration" }
            if !store_ready() {
                p { "Loading Store..." }
            } else {
                div {
                    p { "Tasks: {state_sig.read().tasks.len()}" }
                    // Debug button: Dispatch logic is currently out of scope as AppStore is not yet
                    // exposed via context/signal. Milestone 2.2 focuses on reactive state updates.
                    // For now, this button simply logs to verify interaction.
                    button { onclick: |_| tracing::info!("Button clicked"), "Log Click" }
                }
            }
        }
    }
}

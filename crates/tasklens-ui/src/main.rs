use dioxus::prelude::*;
use futures::StreamExt;
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

    // Initialize store and subscribe in one coroutine
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        tracing::info!("Initializing AppStore...");

        #[cfg(target_arch = "wasm32")]
        let store_result = AppStore::new().await;

        #[cfg(not(target_arch = "wasm32"))]
        let store_result: anyhow::Result<AppStore> = {
            tracing::error!("AppStore not supported on non-WASM in this UI yet");
            Err(anyhow::anyhow!("Not supported on non-WASM"))
        };

        match store_result {
            Ok(mut store) => {
                if let Err(e) = store.init().await {
                    tracing::error!("Failed to init store: {}", e);
                    return;
                }

                tracing::info!("AppStore initialized, subscribing...");
                store_ready.set(true);

                let stream = store.subscribe();
                let mut stream = Box::pin(stream);
                while let Some(new_state) = stream.next().await {
                    state_sig.set(new_state);
                    tracing::info!("State updated");
                }
            }
            Err(e) => {
                tracing::error!("Failed to create store: {}", e);
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

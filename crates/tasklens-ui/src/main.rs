#![allow(non_snake_case)]

// Include the generated build version from build.rs
include!(concat!(env!("OUT_DIR"), "/build_version.rs"));

use dioxus::prelude::*;
use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;

mod components;
mod controllers;
mod router;
mod seed;
mod views;

use crate::router::Route;
use tasklens_store::crypto;
use tasklens_store::network;
use tasklens_store::store::AppStore;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = registerServiceWorker, catch)]
    fn register_service_worker() -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(js_name = subscribeToServiceWorkerStatus, catch)]
    fn subscribe_to_service_worker_status(
        callback: &Closure<dyn FnMut(bool)>,
    ) -> Result<(), JsValue>;
}

fn main() {
    // Initialize the tracing subscriber to capture logs in the browser console.
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    // Ensure Rust panics are logged to the browser console for debugging.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    init_service_worker();

    dioxus::launch(App);
}

fn App() -> Element {
    // Session state
    let mut master_key = use_signal(|| None::<[u8; 32]>);
    let mut is_checking = use_signal(|| true);
    // Mutated inside #[cfg(target_arch = "wasm32")] block below.
    #[allow(unused_mut)]
    let mut service_worker_active = use_signal(|| false);

    // Store State
    let mut store = use_signal(AppStore::new);

    // Provide context
    use_context_provider(|| master_key);
    use_context_provider(|| service_worker_active);
    use_context_provider(|| store);

    // The master key is required for all encrypted storage operations.
    // We check local storage asynchronously on startup.
    use_future(move || async move {
        match crypto::load_key() {
            Ok(Some(key)) => {
                tracing::info!("Loaded key from storage");
                master_key.set(Some(key));
            }
            Ok(None) => {
                tracing::info!("No key found in storage");
            }
            Err(e) => {
                tracing::error!("Error loading key: {:?}", e);
            }
        }
        is_checking.set(false);

        // Check for seed flag
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let search = window.location().search().unwrap_or_default();
            if search.contains("seed=true") {
                crate::seed::prime_store_with_sample_data(&mut store.write()).await;
            }
        }
    });

    // Load store from DB on startup
    use_future(move || async move {
        match AppStore::load_from_db().await {
            Ok(Some(bytes)) => {
                tracing::info!("Loaded {} bytes from storage", bytes.len());
                store.write().load_from_bytes(bytes);
            }
            Ok(None) => {
                tracing::info!("No persisted data found; initializing new store");
                if let Err(e) = store.write().init() {
                    tracing::error!("Failed to initialize store: {:?}", e);
                }
            }
            Err(e) => tracing::error!("Failed to load from storage: {:?}", e),
        }
    });

    // Sync Service Task
    let sync_task = use_coroutine(move |rx_local: UnboundedReceiver<Vec<u8>>| async move {
        // Create a channel for incoming (remote) changes
        let (tx_remote, rx_remote) = futures::channel::mpsc::unbounded();

        // Spawn a helper to apply remote changes to the store
        spawn(handle_remote_changes(rx_remote, store));

        network::run_sync_loop(
            rx_local,
            tx_remote,
            move || *master_key.read(),
            move || store.write().export_save(),
        )
        .await;
    });

    // Provide sync task to children (so they can trigger sync)
    // TaskPage and others can use use_context::<Coroutine<Vec<u8>>>()
    use_context_provider(|| sync_task);

    // Subscribe to Service Worker controller changes via JS glue (WASM only).
    // The closure is leaked intentionally: JS holds a permanent reference via addEventListener.
    #[cfg(target_arch = "wasm32")]
    use_hook(|| {
        let callback = Closure::<dyn FnMut(bool)>::new(move |is_active: bool| {
            tracing::info!(
                "Service Worker status: {}",
                if is_active { "active" } else { "inactive" }
            );
            service_worker_active.set(is_active);
        });

        if let Err(e) = subscribe_to_service_worker_status(&callback) {
            tracing::error!("Service Worker status subscription failed: {:?}", e);
        }

        callback.forget();
    });

    if is_checking() {
        return rsx! {
            components::loading::Loading {}
        };
    }

    rsx! {
        // The Stylesheet component inserts a style link into the head of the document
        document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css"),
        }

        Router::<Route> {}
    }
}

async fn handle_remote_changes(
    mut rx_remote: futures::channel::mpsc::UnboundedReceiver<Vec<u8>>,
    mut store: Signal<AppStore>,
) {
    while let Some(change) = rx_remote.next().await {
        store.write().import_changes(change);
    }
}

/// Register the service worker.
///
/// This function is always compiled and checked, but execution is gated
/// by the `pwa` feature in `main()` and it's a no-op on non-wasm32 targets.
#[allow(dead_code)] // TODO: remove this when this is used
fn init_service_worker() {
    #[cfg(all(feature = "pwa", target_arch = "wasm32"))]
    {
        // We use a safe JS shim to handle the registration logic.
        // This prevents WASM panics by handling feature detection and secure context checks
        // entirely in JavaScript (public/pwa-glue.js), avoiding calls to undefined browser APIs.
        wasm_bindgen_futures::spawn_local(async move {
            let promise = match register_service_worker() {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("registerServiceWorker unavailable: {:?}", e);
                    return;
                }
            };

            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(_) => tracing::info!("Service Worker registered"),
                Err(e) => tracing::error!("Service Worker registration failed: {:?}", e),
            }
        });
    }
}

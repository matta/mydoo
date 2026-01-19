#![allow(non_snake_case)]

// Include the generated build version from build.rs
include!(concat!(env!("OUT_DIR"), "/build_version.rs"));

use dioxus::prelude::*;
use futures::StreamExt;
use tasklens_store::doc_id::DocumentId;

mod components;
mod controllers;
mod hooks;
mod router;
mod seed;
mod utils;
mod views;

use crate::router::Route;
use tasklens_store::store::AppStore;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
pub async fn tasklensReset() -> Result<(), String> {
    tracing::info!("E2E Reset Triggered: Clearing storage...");
    let _ = tasklens_store::crypto::clear_key();

    if let Some(window) = web_sys::window() {
        // 1. Clear Local/Session Storage
        if let Ok(Some(ls)) = window.local_storage() {
            let _ = ls.clear();
        }
        if let Ok(Some(ss)) = window.session_storage() {
            let _ = ss.clear();
        }

        // 2. Delete IndexedDB database "tasklens_db"
        if let Err(e) = rexie::Rexie::delete("tasklens_db").await {
            let msg = format!("Failed to delete database: {:?}", e);
            tracing::error!("{}", msg);
            return Err(msg);
        }
    }
    Ok(())
}

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

    #[cfg(target_arch = "wasm32")]
    {
        // Expose the reset function to the global window for E2E tests
        let window = web_sys::window().expect("no global window");
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::future_to_promise(async move {
                match tasklensReset().await {
                    Ok(_) => Ok(JsValue::UNDEFINED),
                    Err(e) => Err(JsValue::from_str(&e)),
                }
            })
        }) as Box<dyn FnMut() -> js_sys::Promise>);
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("tasklensReset"),
            closure.as_ref().unchecked_ref(),
        );
        closure.forget();
    }

    dioxus::launch(App);
}
#[derive(Clone, Copy)]
pub struct MemoryHeads(pub Signal<String>);
#[derive(Clone, Copy)]
pub struct PersistedHeads(pub Signal<String>);

fn App() -> Element {
    // Session state
    let mut doc_id = use_signal(|| None::<DocumentId>);
    // Loading state is true by default until initial doc load completes
    let mut is_checking = use_signal(|| true);
    // Mutated inside #[cfg(target_arch = "wasm32")] block below.
    #[allow(unused_mut)]
    let mut service_worker_active = use_signal(|| false);

    // Store State
    let mut store = use_signal(AppStore::new);

    // Give doc_id access to persistence
    let memory_heads = use_signal(String::new);
    let persisted_heads = use_signal(String::new);

    // Provide context early to avoid panics in hooks
    use_context_provider(|| doc_id);
    use_context_provider(|| service_worker_active);
    use_context_provider(|| store);
    use_context_provider(|| MemoryHeads(memory_heads));
    use_context_provider(|| PersistedHeads(persisted_heads));

    // Sync Client Hook
    let sync_status = hooks::use_sync::use_sync_client(store);
    hooks::use_persistence::use_persistence(store);

    use_context_provider(|| sync_status);

    // Unified Reactive Initialization
    use_future(move || async move {
        // --- Document Discovery ---
        tracing::info!("Initialization: Document Discovery");

        // Determine Initial Document ID
        // Determine Initial Document ID
        let (initial_id, must_create) = {
            if let Some(id) = AppStore::load_active_doc_id() {
                tracing::info!("Found active document ID: {}", id);
                (id, false)
            } else {
                let new_id = DocumentId::new();
                tracing::info!("No active doc. Generated new ID: {}", new_id);
                (new_id, true)
            }
        };

        // Perform Initial Load
        is_checking.set(true);

        let load_result = if must_create {
            Err(anyhow::anyhow!("Must create"))
        } else {
            // Load bytes first without holding lock
            match AppStore::load_from_db(&initial_id).await {
                Ok(Some(bytes)) => {
                    let mut s = store.write();
                    s.current_id = initial_id.clone();
                    s.load_from_bytes(bytes);
                    AppStore::save_active_doc_id(&initial_id);
                    Ok(())
                }
                Ok(None) => Err(anyhow::anyhow!("Doc not found")),
                Err(e) => Err(e),
            }
        };

        match load_result {
            Ok(_) => {
                tracing::info!("Initial load successful for: {}", initial_id);
                doc_id.set(Some(initial_id));
            }
            Err(_) => {
                tracing::info!("Initial load failed/new. Creating fresh doc.");
                let create_result = {
                    let mut s = store.write();
                    s.create_new()
                };

                match create_result {
                    Ok(new_id) => {
                        tracing::info!("Created new doc: {}", new_id);

                        // Async Save without holding lock
                        let save_data = {
                            let mut s = store.write();
                            (s.current_id.clone(), s.export_save())
                        };

                        spawn(async move {
                            if let Err(e) =
                                AppStore::save_doc_data_async(&save_data.0, save_data.1).await
                            {
                                tracing::error!("Failed to persist new doc: {:?}", e);
                            }
                        });

                        doc_id.set(Some(new_id));
                    }
                    Err(e) => tracing::error!("CRITICAL: Failed to create new doc: {:?}", e),
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Handle seed query param
            thread_local! {
                static INITIAL_LOAD_SEED_CHECKED: std::cell::Cell<bool> = std::cell::Cell::new(false);
            }
            let window = web_sys::window().unwrap();
            let search = window.location().search().unwrap_or_default();
            let trigger = search.contains("seed=true") && !INITIAL_LOAD_SEED_CHECKED.get();
            INITIAL_LOAD_SEED_CHECKED.set(true);

            if trigger {
                tracing::info!("Applying seed data...");
                {
                    let mut s = store.write();
                    crate::seed::prime_store_with_sample_data(&mut s);
                } // ensure lock dropped

                // Force save seeded data
                let save_data = {
                    let mut s = store.write();
                    (s.current_id.clone(), s.export_save())
                };
                spawn(async move {
                    if let Err(e) = AppStore::save_doc_data_async(&save_data.0, save_data.1).await {
                        tracing::error!("Failed to persist seeded doc: {:?}", e);
                    }
                });

                // Clean up URL
                if let Ok(history) = window.history() {
                    let _ = history.replace_state_with_url(
                        &wasm_bindgen::JsValue::NULL,
                        "",
                        Some("/plan"),
                    );
                }
            }
        }

        is_checking.set(false);
    });

    // Persistence Note:
    // We rely on explicit saves in action handlers (TaskPage, PlanPage, etc.) rather than a global
    // effect to avoid infinite loops with the sync client's observer polling.

    if is_checking() {
        return rsx! {
            components::loading::Loading {}
        };
    }

    rsx! {
        // Global Component Theme
        document::Link {
            rel: "stylesheet",
            href: asset!("/assets/dx-components-theme.css"),
        }
        // The Stylesheet component inserts a style link into the head of the document
        document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css"),
        }

        div {
            class: "min-h-screen",
            "data-memory-heads": "{memory_heads}",
            "data-persisted-heads": "{persisted_heads}",
            Router::<Route> {}
        }
    }
}

#[expect(dead_code)]
async fn handle_remote_changes(
    mut rx_remote: futures::channel::mpsc::UnboundedReceiver<Vec<u8>>,
    mut store: Signal<AppStore>,
) {
    while let Some(change) = rx_remote.next().await {
        if let Err(e) = store.write().import_changes(change) {
            tracing::error!("Failed to import remote changes in main loop: {:?}", e);
        }
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

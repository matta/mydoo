#![allow(non_snake_case)]

// Include the generated build version from build.rs
include!(concat!(env!("OUT_DIR"), "/build_version.rs"));

use dioxus::prelude::*;
use tasklens_store::doc_id::DocumentId;

pub mod app_components;
pub mod components;
pub mod controllers;
pub mod dioxus_components;
pub mod hooks;
pub mod router;
pub mod seed;
pub mod utils;
pub mod views;

use crate::app_components::LoadErrorView;
use crate::router::Route;
use tasklens_store::store::AppStore;

use samod::RepoBuilder;
#[cfg(target_arch = "wasm32")]
use tasklens_store::samod_runtime::WasmRuntime;
use tasklens_store::samod_storage::SamodStorage;
use tasklens_store::storage::ActiveDocStorage;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(all(
    target_arch = "wasm32",
    any(debug_assertions, feature = "e2e-test-hooks")
))]
pub async fn tasklensReset() -> Result<(), String> {
    tracing::info!("E2E Reset Triggered: Clearing storage...");

    if let Some(window) = web_sys::window() {
        // 1. Clear Local/Session Storage
        if let Ok(Some(ls)) = window.local_storage() {
            let _ = ls.clear();
        }
        if let Ok(Some(ss)) = window.session_storage() {
            let _ = ss.clear();
        }

        // 2. Delete IndexedDB database "tasklens_samod"
        if let Err(e) = rexie::Rexie::delete("tasklens_samod").await {
            let msg = format!("Failed to delete database: {:?}", e);
            tracing::error!("{}", msg);
            return Err(msg);
        }
    }
    Ok(())
}

/// Seed the active document with the default sample data set for E2E scenarios.
///
/// This is exposed to browser-based tests through `window.tasklensSeedSampleData`
/// so tests can prepare seeded state without a full page navigation.
#[cfg(all(
    target_arch = "wasm32",
    any(debug_assertions, feature = "e2e-test-hooks")
))]
pub fn tasklens_seed_sample_data(mut store: Signal<AppStore>) -> Result<(), String> {
    let mut app_store = store.write();

    if app_store.handle.is_none() {
        return Err("Cannot seed sample data before active document is ready".to_string());
    }

    tracing::info!("E2E Seed Triggered: injecting sample data into active document");
    crate::seed::prime_store_with_sample_data(&mut app_store);
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

    #[cfg(all(
        target_arch = "wasm32",
        any(debug_assertions, feature = "e2e-test-hooks")
    ))]
    {
        // Expose the reset function to the global window for E2E tests
        let window = web_sys::window().expect("global window must exist");
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
pub struct MemoryHeads(Signal<String>);

impl MemoryHeads {
    pub fn new(signal: Signal<String>) -> Self {
        Self(signal)
    }

    pub fn read(&self) -> impl std::ops::Deref<Target = String> + '_ {
        self.0.read()
    }

    pub fn set(&mut self, value: String) {
        self.0.set(value);
    }
}

impl std::fmt::Display for MemoryHeads {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy)]
pub struct PersistedHeads(Signal<String>);

impl PersistedHeads {
    pub fn new(signal: Signal<String>) -> Self {
        Self(signal)
    }

    pub fn read(&self) -> impl std::ops::Deref<Target = String> + '_ {
        self.0.read()
    }

    pub fn set(&mut self, value: String) {
        self.0.set(value);
    }
}
impl std::fmt::Display for PersistedHeads {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[tracing::instrument(skip_all)]
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
    let mut load_error = use_signal(|| None::<String>);

    // Give doc_id access to persistence
    let memory_heads = use_signal(String::new);
    let persisted_heads = use_signal(String::new);

    // Provide context early to avoid panics in hooks
    use_context_provider(|| doc_id);
    use_context_provider(|| service_worker_active);
    use_context_provider(|| store);
    use_context_provider(|| load_error);
    use_context_provider(|| MemoryHeads::new(memory_heads));
    use_context_provider(|| PersistedHeads::new(persisted_heads));

    // Sync Client Hook
    let sync_status = hooks::use_sync::use_sync_client(store);
    hooks::use_persistence::use_persistence(
        doc_id.into(),
        MemoryHeads::new(memory_heads),
        PersistedHeads::new(persisted_heads),
    );

    // Context for TunnelState (Memoized state)
    let tunnel_state = hooks::use_tunnel_state::use_tunnel_state();
    use_context_provider(|| tunnel_state);

    use_context_provider(|| sync_status);

    // Unified Reactive Initialization
    use_future(move || async move {
        #[cfg(target_arch = "wasm32")]
        if let Some(window) = web_sys::window() {
            let search = window.location().search().unwrap_or_default();
            if search.contains("skip_db_init=true") {
                tracing::info!("Skipping App initialization due to skip_db_init flag");
                return;
            }
        }

        // --- Samod Repo Initialization ---

        #[cfg(target_arch = "wasm32")]
        {
            let runtime = WasmRuntime;
            // Use a specific DB name for Samod to avoid conflict with legacy data
            let storage = SamodStorage::new("tasklens_samod", "documents");
            let repo = RepoBuilder::new(runtime)
                .with_storage(storage)
                // TODO: Set announce policy if needed
                .load_local()
                .await;

            store.write().repo = Some(repo);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let runtime = crate::utils::dioxus_runtime::DioxusRuntime;
            // In-memory storage for desktop (persists only for session duration)
            let storage = SamodStorage::new("tasklens_samod", "documents");
            let repo = RepoBuilder::new(runtime)
                .with_storage(storage)
                .load_local()
                .await;

            store.write().repo = Some(repo);
        }

        // --- Document Discovery ---

        // Determine Initial Document ID from URL/LocalStorage
        let initial_url_opt = ActiveDocStorage::load_active_url();

        is_checking.set(true);

        let repo = store.read().repo.clone().expect("Repo initialized");

        if let Some(url) = initial_url_opt {
            let id = url.document_id();

            // Attempt to find existing document without holding store lock
            let find_res = AppStore::find_doc(repo.clone(), id).await;

            match find_res {
                Ok(Some(handle)) => {
                    store.write().set_active_doc(handle, id);
                    doc_id.set(Some(id));
                }
                Ok(None) => {
                    tracing::error!("Doc {} not found. Creating new.", id);
                    // Fallback to create new
                    match AppStore::create_new(repo.clone()).await {
                        Ok((handle, new_id)) => {
                            store.write().set_active_doc(handle, new_id);
                            doc_id.set(Some(new_id));
                        }
                        Err(e) => {
                            let msg = format!("CRITICAL: Failed to create new doc: {:?}", e);
                            tracing::error!("{}", msg);
                            load_error.set(Some(msg));
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Error finding doc: {:?}. Creating new.", e);
                    tracing::error!("{}", msg);
                    match AppStore::create_new(repo.clone()).await {
                        Ok((handle, new_id)) => {
                            store.write().set_active_doc(handle, new_id);
                            doc_id.set(Some(new_id));
                        }
                        Err(e) => {
                            let msg = format!("CRITICAL: Failed to create new doc: {:?}", e);
                            tracing::error!("{}", msg);
                            load_error.set(Some(msg));
                        }
                    }
                }
            }
        } else {
            // No active doc, create new
            match AppStore::create_new(repo.clone()).await {
                Ok((handle, new_id)) => {
                    store.write().set_active_doc(handle, new_id);
                    doc_id.set(Some(new_id));
                }
                Err(e) => {
                    let msg = format!("CRITICAL: Failed to create new doc: {:?}", e);
                    tracing::error!("{}", msg);
                    load_error.set(Some(msg));
                }
            }
        }

        #[cfg(all(
            target_arch = "wasm32",
            any(debug_assertions, feature = "e2e-test-hooks")
        ))]
        {
            // Handle seed query param
            thread_local! {
                static INITIAL_LOAD_SEED_CHECKED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
            }
            if let Some(window) = web_sys::window() {
                let search = window.location().search().unwrap_or_default();
                let trigger = search.contains("seed=true") && !INITIAL_LOAD_SEED_CHECKED.get();
                INITIAL_LOAD_SEED_CHECKED.set(true);

                if trigger {
                    tracing::info!("Applying seed data...");
                    {
                        let mut s = store.write();
                        crate::seed::prime_store_with_sample_data(&mut s);
                    }
                    // Samod auto-saves, no need to forcing save.

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
        }

        is_checking.set(false);
    });

    #[cfg(all(
        target_arch = "wasm32",
        any(debug_assertions, feature = "e2e-test-hooks")
    ))]
    {
        let seed_store = store;
        use_effect(move || {
            let window = web_sys::window().expect("global window must exist");
            let closure = Closure::wrap(Box::new(move || {
                let seed_store = seed_store;
                wasm_bindgen_futures::future_to_promise(async move {
                    match tasklens_seed_sample_data(seed_store) {
                        Ok(()) => Ok(JsValue::UNDEFINED),
                        Err(e) => Err(JsValue::from_str(&e)),
                    }
                })
            }) as Box<dyn FnMut() -> js_sys::Promise>);

            let _ = js_sys::Reflect::set(
                &window,
                &JsValue::from_str("tasklensSeedSampleData"),
                closure.as_ref().unchecked_ref(),
            );
            closure.forget();
        });
    }

    let has_error = load_error().is_some();
    let app_state = if has_error {
        "error"
    } else if is_checking() {
        "loading"
    } else {
        "ready"
    };

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
        document::Stylesheet {
            href: asset!("/assets/app.css"),
        }

        div {
            class: "min-h-screen",
            "data-app-state": app_state,
            "data-memory-heads": "{memory_heads()}",
            "data-persisted-heads": "{persisted_heads()}",
            if has_error {
                LoadErrorView {
                    error: load_error().unwrap_or_default(),
                    help_text: None,
                }
            } else if is_checking() {
                app_components::loading::Loading {}
            } else {
                Router::<Route> {}
            }
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

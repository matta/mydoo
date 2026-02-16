use dioxus::prelude::*;
use tasklens_store::store::AppStore;

#[cfg(all(target_arch = "wasm32", feature = "e2e-test-hooks"))]
use wasm_bindgen::prelude::*;
#[cfg(all(target_arch = "wasm32", feature = "e2e-test-hooks"))]
use web_sys::UrlSearchParams;

#[cfg(all(target_arch = "wasm32", feature = "e2e-test-hooks"))]
pub(crate) fn use_e2e_setup(store: Signal<AppStore>) {
    use tracing::warn;

    use_effect(move || {
        let window = web_sys::window().expect("global window must exist");
        let search = window.location().search().unwrap_or_default();

        let params = UrlSearchParams::new_with_str(&search).unwrap_or_else(|e| {
            // Per repository rule: 'When looking up data from a source that is expected to be a 'single source of truth', explicitly handle cases where data might be unexpectedly missing. Use `unwrap_or_else` with a warning log (e.g., `tracing::warn!`) to make such issues detectable and debuggable, rather than silently falling back to default values.'
            warn!("Failed to parse URL search parameters: {:?}", e);
            UrlSearchParams::new().unwrap() // Fallback to empty params
        });

        if params.get("e2e_hooks").as_deref() == Some("true") {
            tracing::info!("E2E Hooks Enabled");

            // 1. Expose tasklensReset to the global window for E2E tests
            let reset_closure = Closure::wrap(Box::new(|| {
                wasm_bindgen_futures::future_to_promise(async move {
                    match tasklens_reset_impl().await {
                        Ok(_) => Ok(JsValue::UNDEFINED),
                        Err(e) => Err(JsValue::from_str(&e)),
                    }
                })
            })
                as Box<dyn FnMut() -> js_sys::Promise>);

            let _ = js_sys::Reflect::set(
                &window,
                &JsValue::from_str("tasklensReset"),
                reset_closure.as_ref().unchecked_ref(),
            );
            reset_closure.forget();

            // 2. Expose tasklensSeedSampleData to the global window for E2E tests
            let seed_store = store;
            let seed_closure = Closure::wrap(Box::new(move || {
                let seed_store = seed_store;
                wasm_bindgen_futures::future_to_promise(async move {
                    match tasklens_seed_sample_data_impl(seed_store) {
                        Ok(()) => Ok(JsValue::UNDEFINED),
                        Err(e) => Err(JsValue::from_str(&e)),
                    }
                })
            }) as Box<dyn FnMut() -> js_sys::Promise>);

            let _ = js_sys::Reflect::set(
                &window,
                &JsValue::from_str("tasklensSeedSampleData"),
                seed_closure.as_ref().unchecked_ref(),
            );
            seed_closure.forget();

            // 3. Handle initial seed query param
            // We wait until the store has a handle before seeding.
            // Reading store.read() makes this effect re-run when the store is updated.
            if params.get("seed").as_deref() == Some("true") && store.read().handle.is_some() {
                thread_local! {
                    static INITIAL_LOAD_SEED_CHECKED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
                }

                if !INITIAL_LOAD_SEED_CHECKED.get() {
                    INITIAL_LOAD_SEED_CHECKED.set(true);

                    tracing::info!("Applying seed data from URL parameter...");
                    {
                        let mut s = store.write();
                        crate::seed::prime_store_with_sample_data(&mut s);
                    }

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
    });
}

#[cfg(all(target_arch = "wasm32", feature = "e2e-test-hooks"))]
async fn tasklens_reset_impl() -> Result<(), String> {
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

#[cfg(all(target_arch = "wasm32", feature = "e2e-test-hooks"))]
fn tasklens_seed_sample_data_impl(mut store: Signal<AppStore>) -> Result<(), String> {
    let mut app_store = store.write();

    if app_store.handle.is_none() {
        return Err("Cannot seed sample data before active document is ready".to_string());
    }

    tracing::info!("E2E Seed Triggered: injecting sample data into active document");
    crate::seed::prime_store_with_sample_data(&mut app_store);
    Ok(())
}

// Fallback for non-wasm32 or when feature is disabled
#[cfg(not(all(target_arch = "wasm32", feature = "e2e-test-hooks")))]
pub(crate) fn use_e2e_setup(_store: Signal<AppStore>) {}

use crate::components::navbar::{Navbar, NavbarItem, NavbarNav};
use crate::router::Route;
use crate::views::auth::SettingsModal;
use dioxus::prelude::*;
use tasklens_store::doc_id::DocumentId;
use tasklens_store::store::AppStore;

#[component]
pub fn AppNavBar() -> Element {
    let active_index = use_signal(|| 0);
    let mut show_settings = use_signal(|| false);
    let mut store = use_context::<Signal<AppStore>>();
    let mut doc_id = use_context::<Signal<Option<DocumentId>>>();

    // FIXME: This logic is duplicated and should be consolidated.
    let handle_doc_change = move |new_doc_id: DocumentId| {
        tracing::info!("Attempting to switch to Document ID: {}", new_doc_id);
        spawn(async move {
            tracing::info!("Switching to Document ID: {}", new_doc_id);

            // 1. Get repo without holding lock
            let repo = store.read().repo.clone();

            if let Some(repo) = repo {
                // 2. Perform async lookup detached from store instance
                match AppStore::find_doc(repo, new_doc_id.clone()).await {
                    Ok(Some(handle)) => {
                        tracing::info!(
                            "find_doc_detached successful for Document ID: {}",
                            new_doc_id
                        );

                        // 3. Acquire lock ONLY for the sync update
                        store.write().set_active_doc(handle, new_doc_id.clone());
                        tracing::info!("set_active_doc successful for Document ID: {}", new_doc_id);

                        let logged_doc_id = new_doc_id.clone();
                        doc_id.set(Some(new_doc_id));
                        tracing::info!(
                            "doc_id.set() successful for Document ID: {}",
                            logged_doc_id
                        );
                    }
                    Ok(None) => {
                        tracing::error!("Document not found: {}", new_doc_id);
                    }
                    Err(e) => {
                        tracing::error!("find_doc_detached failed: {:?}", e);
                    }
                }
            } else {
                tracing::error!("Repo not initialized");
            }
        });
    };

    // FIXME: This logic is duplicated and should be consolidated.
    let handle_create_doc = move |_| {
        tracing::info!("Creating new document");
        spawn(async move {
            // 1. Get repo without holding lock
            let repo = store.read().repo.clone();

            if let Some(repo) = repo {
                // 2. Perform async creation detached from store instance
                match AppStore::create_new(repo).await {
                    Ok((handle, new_id)) => {
                        tracing::info!("Created new doc successfully: {}", new_id);

                        // 3. Acquire lock ONLY for the sync update
                        store.write().set_active_doc(handle, new_id.clone());
                        doc_id.set(Some(new_id));
                    }
                    Err(e) => tracing::error!("Failed to create doc: {:?}", e),
                }
            } else {
                tracing::error!("Repo not initialized");
            }
        });
    };

    rsx! {
        if show_settings() {
            SettingsModal {
                on_close: move |_| show_settings.set(false),
                doc_id,
                on_doc_change: handle_doc_change,
                on_create_doc: handle_create_doc,
            }
        }
        Navbar {
            NavbarNav { index: active_index,
                NavbarItem {
                    index: active_index,
                    value: 0usize,
                    to: Route::PlanPage { focus_task: None, seed: None },
                    "Plan"
                }
                NavbarItem {
                    index: active_index,
                    value: 1usize,
                    to: Route::DoPage {},
                    "Do"
                }
                NavbarItem {
                    index: active_index,
                    value: 2usize,
                    to: Route::Balance {},
                    "Balance"
                }
            }
            div { class: "flex items-center space-x-2 pr-4",
                crate::components::SyncIndicator {}

                button {
                    class: "text-gray-500 hover:text-gray-700 p-1 rounded-md hover:bg-gray-100",
                    onclick: move |_| show_settings.set(true),
                    aria_label: "Settings",
                    "data-testid": "settings-button",
                    svg {
                        class: "h-6 w-6",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z",
                        }
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
                        }
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}

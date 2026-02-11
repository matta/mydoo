use crate::components::navbar::{Navbar, NavbarItem, NavbarNav};
use crate::components::search_panel::SearchPanel;
use crate::controllers::doc_controller;
use crate::dioxus_components::button::{Button, ButtonVariant};
use crate::router::Route;
use crate::views::auth::SettingsModal;
use dioxus::prelude::*;
use tasklens_store::doc_id::DocumentId;
use tasklens_store::store::AppStore;

#[component]
pub fn AppNavBar() -> Element {
    let active_index = use_signal(|| 0);
    let mut show_settings = use_signal(|| false);
    let mut show_search = use_signal(|| false);
    let store = use_context::<Signal<AppStore>>();
    let doc_id = use_context::<Signal<Option<DocumentId>>>();

    // Hydrate state at the top level and provide it to children
    let state = crate::hooks::use_tunnel_state::use_tunnel_state();
    use_context_provider(|| state);

    let handle_doc_change = move |new_doc_id: DocumentId| {
        doc_controller::switch_document(store, doc_id, new_doc_id);
    };

    let handle_create_doc = move |_| {
        doc_controller::create_new_document(store, doc_id);
    };

    // Global Ctrl+K / Cmd+K keyboard shortcut to toggle search.
    // Registered once on mount via use_hook to avoid leaking duplicate listeners.
    use_hook(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;

            let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let is_mod = event.meta_key() || event.ctrl_key();
                if is_mod && event.key() == "k" {
                    event.prevent_default();
                    show_search.set(!show_search());
                } else if event.key() == "Escape" && show_search() {
                    show_search.set(false);
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

            if let Some(window) = web_sys::window() {
                if let Err(e) = window
                    .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                {
                    tracing::warn!("Failed to register Ctrl+K listener: {:?}", e);
                }
            }

            // Leak the closure intentionally so the listener stays active.
            // This is a top-level app component that lives for the entire session.
            closure.forget();
        }
    });

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
                    to: Route::BalancePage {},
                    "Balance"
                }
            }
            div { class: "flex items-center space-x-2 pr-4",
                crate::components::SyncIndicator {}

                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| show_search.set(!show_search()),
                    aria_label: "Search tasks",
                    "data-testid": "search-button",
                    svg {
                        class: "h-6 w-6",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
                        }
                    }
                }

                Button {
                    variant: ButtonVariant::Ghost,
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
        SearchPanel {
            open: show_search,
            on_close: move |_| show_search.set(false),
        }
        Outlet::<Route> {}
    }
}

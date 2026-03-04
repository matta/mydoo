use crate::app_components::{BackButton, DocIdManager};
use crate::controllers::doc_controller;
use crate::router::ViewContext;
use dioxus::prelude::*;
use tasklens_store::doc_id::DocumentId;
use tasklens_store::store::AppStore;

#[component]
pub fn SettingsPage(ctx: Option<ViewContext>) -> Element {
    #[css_module("/src/views/settings_page.css")]
    struct Styles;

    let store = use_context::<Signal<AppStore>>();
    let doc_id = use_context::<Signal<Option<DocumentId>>>();
    let view_context = ctx.unwrap_or_default();

    let handle_doc_change = move |new_doc_id: DocumentId| {
        doc_controller::switch_document(store, doc_id, new_doc_id);
    };

    let handle_create_doc = move |_| {
        doc_controller::create_new_document(store, doc_id);
    };

    // Detect whether we arrived here via in-app navigation or deep link.
    // We check for an app-set marker in the *previous* history entry's state.
    // Dioxus router pushes state on navigation, so if the entry before us
    // has non-trivial state, we came from within the app.
    #[allow(unused_mut, unused_variables)]
    let mut can_go_back = use_signal(|| false);
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let has_back = window
                    .history()
                    .ok()
                    .and_then(|h| h.length().ok())
                    // Dioxus router uses history.pushState for navigation.
                    // A length > 2 means we have at least one in-app page
                    // before settings (length 1 = initial, 2 = settings push).
                    .map(|len| len > 2)
                    .unwrap_or(false);
                can_go_back.set(has_back);
            }
        }
    });

    let close_settings = {
        #[allow(unused_variables)]
        let view_context = view_context.clone();
        move |_: MouseEvent| {
            #[cfg(target_arch = "wasm32")]
            {
                if can_go_back() {
                    // In-App Parity Rule: behave like browser Back
                    if let Some(window) = web_sys::window() {
                        let _ = window.history().and_then(|h| h.back());
                    }
                    return;
                }

                // Deep-Link Exception: replace current history entry with
                // the fallback route so pressing Back again cannot re-open
                // settings.
                let fallback_path = match &view_context {
                    ViewContext::Do => "/do",
                    ViewContext::Plan => "/plan",
                };
                if let Some(window) = web_sys::window() {
                    let _ = window.location().replace(fallback_path);
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let nav = navigator();
                nav.go_back();
            }
        }
    };

    rsx! {
        div {
            "data-testid": "settings-page",
            class: Styles::settings_root,

            div { class: Styles::settings_header,
                div {
                    "data-testid": "close-settings",
                    BackButton {
                        onclick: close_settings,
                    }
                }
                h1 { class: Styles::settings_title, "Settings" }
            }

            div { class: Styles::settings_body,
                DocIdManager {
                    current_doc_id: doc_id,
                    on_change: handle_doc_change,
                    on_create: handle_create_doc,
                }
            }
        }
    }
}

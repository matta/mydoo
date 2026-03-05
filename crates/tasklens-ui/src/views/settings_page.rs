use crate::app_components::{BackButton, DocIdManager};
use crate::controllers::doc_controller;
use crate::router::ViewContext;
use dioxus::prelude::*;
use tasklens_store::doc_id::DocumentId;
use tasklens_store::store::AppStore;

#[css_module("/src/views/settings_page.css")]
struct Styles;

#[component]
pub fn SettingsPage(ctx: Option<ViewContext>) -> Element {
    let store = use_context::<Signal<AppStore>>();
    let doc_id = use_context::<Signal<Option<DocumentId>>>();
    let handle_doc_change = move |new_doc_id: DocumentId| {
        doc_controller::switch_document(store, doc_id, new_doc_id);
    };

    let handle_create_doc = move |_| {
        doc_controller::create_new_document(store, doc_id);
    };

    let close_settings = move |_: MouseEvent| {
        navigator().go_back();
    };

    rsx! {
        div {
            "data-testid": "settings-page",
            class: Styles::settings_root,

            div { class: Styles::settings_header,
                BackButton {
                    onclick: close_settings,
                    "data-testid": "close-settings",
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

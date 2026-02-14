//! Authentication and Settings Views.
//!
//! This module defines the user interface for document management settings.
//! It includes the [`SettingsModal`] which acts as the main container for
//! managing documents (switching, creating, etc).

use crate::app_components::DocIdManager;
use crate::dioxus_components::button::{Button, ButtonVariant};
use crate::dioxus_components::dialog::{DialogContent, DialogRoot, DialogTitle};
use dioxus::prelude::*;
use tasklens_store::doc_id::DocumentId;

/// The primary modal for managing application settings and documents.
///
/// This component now solely focuses on Document Management, as user identity
/// has been removed in favor of purely document-based access.
///
/// # Props
/// * `on_close` - Event handler triggered when the modal should be closed.
/// * `doc_id` - Signal containing the current document ID.
/// * `on_doc_change` - Event handler called when the document ID changes.
#[component]
pub fn SettingsModal(
    on_close: EventHandler<()>,
    doc_id: Signal<Option<DocumentId>>,
    on_doc_change: EventHandler<DocumentId>,
    on_create_doc: EventHandler<()>,
) -> Element {
    rsx! {
        DialogRoot { open: true, on_open_change: move |_| on_close.call(()),
            DialogContent { class: "max-w-lg",
                DialogTitle { "Document Management" }

                div { class: "py-4",
                    DocIdManager {
                        current_doc_id: doc_id,
                        on_change: on_doc_change,
                        on_create: on_create_doc,
                    }
                }

                div { class: "mt-6 flex justify-end",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| on_close.call(()),
                        "data-testid": "close-settings",
                        "Close"
                    }
                }
            }
        }
    }
}

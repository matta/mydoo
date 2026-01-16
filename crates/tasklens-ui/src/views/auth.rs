//! Authentication and Settings Views.
//!
//! This module defines the user interface for document management settings.
//! It includes the [`SettingsModal`] which acts as the main container for
//! managing documents (switching, creating, etc).

use crate::components::*;
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
        div {
            class: "fixed inset-0 z-50 overflow-y-auto",
            role: "dialog",
            "aria-modal": "true",
            "aria-label": "Settings",

            div { class: "flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0",
                div {
                    class: "fixed inset-0 transition-opacity",
                    aria_hidden: "true",
                    onclick: move |_| on_close.call(()),
                    div { class: "absolute inset-0 bg-gray-500 opacity-75" }
                }

                span {
                    class: "hidden sm:inline-block sm:align-middle sm:h-screen",
                    aria_hidden: "true",
                    "\u{200b}"
                }

                div { class: "relative z-10 inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full",
                    div { class: "bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4",
                        div { class: "sm:flex sm:items-start",
                            div { class: "mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left w-full",
                                h3 { class: "text-lg leading-6 font-medium text-gray-900 mb-4",
                                    "Document Management"
                                }

                                // 1. Document Management
                                div { class: "mb-8",
                                    DocIdManager {
                                        current_doc_id: doc_id,
                                        on_change: on_doc_change,
                                        on_create: on_create_doc,
                                    }
                                }
                            }
                        }
                    }
                    div { class: "bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse",
                        Button {
                            variant: ButtonVariant::Secondary,
                            class: "mt-3 w-full sm:mt-0 sm:ml-3 sm:w-auto",
                            onclick: move |_| on_close.call(()),
                            data_testid: "close-settings",
                            "Close"
                        }
                    }
                }
            }
        }
    }
}

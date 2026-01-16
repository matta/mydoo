//! Document ID Manager Component
//!
//! Provides UI for managing document identifiers, including:
//! - Displaying the current document ID
//! - Generating a new document (new random ID)
//! - Entering an existing document ID to switch documents

use crate::components::*;
use dioxus::prelude::*;

/// Document ID Manager component for switching between documents.
///
/// # Props
///
/// * `current_doc_id` - Signal containing the current document ID.
/// * `on_change` - Event handler called when the document ID changes.
#[component]
pub fn DocIdManager(
    current_doc_id: Signal<Option<tasklens_store::doc_id::DocumentId>>,
    on_change: EventHandler<tasklens_store::doc_id::DocumentId>,
    on_create: EventHandler<()>,
) -> Element {
    let mut show_input = use_signal(|| false);
    let mut input_value = use_signal(String::new);
    let mut error_msg = use_signal(String::new);
    let mut show_copy_toast = use_signal(|| false);

    let truncated_id = use_memo(move || {
        current_doc_id().map(|id| {
            let s = id.to_string();
            if s.len() > 8 {
                format!("{}...", &s[..8])
            } else {
                s
            }
        })
    });

    let handle_new_document = move |_| {
        on_create.call(());
    };

    let handle_enter_id = move |_| {
        let text = input_value().trim().to_string();

        if text.is_empty() {
            error_msg.set("Document ID cannot be empty".to_string());
            return;
        }

        // Try to parse as TaskLensUrl first, then as plain DocumentId
        let id = if let Ok(url) = text.parse::<tasklens_store::doc_id::TaskLensUrl>() {
            url.document_id
        } else if let Ok(id) = text.parse::<tasklens_store::doc_id::DocumentId>() {
            id
        } else {
            error_msg.set("Invalid Document ID or tasklens: URL".to_string());
            return;
        };

        error_msg.set(String::new());
        show_input.set(false);
        input_value.set(String::new());
        on_change.call(id);
    };

    let handle_copy = move |_| {
        if let Some(id) = current_doc_id() {
            let url = tasklens_store::doc_id::TaskLensUrl::from(id).to_string();
            spawn(async move {
                match document::eval(&format!(
                    "return navigator.clipboard.writeText('{}').then(() => true)",
                    url
                ))
                .await
                {
                    Ok(_) => {
                        tracing::info!("Document URL copied to clipboard");
                        show_copy_toast.set(true);
                        if let Err(e) = document::eval(
                            "return new Promise(r => setTimeout(() => r(true), 3000))",
                        )
                        .await
                        {
                            tracing::error!("Timer failed: {:?}", e);
                        }
                        show_copy_toast.set(false);
                    }
                    Err(e) => {
                        tracing::error!("Clipboard copy failed: {:?}", e);
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "space-y-4 border-t border-gray-200 pt-4 mt-4",
            h4 { class: "text-sm font-medium text-gray-700", "Document Management" }

            if !error_msg().is_empty() {
                Alert { variant: AlertVariant::Error, title: "Error", "{error_msg}" }
            }

            // Current Document Display
            div { class: "space-y-2",
                label { class: "block text-sm font-medium text-gray-700", "Current Document" }
                div { class: "flex items-center space-x-2",
                    div {
                        class: "flex-1 px-3 py-2 bg-gray-50 border border-gray-200 rounded text-sm font-mono text-gray-600",
                        "data-testid": "document-id-display",
                        {truncated_id().unwrap_or_else(|| "No document loaded".to_string())}
                    }
                    if let Some(id) = current_doc_id() {
                        span {
                            class: "sr-only",
                            "data-testid": "full-document-id",
                            "{id}"
                        }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_copy,
                        disabled: current_doc_id().is_none(),
                        "Copy Full ID"
                    }
                }

                if show_copy_toast() {
                    div { class: "text-sm text-green-600 flex items-center",
                        svg {
                            class: "h-4 w-4 mr-1",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M5 13l4 4L19 7",
                            }
                        }
                        "Copied to clipboard"
                    }
                }
            }

            // Action Buttons
            div { class: "flex space-x-2",
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: handle_new_document,
                    class: "flex-1",
                    data_testid: "new-document-button",
                    "New Document"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        show_input.set(!show_input());
                        error_msg.set(String::new());
                    },
                    class: "flex-1",
                    data_testid: "toggle-enter-id-button",
                    if show_input() {
                        "Cancel"
                    } else {
                        "Enter ID"
                    }
                }
            }

            // Enter ID Input (conditional)
            if show_input() {
                div { class: "space-y-2",
                    label { class: "block text-sm font-medium text-gray-700", "Enter Document ID" }
                    Input {
                        value: "{input_value}",
                        oninput: move |val| input_value.set(val),
                        placeholder: "Enter Base58 document ID...",
                        class: "w-full font-mono text-sm",
                        data_testid: "document-id-input",
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: handle_enter_id,
                        class: "w-full",
                        data_testid: "load-document-button",
                        "Load Document"
                    }
                }
            }
        }
    }
}

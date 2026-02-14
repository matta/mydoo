//! Document ID Manager Component
//!
//! Provides UI for managing document identifiers, including:
//! - Displaying the current document ID
//! - Generating a new document (new random ID)
//! - Entering an existing document ID to switch documents

use crate::components::{Alert, AlertVariant};
use crate::dioxus_components::button::{Button, ButtonVariant};
use crate::dioxus_components::input::Input;
use crate::hooks::use_tunnel_state::use_tunnel_state;
use dioxus::prelude::*;
// use dioxus::events::FormEvent;

/// Document ID Manager component for switching between documents.
///
/// # Props
///
/// * `current_doc_id` - Signal containing the current document ID.
/// * `on_change` - Event handler called when the document ID changes.
#[component]
pub(crate) fn DocIdManager(
    current_doc_id: Signal<Option<tasklens_store::doc_id::DocumentId>>,
    on_change: EventHandler<tasklens_store::doc_id::DocumentId>,
    on_create: EventHandler<()>,
) -> Element {
    let mut store = use_context::<Signal<tasklens_store::store::AppStore>>();
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

    let tunnel_state = use_tunnel_state();

    let metadata_doc_id = use_memo(move || {
        tunnel_state
            .read()
            .metadata
            .as_ref()
            .and_then(|meta| meta.automerge_url.as_ref())
            .and_then(|url| url.parse::<tasklens_store::doc_id::TaskLensUrl>().ok())
            .map(|url| url.document_id.to_string())
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
            error_msg.set("Invalid Document ID or automerge: URL".to_string());
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

    let handle_download = move |_| {
        let bytes = store.read().export_save();
        let current_id = store
            .read()
            .current_id
            .as_ref()
            .map(|id| id.to_string())
            .unwrap_or_default();

        // 1 MiB limit for data URL downloads (browsers have ~2MB limit, be conservative)
        const MAX_DOWNLOAD_SIZE: usize = 1024 * 1024;
        if bytes.len() > MAX_DOWNLOAD_SIZE {
            error_msg.set(format!(
                "Document too large to download ({:.1} MiB). Maximum size is 1 MiB.",
                bytes.len() as f64 / (1024.0 * 1024.0)
            ));
            return;
        }

        spawn(async move {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let filename = format!("mydoo-{}.automerge", current_id);
            let script = format!(
                r#"
                const link = document.createElement('a');
                link.href = 'data:application/octet-stream;base64,{}';
                link.download = '{}';
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
                "#,
                b64, filename
            );
            if let Err(e) = document::eval(&script).await {
                tracing::error!("Download failed: {:?}", e);
            }
        });
    };

    let handle_upload = move |evt: Event<FormData>| {
        spawn(async move {
            let files = evt.files();
            if let Some(file) = files.as_slice().first() {
                match file.read_bytes().await {
                    Ok(bytes) => {
                        // Get repo without holding lock
                        let repo = store.read().repo.clone();

                        if let Some(repo) = repo {
                            match tasklens_store::store::AppStore::import_doc(repo, bytes.to_vec())
                                .await
                            {
                                Ok((handle, new_id)) => {
                                    tracing::info!("Imported document: {}", new_id);
                                    // Acquire lock only to update state
                                    store.write().set_active_doc(handle, new_id);
                                    on_change.call(new_id);
                                }
                                Err(e) => {
                                    tracing::error!("Import failed: {:?}", e);
                                    error_msg.set(format!("Import failed: {}", e));
                                }
                            }
                        } else {
                            error_msg.set("Repo not initialized".to_string());
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to read file: {:?}", e);
                        error_msg.set(format!("Failed to read file: {}", e));
                    }
                }
            }
        });
    };

    rsx! {
        div { class: "space-y-4 border-t border-base-200 pt-4 mt-4",
            h4 { class: "text-base font-medium text-base-content/70", "Document Management" }

            if !error_msg().is_empty() {
                Alert { variant: AlertVariant::Error, title: "Error", "{error_msg}" }
            }

            // Current Document Display
            div { class: "space-y-2",
                label { class: "block text-base font-medium text-base-content/70", "Current Document" }
                div { class: "flex items-center space-x-2",
                    div {
                        class: "flex-1 px-3 py-3 bg-base-200 border border-base-300 rounded text-base font-mono text-base-content/80",
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

                if let Some(meta_id) = metadata_doc_id() {
                    div { class: "mt-2 pt-2 border-t border-base-200",
                        label { class: "block text-xs font-medium text-base-content/50 mb-1",
                            "Metadata ID (Internal)"
                        }
                        div { class: "flex items-center space-x-2",
                            div { class: "px-2 py-1 bg-base-200 border border-base-300 rounded text-xs font-mono text-base-content/80",
                                "{meta_id}"
                            }
                            if let Some(curr) = current_doc_id() {
                                if curr.to_string() != meta_id {
                                    span { class: "text-error text-xs font-bold", "Mismatch!" }
                                } else {
                                    span { class: "text-success text-xs", "Matches" }
                                }
                            }
                        }
                    }
                }

                if show_copy_toast() {
                    div { class: "text-base text-success flex items-center",
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
            div { class: "flex flex-wrap gap-2",
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: handle_new_document,
                    class: "flex-1 min-w-[120px]",
                    "data-testid": "new-document-button",
                    "New Document"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        show_input.set(!show_input());
                        error_msg.set(String::new());
                    },
                    class: "flex-1 min-w-[120px]",
                    "data-testid": "toggle-enter-id-button",
                    if show_input() {
                        "Cancel"
                    } else {
                        "Enter ID"
                    }
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: handle_download,
                    disabled: current_doc_id().is_none(),
                    class: "flex-1 min-w-[120px]",
                    "data-testid": "download-document-button",
                    "Download"
                }
                div { class: "flex-1 min-w-[120px] relative",
                    Button {
                        variant: ButtonVariant::Secondary,
                        class: "w-full",
                        "data-testid": "upload-document-button",
                        "Upload"
                    }
                    input {
                        r#type: "file",
                        accept: ".automerge",
                        class: "absolute inset-0 opacity-0 cursor-pointer",
                        onchange: handle_upload,
                        "data-testid": "document-upload-input",
                    }
                }
            }

            // Enter ID Input (conditional)
            if show_input() {
                div { class: "space-y-2",
                    label { class: "block text-base font-medium text-base-content/70", "Enter Document ID" }
                    Input {
                        value: input_value(),
                        oninput: move |evt: FormEvent| input_value.set(evt.value()),
                        placeholder: "Enter Base58 document ID...",
                        class: "w-full font-mono text-base",
                        "data-testid": "document-id-input",
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: handle_enter_id,
                        class: "w-full",
                        "data-testid": "load-document-button",
                        "Load Document"
                    }
                }
            }
        }
    }
}

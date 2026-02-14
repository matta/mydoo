use crate::dioxus_components::button::{Button, ButtonVariant};
use crate::dioxus_components::input::Input;
#[cfg(target_arch = "wasm32")]
use crate::hooks::use_sync::SYNC_SERVER_URL_KEY;
use crate::hooks::use_sync::SyncStatus;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use gloo_storage::{LocalStorage, Storage};

fn get_sync_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        LocalStorage::get::<String>(SYNC_SERVER_URL_KEY)
            .unwrap_or_else(|_| "ws://localhost:3000/sync".to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "ws://localhost:3000/sync".to_string()
    }
}

fn set_sync_url(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = LocalStorage::set(SYNC_SERVER_URL_KEY, url);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing::warn!("Sync URL settings not persisted on desktop: {}", url);
    }
}

fn reload_application() {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

#[component]
pub(crate) fn SyncIndicator() -> Element {
    let sync_status = use_context::<Signal<SyncStatus>>();
    let mut show_settings = use_signal(|| false);
    let mut url_input = use_signal(get_sync_url);

    let (color, text) = match sync_status() {
        SyncStatus::Disconnected => ("bg-base-content/20", "Disconnected"),
        SyncStatus::Connecting => ("bg-warning", "Connecting"),
        SyncStatus::Connected => ("bg-success", "Connected"),
        SyncStatus::Error(_e) => ("bg-error", "Error"),
    };

    rsx! {
        div {
            class: "relative inline-block",
            "data-testid": "sync-indicator",
            Button {
                variant: ButtonVariant::Ghost,
                "data-testid": "sync-status-button",
                onclick: move |_| {
                    let new_state = !show_settings();
                    if new_state {
                        url_input.set(get_sync_url());
                    }
                    show_settings.set(new_state);
                },
                span { class: "h-2 w-2 rounded-full {color}" }
                span { class: "text-base-content/80", "{text}" }
            }

            if show_settings() {
                div {
                    class: "absolute right-0 z-50 mt-2 w-64 rounded-md border border-base-300 bg-base-100 p-4 shadow-lg",
                    "data-testid": "sync-settings-popover",
                    h3 { class: "font-semibold text-base-content mb-3", "Sync Settings" }
                    div { class: "space-y-3",
                        div { class: "space-y-1",
                            label { class: "py-1 text-xs",
                                "Server URL"
                            }
                            Input {
                                class: "w-full",
                                "data-testid": "sync-server-url-input",
                                value: "{url_input}",
                                oninput: move |e: FormEvent| url_input.set(e.value()),
                            }
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                set_sync_url(&url_input());
                                show_settings.set(false);
                                reload_application();
                            },
                            "Save & Reconnect"
                        }
                    }
                }
            }
        }
    }
}

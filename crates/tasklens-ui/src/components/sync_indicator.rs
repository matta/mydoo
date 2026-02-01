use crate::components::Button;
use crate::components::button::ButtonVariant;
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
pub fn SyncIndicator() -> Element {
    let sync_status = use_context::<Signal<SyncStatus>>();
    let mut show_settings = use_signal(|| false);
    let mut url_input = use_signal(get_sync_url);

    let (color, text) = match sync_status() {
        SyncStatus::Disconnected => ("bg-gray-400", "Disconnected"),
        SyncStatus::Connecting => ("bg-yellow-400", "Connecting"),
        SyncStatus::Connected => ("bg-green-500", "Connected"),
        SyncStatus::Error(_e) => ("bg-red-500", "Error"),
    };

    rsx! {
        div {
            class: "relative inline-block",
            "data-testid": "sync-indicator",
            button {
                class: "flex items-center space-x-2 px-3 py-2 rounded-full text-sm font-medium bg-white border border-gray-200 shadow-sm hover:bg-gray-50 transition-colors",
                "data-testid": "sync-status-button",
                onclick: move |_| {
                    let new_state = !show_settings();
                    if new_state {
                        url_input.set(get_sync_url());
                    }
                    show_settings.set(new_state);
                },
                span { class: "h-2 w-2 rounded-full {color}" }
                span { class: "text-gray-600", "{text}" }
            }

            if show_settings() {
                div {
                    class: "absolute top-full left-0 mt-2 w-64 p-4 bg-white rounded-lg shadow-xl border border-gray-200 z-50",
                    "data-testid": "sync-settings-popover",
                    h3 { class: "text-base font-semibold text-gray-800 mb-3", "Sync Settings" }
                    div { class: "space-y-3",
                        div {
                            label { class: "block text-xs text-gray-500 mb-1", "Server URL" }
                            input {
                                class: "w-full px-2 py-2 text-base border border-gray-300 rounded focus:ring-1 focus:ring-blue-500 outline-none",
                                "data-testid": "sync-server-url-input",
                                value: "{url_input}",
                                oninput: move |e| url_input.set(e.value()),
                            }
                        }
                        div { class: "flex space-x-2",
                            Button {
                                variant: ButtonVariant::Primary,
                                class: "flex-1 text-sm py-2",
                                onclick: move |_| {
                                    set_sync_url(&url_input());
                                    show_settings.set(false);
                                    // Reload to reconnect with new URL
                                    // In a better implementation, the hook would watch this.
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
}

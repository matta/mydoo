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
    #[css_module("/src/app_components/sync_indicator.css")]
    struct Styles;

    let sync_status = use_context::<Signal<SyncStatus>>();
    let mut show_settings = use_signal(|| false);
    let mut url_input = use_signal(get_sync_url);

    let (color_class, text) = match sync_status() {
        SyncStatus::Disconnected => (Styles::status_disconnected, "Disconnected"),
        SyncStatus::Connecting => (Styles::status_connecting, "Connecting"),
        SyncStatus::Connected => (Styles::status_connected, "Connected"),
        SyncStatus::Error(_e) => (Styles::status_error, "Error"),
    };

    let dot_class = format!("{} {}", Styles::status_dot, color_class);

    rsx! {
        div {
            class: Styles::indicator_container,
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
                span { class: dot_class, }
                span { class: Styles::text_muted, "{text}" }
            }

            if show_settings() {
                div {
                    class: Styles::settings_popover,
                    "data-testid": "sync-settings-popover",
                    h3 { class: Styles::popover_title, "Sync Settings" }
                    div { class: Styles::settings_stack,
                        div { class: Styles::field_stack,
                            label { class: Styles::field_label,
                                "Server URL"
                            }
                            Input {
                                class: Styles::popover_input_full,
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

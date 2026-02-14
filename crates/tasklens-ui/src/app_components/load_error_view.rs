use crate::app_components::{Alert, AlertVariant};
use dioxus::prelude::*;

/// A shared component for displaying document load or hydration errors.
#[component]
pub fn LoadErrorView(error: String, help_text: Option<String>) -> Element {
    let help_text = help_text.unwrap_or_else(|| {
        "Access the settings menu to switch documents or change sync servers.".to_string()
    });

    rsx! {
        Alert {
            variant: AlertVariant::Error,
            title: "Loading Error",
            class: "shadow-sm items-start flex-col gap-4",
            div { class: "font-mono text-sm break-words leading-relaxed w-full",
                "{error}"
            }
            div { class: "divider my-0 opacity-20" }
            p { class: "text-sm opacity-90", "{help_text}" }
        }
    }
}

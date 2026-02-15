use crate::app_components::{Alert, AlertVariant};
use dioxus::prelude::*;

/// A shared component for displaying document load or hydration errors.
#[component]
pub fn LoadErrorView(error: String, help_text: Option<String>) -> Element {
    #[css_module("/src/app_components/load_error_view.css")]
    struct Styles;

    let help_text = help_text.unwrap_or_else(|| {
        "Access the settings menu to switch documents or change sync servers.".to_string()
    });

    rsx! {
        Alert {
            variant: AlertVariant::Error,
            title: "Loading Error",
            class: Some(Styles::error_alert.to_string()),
            div { class: Styles::error_details,
                "{error}"
            }
            div { class: Styles::divider }
            p { class: Styles::help_text, "{help_text}" }
        }
    }
}

use dioxus::prelude::*;

/// Variants for the [`Alert`] component, determining its color scheme and semantic meaning.
#[derive(PartialEq, Clone, Copy)]
pub enum AlertVariant {
    /// Blue-styled information alert.
    Info,
    /// Green-styled success alert.
    Success,
    /// Red-styled error alert.
    Error,
}

/// A flexible alert component for displaying messages to the user.
///
/// Use this component to show feedback, warnings, or informational messages.
/// It supports different variants for semantic coloring and an optional title.
///
/// # Props
///
/// * `variant` - The visual style of the alert (Info, Success, Error). Defaults to `Info`.
/// * `title` - An optional bold title displayed at the top of the alert.
/// * `children` - The content of the alert message.
/// * `class` - Optional additional CSS classes.
#[component]
pub fn Alert(
    variant: Option<AlertVariant>,
    title: Option<String>,
    children: Element,
    class: Option<String>,
) -> Element {
    let variant = variant.unwrap_or(AlertVariant::Info);
    let extra_classes = class.unwrap_or_default();

    let (bg_color, text_color, heading_color) = match variant {
        AlertVariant::Info => ("bg-blue-50", "text-blue-700", "text-blue-800"),
        AlertVariant::Success => ("bg-green-50", "text-green-700", "text-green-800"),
        AlertVariant::Error => ("bg-red-50", "text-red-700", "text-red-800"),
    };

    rsx! {
        div { class: "rounded-md {bg_color} p-4 {extra_classes}",
            div { class: "flex",
                // Icon could be added here based on variant, keeping it simple for now or strictly generic
                div { class: "ml-3",
                    if let Some(t) = title {
                        h3 { class: "text-sm font-bold {heading_color}", "{t}" }
                    }
                    div { class: "mt-2 text-sm {text_color}", {children} }
                }
            }
        }
    }
}

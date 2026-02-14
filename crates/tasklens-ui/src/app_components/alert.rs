use dioxus::prelude::*;

/// Variants for the [`Alert`] component, determining its color scheme and semantic meaning.
#[derive(PartialEq, Clone, Copy)]
pub(crate) enum AlertVariant {
    /// Blue-styled information alert.
    #[allow(dead_code)]
    Info,
    /// Green-styled success alert.
    #[allow(dead_code)]
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
pub(crate) fn Alert(
    variant: Option<AlertVariant>,
    title: Option<String>,
    children: Element,
    class: Option<String>,
) -> Element {
    let variant = variant.unwrap_or(AlertVariant::Info);
    let extra_classes = class.unwrap_or_default();

    let variant_class = match variant {
        AlertVariant::Info => "alert-info",
        AlertVariant::Success => "alert-success",
        AlertVariant::Error => "alert-error",
    };

    rsx! {
        div {
            class: "alert {variant_class} {extra_classes}",
            role: "alert",
            div {
                if let Some(t) = title {
                    h3 { class: "font-bold", "{t}" }
                }
                div { class: "text-base", {children} }
            }
        }
    }
}

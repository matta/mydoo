use dioxus::prelude::*;

/// Variants for the [`Button`] component, determining its visual style.
#[derive(PartialEq, Clone, Copy)]
pub enum ButtonVariant {
    /// Solid blue background, white text. Primary action.
    Primary,
    /// White background, gray border. Secondary action.
    Secondary,
    /// Solid red background, white text. Destructive action.
    #[expect(dead_code)]
    Destructive,
    /// No background, hover effect only. Low emphasis.
    Ghost,
}

/// A generic, accessible button component.
///
/// Wraps the standard HTML `<button>` element with consistent styling and behavior.
///
/// # Props
///
/// * `children` - The content to display inside the button (e.g., text, icons).
/// * `onclick` - Optional event handler for click events.
/// * `variant` - The visual style of the button. Defaults to `Primary`.
/// * `class` - Optional additional CSS classes.
/// * `disabled` - If true, the button is visually disabled and non-interactive.
/// * `type` - The HTML button type ("button", "submit", "reset"). Defaults to "button".
#[component]
pub fn Button(
    children: Element,
    onclick: Option<EventHandler<MouseEvent>>,
    variant: Option<ButtonVariant>,
    class: Option<String>,
    disabled: Option<bool>,
    r#type: Option<String>,
) -> Element {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    let disabled = disabled.unwrap_or(false);
    let type_str = r#type.unwrap_or("button".to_string());
    let extra_classes = class.unwrap_or_default();

    let base_classes = "inline-flex items-center justify-center rounded-md border text-sm font-medium shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";

    let variant_classes = match variant {
        ButtonVariant::Primary => {
            "border-transparent text-white bg-blue-600 hover:bg-blue-700 focus:ring-blue-500"
        }
        ButtonVariant::Secondary => {
            "border-gray-300 text-gray-700 bg-white hover:bg-gray-50 focus:ring-blue-500"
        }
        ButtonVariant::Destructive => {
            "border-transparent text-white bg-red-600 hover:bg-red-700 focus:ring-red-500"
        }
        ButtonVariant::Ghost => {
            "border-transparent text-gray-500 hover:text-gray-700 hover:bg-gray-100 shadow-none"
        }
    };

    let padding_classes = "px-4 py-2";

    rsx! {
        button {
            r#type: "{type_str}",
            class: "{base_classes} {variant_classes} {padding_classes} {extra_classes}",
            disabled,
            onclick: move |evt| {
                if let Some(handler) = onclick.as_ref().filter(|_| !disabled) {
                    handler.call(evt);
                }
            },
            {children}
        }
    }
}

use dioxus::prelude::*;

/// Variants for the [`Button`] component, determining its visual style.
#[derive(PartialEq, Clone, Copy)]
pub enum ButtonVariant {
    /// Solid blue background, white text. Primary action.
    Primary,
    /// White background, gray border. Secondary action.
    Secondary,
    /// Solid red background, white text. Destructive action.
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
    data_testid: Option<String>,
) -> Element {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    let disabled = disabled.unwrap_or(false);
    let type_str = r#type.unwrap_or("button".to_string());
    let extra_classes = class.unwrap_or_default();

    let base_classes = "btn";

    let variant_classes = match variant {
        ButtonVariant::Primary => "btn-primary",
        ButtonVariant::Secondary => "btn-neutral",
        ButtonVariant::Destructive => "btn-error text-white",
        ButtonVariant::Ghost => "btn-ghost",
    };

    let padding_classes = "";

    rsx! {
        button {
            r#type: "{type_str}",
            class: "{base_classes} {variant_classes} {padding_classes} {extra_classes}",
            disabled,
            "data-testid": data_testid,
            onclick: move |evt| {
                if let Some(handler) = onclick.as_ref().filter(|_| !disabled) {
                    handler.call(evt);
                }
            },
            {children}
        }
    }
}

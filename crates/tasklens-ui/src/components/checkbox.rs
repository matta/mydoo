use dioxus::prelude::*;

/// A styled checkbox component with an optional label.
///
/// # Props
///
/// * `checked` - The current checked state of the input.
/// * `onchange` - Event handler called when the checked state changes. Returns the new boolean value.
/// * `id` - Optional HTML ID for the input. If a label is provided, this connects the label to the input.
/// * `label` - Optional text label displayed next to the checkbox.
/// * `class` - Optional additional CSS classes for the input element.
#[component]
pub fn Checkbox(
    checked: bool,
    onchange: EventHandler<bool>,
    id: Option<String>,
    label: Option<String>,
    class: Option<String>,
) -> Element {
    let extra_classes = class.unwrap_or_default();
    let id_str = id.clone().unwrap_or_default();

    // The input itself
    let input_node = rsx! {
        input {
            id: "{id_str}",
            r#type: "checkbox",
            class: "checkbox checkbox-primary checkbox-sm {extra_classes}",
            checked,
            onchange: move |evt| onchange.call(evt.checked()),
            onclick: move |evt| evt.stop_propagation(),
        }
    };

    if let Some(text) = label {
        rsx! {
            label { class: "label cursor-pointer justify-start gap-3",
                {input_node}
                span { class: "label-text", "{text}" }
            }
        }
    } else {
        input_node
    }
}

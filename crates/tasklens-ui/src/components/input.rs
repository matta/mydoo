use dioxus::prelude::*;

/// A styled text input component.
///
/// Wraps the standard HTML `<input>` element with consistent styling.
///
/// # Props
///
/// * `value` - The current text value of the input.
/// * `oninput` - Event handler called when the text changes. Returns the new string value.
/// * `placeholder` - Optional placeholder text.
/// * `type` - The HTML input type (e.g., "text", "password", "email"). Defaults to "text".
/// * `class` - Optional additional CSS classes.
/// * `onkeypress` - Optional event handler for keyboard events (e.g., handling "Enter" key).
/// * `id` - Optional HTML ID for the input.
#[component]
pub fn Input(
    value: String,
    oninput: EventHandler<String>,
    placeholder: Option<String>,
    r#type: Option<String>,
    class: Option<String>,
    onkeypress: Option<EventHandler<KeyboardEvent>>,
    id: Option<String>,
) -> Element {
    let type_str = r#type.unwrap_or("text".to_string());
    let placeholder_str = placeholder.unwrap_or_default();
    let extra_classes = class.unwrap_or_default();
    let id_str = id.unwrap_or_default();

    rsx! {
        input {
            id: "{id_str}",
            r#type: "{type_str}",
            class: "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm {extra_classes}",
            placeholder: "{placeholder_str}",
            value: "{value}",
            oninput: move |evt| oninput.call(evt.value()),
            onkeypress: move |evt| {
                if let Some(handler) = onkeypress {
                    handler.call(evt);
                }
            }
        }
    }
}

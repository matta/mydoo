/// Browser-native datetime-local input wrapper with scoped CSS module styling.
///
/// Uses `input[type="datetime-local"]` for combined date+time selection.
/// CSS is scoped via Dioxus CSS modules to avoid class-name collisions.
use dioxus::prelude::*;

#[css_module("/src/app_components/date_time_input.css")]
struct Styles;

#[derive(PartialEq, Clone, Props)]
pub(crate) struct DateTimeInputProps {
    /// HTML `id` attribute for label association.
    #[props(optional)]
    pub id: Option<String>,
    /// Test hook attribute.
    #[props(optional)]
    pub data_testid: Option<String>,
    /// Current value in `YYYY-MM-DDThh:mm` format (or empty/None for no selection).
    pub value: Option<String>,
    /// Fired when the user picks a datetime; carries the `YYYY-MM-DDThh:mm` string.
    pub onchange: EventHandler<String>,
}

/// A thin wrapper around `<input type="datetime-local">` styled with a CSS module.
#[component]
pub fn DateTimeInput(props: DateTimeInputProps) -> Element {
    rsx! {
        input {
            r#type: "datetime-local",
            id: props.id,
            "data-testid": props.data_testid,
            class: Styles::datetime_input,
            value: props.value.unwrap_or_default(),
            onchange: move |evt: FormEvent| props.onchange.call(evt.value()),
        }
    }
}

/// Browser-native date input wrapper with scoped CSS module styling.
///
/// Uses `input[type="date"]` for date-only selection. CSS is scoped via
/// Dioxus CSS modules to avoid class-name collisions.
use dioxus::prelude::*;

#[css_module("/src/app_components/date_input.css")]
struct Styles;

#[derive(PartialEq, Clone, Props)]
pub(crate) struct DateInputProps {
    /// HTML `id` attribute for label association.
    #[props(optional)]
    pub id: Option<String>,
    /// Test hook attribute.
    #[props(optional)]
    pub data_testid: Option<String>,
    /// Current value in `YYYY-MM-DD` format (or empty/None for no selection).
    pub value: Option<String>,
    /// Fired when the user picks a date; carries the `YYYY-MM-DD` string.
    pub onchange: EventHandler<String>,
}

/// A thin wrapper around `<input type="date">` styled with a CSS module.
#[component]
pub fn DateInput(props: DateInputProps) -> Element {
    rsx! {
        input {
            r#type: "date",
            id: props.id,
            "data-testid": props.data_testid,
            class: Styles::date_input,
            value: props.value.unwrap_or_default(),
            onchange: move |evt: FormEvent| props.onchange.call(evt.value()),
        }
    }
}

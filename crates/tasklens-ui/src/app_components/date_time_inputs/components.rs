use dioxus::prelude::*;

#[css_module("/src/app_components/date_time_inputs/style.css")]
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
pub(crate) fn DateInput(props: DateInputProps) -> Element {
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
pub(crate) fn DateTimeInput(props: DateTimeInputProps) -> Element {
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

use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct DatePickerProps {
    #[props(optional)]
    pub id: Option<String>,
    pub value: Option<String>,
    pub onchange: EventHandler<String>,
}

#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    // Basic HTML5 date picker for now
    rsx! {
        input {
            r#type: "date",
            id: props.id,
            class: "border rounded p-2 text-sm bg-white dark:bg-gray-800 dark:border-gray-700 w-full",
            value: props.value.unwrap_or_default(),
            onchange: move |evt| props.onchange.call(evt.value()),
        }
    }
}

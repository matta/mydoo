use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub(crate) struct SelectProps {
    #[props(optional)]
    pub id: Option<String>,
    pub value: String,
    pub onchange: EventHandler<String>,
    pub children: Element,
    #[props(optional)]
    pub class: Option<String>,
}

#[component]
pub fn Select(props: SelectProps) -> Element {
    rsx! {
        select {
            id: props.id,
            class: format!(
                "w-full rounded-md border border-base-300 bg-base-100 px-3 py-2 text-base {}",
                props.class.unwrap_or_default(),
            ),
            value: props.value,
            onchange: move |evt| props.onchange.call(evt.value()),
            {props.children}
        }
    }
}

use crate::dioxus_components::button::{Button, ButtonVariant};
use crate::dioxus_components::input::Input;
use dioxus::prelude::*;

#[component]
pub(crate) fn TaskInput(
    value: Signal<String>,
    on_add: EventHandler<()>,
    #[props(default)] data_testid: Option<String>,
) -> Element {
    #[css_module("/src/app_components/task_input.css")]
    struct Styles;

    let mut value = value;
    rsx! {
        div {
            class: Styles::input_container,
            "data-testid": data_testid,
            Input {
                value: value(),
                oninput: move |evt: FormEvent| value.set(evt.value()),
                placeholder: "Add a new task...",
                class: Styles::input_field,
                onkeypress: move |evt: KeyboardEvent| {
                    if evt.key() == Key::Enter {
                        on_add.call(());
                    }
                },
            }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| on_add.call(()),
                "Add"
            }
        }
    }
}

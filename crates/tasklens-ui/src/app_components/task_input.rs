use crate::app_components::{AppInput, Row, RowAlign, RowGap};
use crate::dioxus_components::button::{Button, ButtonVariant};
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
        Row {
            class: Styles::input_container,
            gap: RowGap::Sm,
            align: RowAlign::Center,
            "data-testid": data_testid,
            div {
                class: Styles::input_field,
                AppInput {
                    full_width: true,
                    aria_label: "New task title",
                    value: value(),
                    oninput: move |evt: FormEvent| value.set(evt.value()),
                    placeholder: "Add a new task...",
                    onkeypress: move |evt: KeyboardEvent| {
                        if evt.key() == Key::Enter {
                            on_add.call(());
                        }
                    },
                }
            }
            Button {
                variant: ButtonVariant::Primary,
                disabled: value().trim().is_empty(),
                onclick: move |_| on_add.call(()),
                "Add"
            }
        }
    }
}

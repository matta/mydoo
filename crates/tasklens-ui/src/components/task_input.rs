use crate::components::*;
use dioxus::prelude::*;

#[component]
pub fn TaskInput(value: Signal<String>, on_add: EventHandler<()>) -> Element {
    let mut value = value;
    rsx! {
        div { class: "flex gap-2 mb-4",
            Input {
                value: "{value}",
                oninput: move |t| value.set(t),
                placeholder: "Add a new task...",
                class: "flex-grow mr-2",
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

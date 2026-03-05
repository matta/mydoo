use crate::dioxus_components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

/// A specialized button for navigation, featuring a "Back" label and a left-pointing arrow icon.
///
/// This component is a wrapper around [`Button`] with the `Ghost` variant, specifically designed
/// for returning to previous screens or closing modals.
///
/// # Props
///
/// * `onclick` - Event handler called when the button is clicked.
/// * `data_testid` - Optional E2E selector attached to the rendered button.
#[component]
pub(crate) fn BackButton(
    onclick: EventHandler<MouseEvent>,
    #[props(default)] data_testid: Option<String>,
) -> Element {
    #[css_module("/src/app_components/back_button.css")]
    struct Styles;

    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            class: Styles::back_button,
            onclick: move |evt| onclick.call(evt),
            "data-testid": data_testid,
            svg {
                class: Styles::back_icon,
                fill: "none",
                view_box: "0 0 24 24",
                stroke: "currentColor",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M10 19l-7-7m0 0l7-7m-7 7h18",
                }
            }
            "Back"
        }
    }
}

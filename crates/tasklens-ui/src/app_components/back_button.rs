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
#[component]
pub(crate) fn BackButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            class: "mb-4 pl-0 hover:bg-transparent",
            onclick: move |evt| onclick.call(evt),
            svg {
                class: "h-4 w-4 mr-1",
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

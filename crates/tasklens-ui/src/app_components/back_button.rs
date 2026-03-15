use crate::dioxus_components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[css_module("/src/app_components/back_button.css")]
struct Styles;

/// A specialized button for navigation, featuring a left-pointing arrow icon.
///
/// This component is a wrapper around [`Button`] with the `Icon` variant, specifically designed
/// for returning to previous screens or closing modals.
///
/// # Props
///
/// * `onclick` - Event handler called when the button is clicked.
/// * `data_testid` - Optional E2E selector attached to the rendered button.
#[component]
pub(crate) fn BackButton(
    #[props(extends=Button)] attributes: Vec<Attribute>,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let base = attributes!(button {
        class: Styles::back_button,
        title: "Back",
        aria_label: "Back",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        Button {
            attributes: merged,
            variant: ButtonVariant::Icon,
            onclick: move |evt| onclick.call(evt),
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
        }
    }
}

use crate::dioxus_components::card::{Card, CardContent};
use dioxus::prelude::*;

/// A reusable empty state component.
///
/// Used to display a placeholder message when a list or view has no content.
#[component]
pub(crate) fn EmptyState(
    title: String,
    #[props(default)] subtitle: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        Card {
            CardContent {
                p { "{title}" }
                if let Some(sub) = subtitle {
                    p { "{sub}" }
                }
                div { {children} }
            }
        }
    }
}

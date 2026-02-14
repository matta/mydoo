use dioxus::prelude::*;

/// A reusable empty state component with consistent DaisyUI styling.
///
/// Used to display a placeholder message when a list or view has no content.
#[component]
pub(crate) fn EmptyState(
    title: String,
    #[props(default)] subtitle: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "card bg-base-200 border-2 border-dashed border-base-300",
            div { class: "card-body items-center text-center py-12",
                p { class: "text-base-content/60", "{title}" }
                if let Some(sub) = subtitle {
                    p { class: "text-base-content/60 mt-2", "{sub}" }
                }
                div { class: "mt-4", {children} }
            }
        }
    }
}

use dioxus::prelude::*;

#[css_module("/src/app_components/loading_style.css")]
struct Styles;

/// A full-screen loading component with a spinner.
#[component]
pub(crate) fn Loading() -> Element {
    rsx! {
        div { class: Styles::overlay,
            span { class: Styles::spinner }
            p { class: Styles::message, "Loading..." }
        }
    }
}

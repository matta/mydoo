use dioxus::prelude::*;

/// A full-screen loading component with a spinner.
#[component]
pub fn Loading() -> Element {
    rsx! {
        div { class: "fixed inset-0 flex flex-col items-center justify-center bg-base-100 z-50",
            span { class: "loading loading-spinner loading-lg text-primary mb-4" }
            p { class: "text-base-content font-medium animate-pulse", "Loading..." }
        }
    }
}

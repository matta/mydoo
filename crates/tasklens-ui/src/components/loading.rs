use dioxus::prelude::*;

/// A full-screen loading component with a spinner.
#[component]
pub fn Loading() -> Element {
    rsx! {
        div { class: "fixed inset-0 flex flex-col items-center justify-center bg-white dark:bg-stone-950 z-50",
            div { class: "relative w-24 h-24 mb-4",
                div { class: "absolute inset-0 border-4 border-gray-200 dark:border-stone-700 rounded-full" }
                div { class: "absolute inset-0 border-4 border-blue-500 rounded-full border-t-transparent animate-spin" }
                div { class: "absolute inset-4 border-4 border-purple-400 rounded-full border-b-transparent animate-spin-slow" }
            }
            p { class: "text-gray-600 dark:text-stone-400 font-medium animate-pulse-fast", "Loading..." }
        }
    }
}

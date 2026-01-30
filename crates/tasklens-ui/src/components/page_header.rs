use dioxus::prelude::*;

#[component]
pub fn PageHeader(title: String, children: Element) -> Element {
    rsx! {
        div { class: "flex justify-between items-center mb-6",
            h1 { class: "text-2xl font-bold text-gray-900 dark:text-white", "{title}" }
            div { class: "flex items-center space-x-2",
                {children}
            }
        }
    }
}

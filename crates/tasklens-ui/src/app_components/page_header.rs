use dioxus::prelude::*;

#[component]
pub(crate) fn PageHeader(title: String, children: Element) -> Element {
    #[css_module("/src/app_components/page_header.css")]
    struct Styles;

    rsx! {
        div { class: Styles::header_root,
            h1 { class: Styles::header_title, "{title}" }
            div { class: Styles::header_actions,
                {children}
            }
        }
    }
}

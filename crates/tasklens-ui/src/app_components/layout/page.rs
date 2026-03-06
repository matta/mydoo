use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[css_module("/src/app_components/layout/page.css")]
struct Styles;

/// A primitive layout component that handles safe area paddings and maximum width.
///
/// It encapsulates the `.page_container` styles previously duplicated across all views.
#[component]
pub(crate) fn PageContainer(
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: Styles::page_container,
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

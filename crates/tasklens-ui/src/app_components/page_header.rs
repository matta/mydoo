use crate::app_components::{Row, RowAlign, RowGap, RowJustify};
use dioxus::prelude::*;

#[component]
pub(crate) fn PageHeader(title: String, children: Element) -> Element {
    #[css_module("/src/app_components/page_header.css")]
    struct Styles;

    rsx! {
        Row { class: Styles::header_root, gap: RowGap::Sm, align: RowAlign::Center, justify: RowJustify::Between,
            h1 { class: Styles::header_title, "{title}" }
            Row { gap: RowGap::Sm, align: RowAlign::Center,
                {children}
            }
        }
    }
}

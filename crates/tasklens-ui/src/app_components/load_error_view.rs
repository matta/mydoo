use dioxus::prelude::*;

/// A shared component for displaying document load or hydration errors.
#[component]
pub fn LoadErrorView(error: String, help_text: Option<String>) -> Element {
    let help_text = help_text.unwrap_or_else(|| {
        "Access the settings menu to switch documents or change sync servers.".to_string()
    });

    rsx! {
        div { class: "alert alert-error shadow-sm items-start flex-col gap-4",
            div { class: "flex items-center gap-3 w-full",
                svg {
                    class: "h-6 w-6 shrink-0 stroke-current",
                    fill: "none",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
                    }
                }
                h2 { class: "text-lg font-bold", "Loading Error" }
            }
            div { class: "font-mono text-sm break-words leading-relaxed w-full",
                "{error}"
            }
            div { class: "divider my-0 opacity-20" }
            p { class: "text-sm opacity-90", "{help_text}" }
        }
    }
}

use dioxus::prelude::*;

/// A shared component for displaying document load or hydration errors.
#[component]
pub fn LoadErrorView(error: String, help_text: Option<String>) -> Element {
    let help_text = help_text.unwrap_or_else(|| {
        "Access the settings menu to switch documents or change sync servers.".to_string()
    });

    rsx! {
        div { class: "mt-8 p-6 bg-red-50 border border-red-200 rounded-xl shadow-sm",
            div { class: "flex items-center space-x-3 mb-4",
                span { class: "p-2 bg-red-100 rounded-lg",
                    svg {
                        class: "h-6 w-6 text-red-600",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
                        }
                    }
                }
                h2 { class: "text-lg font-semibold text-red-800", "Loading Error" }
            }
            div { class: "text-red-700 text-sm font-mono break-words leading-relaxed",
                "{error}"
            }
            div { class: "mt-6 pt-4 border-t border-red-100 flex flex-col space-y-3",
                p { class: "text-sm text-red-600", "{help_text}" }
            }
        }
    }
}

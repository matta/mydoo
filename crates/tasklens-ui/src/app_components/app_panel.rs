use crate::dioxus_components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

#[css_module("/src/app_components/app_panel.css")]
struct Styles;

/// Shared non-modal panel shell used by app-owned editing/settings surfaces.
#[component]
pub(crate) fn AppPanel(
    title: String,
    on_close: EventHandler<()>,
    children: Element,
    #[props(default)] subtitle: Option<String>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] panel_testid: Option<String>,
    #[props(default)] close_button_label: Option<String>,
    #[props(default)] close_button_testid: Option<String>,
    #[props(default)] header_actions: Option<Element>,
    #[props(default)] footer: Option<Element>,
) -> Element {
    let panel_aria_label = aria_label.unwrap_or_else(|| title.clone());
    let close_label = close_button_label.unwrap_or_else(|| "Close panel".to_string());
    let panel_testid = panel_testid.unwrap_or_default();
    let close_button_testid = close_button_testid.unwrap_or_default();

    rsx! {
        div {
            class: Styles::panel_shell,
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    on_close.call(());
                }
            },
            div {
                class: Styles::panel_root,
                role: "dialog",
                aria_modal: "false",
                aria_label: panel_aria_label,
                "data-testid": panel_testid,

                div { class: "dialog-header",
                    div { class: "dialog-title-group",
                        h2 { class: Styles::panel_title, "{title}" }
                        if let Some(text) = subtitle {
                            span { class: Styles::panel_subtitle, "{text}" }
                        }
                    }

                    div { class: Styles::panel_actions,
                        if let Some(actions) = header_actions {
                            {actions}
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "{Styles::panel_icon_button} app_ghost_hover app_transition",
                            aria_label: close_label,
                            "data-testid": close_button_testid,
                            onclick: move |_| on_close.call(()),
                            svg {
                                "fill": "none",
                                "viewBox": "0 0 24 24",
                                "stroke-width": "2",
                                "stroke": "currentColor",
                                class: Styles::close_icon,
                                path {
                                    "stroke-linecap": "round",
                                    "stroke-linejoin": "round",
                                    "d": "M6 18L18 6M6 6l12 12",
                                }
                            }
                        }
                    }
                }

                div { class: "{Styles::panel_body} dialog-body",
                    {children}
                }

                if let Some(footer_content) = footer {
                    div { class: "dialog-footer",
                        {footer_content}
                    }
                }
            }
        }
    }
}

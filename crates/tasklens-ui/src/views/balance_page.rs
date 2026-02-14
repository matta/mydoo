//! Balance Page
//!
//! Displays the Balance View showing effort distribution across root goals.
//! Users can adjust target percentages via sliders to rebalance their focus.

use crate::app_components::{BalanceSlider, EmptyState, LoadErrorView, PageHeader};
use crate::controllers::task_controller;
use crate::dioxus_components::badge::{Badge, BadgeVariant};
use crate::dioxus_components::card::{Card, CardContent};
use crate::dioxus_components::progress::Progress;
use crate::hooks::use_balance_interaction::{BalanceItem, use_balance_interaction};
use dioxus::prelude::*;

/// The main page component for the Balance View.
///
/// This view allows users to visualize and adjust the distribution of effort
/// across their top-level goals. It uses the `use_balance_interaction` hook
/// to manage the state of the sliders and the distribution logic.
#[component]
pub fn BalancePage() -> Element {
    let load_error = use_context::<Signal<Option<String>>>();
    let controller = task_controller::use_task_controller();
    let (render_items, interaction) =
        use_balance_interaction(EventHandler::new(move |distribution| {
            tracing::info!("set_balance_distribution: {:#?}", distribution);
            controller.set_balance_distribution(distribution);
        }));

    #[css_module("/src/views/balance_page.css")]
    struct Styles;

    rsx! {
        div {
            class: Styles::page_container,
            style: "padding-top: var(--safe-top); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",

            PageHeader { title: "Balance" }

            if let Some(error) = load_error() {
                LoadErrorView {
                    error,
                    help_text: Some(
                        "Access the settings menu to switch documents or change sync servers."
                            .to_string(),
                    ),
                }
            } else if render_items.is_empty() {
                EmptyState {
                    title: "No goals to balance.",
                    subtitle: "Create root-level tasks in the Plan view to see balance data.",
                }
            } else {
                div { class: Styles::item_list,
                    for item in render_items.iter().cloned() {
                        BalanceItemRow {
                            key: "{item.id}",
                            item,
                            on_input: move |(target_id, new_value)| interaction.handle_input(target_id, new_value),
                            on_change: move || interaction.handle_change(),
                        }
                    }
                }
            }
        }
    }
}

/// A row in the balance list representing a single root goal.
///
/// It displays the goal's title, its current status (Starving/Balanced),
/// and provides a slider to adjust the target percentage.
#[component]
fn BalanceItemRow(
    item: BalanceItem,
    on_input: EventHandler<(tasklens_core::types::TaskID, f64)>,
    on_change: EventHandler<()>,
) -> Element {
    #[css_module("/src/views/balance_page.css")]
    struct Styles;

    // If we have a preview, use it. Otherwise use the item's target (which comes from store).
    let current_target_percent = item.preview_percent.unwrap_or(item.target_percent);

    let display_target_pct = (current_target_percent * 100.0).round() as i32;
    let actual_pct = (item.actual_percent * 100.0).round() as i32;

    let status_variant = if item.is_starving {
        BadgeVariant::Destructive
    } else {
        BadgeVariant::Primary
    };

    let status_label = if item.is_starving {
        "Starving"
    } else {
        "Balanced"
    };

    rsx! {
        Card {
            "data-testid": "balance-item",
            "data-task-id": "{item.id}",
            "data-starving": "{item.is_starving}",

            CardContent {
                div { class: Styles::item_header,
                    div {
                        h3 { class: Styles::item_title, "{item.title}" }
                        Badge {
                            variant: status_variant,
                            "data-testid": "balance-status",
                            "{status_label}"
                        }
                    }
                    div { class: Styles::stats_container,
                        div { class: Styles::stat_label,
                            "Target: "
                            span { class: Styles::stat_value, "{display_target_pct}%" }
                        }
                        div { class: Styles::stat_label,
                            "Actual: "
                            span { class: Styles::stat_value, "{actual_pct}%" }
                        }
                    }
                }

                BalanceBar {
                    target_percent: current_target_percent,
                    actual_percent: item.actual_percent,
                }

                div { class: Styles::slider_section,
                    label { class: Styles::slider_label,
                        "Adjust Target"
                    }
                    BalanceSlider {
                        min: 0.01,
                        max: 1.0,
                        step: 0.01,
                        value: current_target_percent,
                        oninput: {
                            let target_id = item.id.clone();
                            move |new_value| on_input.call((target_id.clone(), new_value))
                        },
                        onchange: move |_| on_change.call(()),
                    }
                }
            }
        }
    }
}

/// A visual progress bar that displays both the target distribution and the actual effort.
///
/// Uses DaisyUI progress component with an overlay for the target indicator.
/// The actual effort is shown as the primary progress bar, with target as a subtle overlay.
#[component]
fn BalanceBar(target_percent: f64, actual_percent: f64) -> Element {
    #[css_module("/src/views/balance_page.css")]
    struct Styles;

    let target_value = (target_percent * 100.0).clamp(0.0, 100.0) as i32;
    let actual_value = (actual_percent * 100.0).clamp(0.0, 100.0) as i32;

    rsx! {
        div { class: Styles::progress_container,
            Progress {
                value: actual_value as f64,
                max: 100.0,
                class: Styles::progress_bar,
                "data-testid": "actual-bar",
            }
            div {
                class: Styles::target_indicator,
                style: "width: {target_value}%",
                "data-testid": "target-bar",
            }
        }
    }
}

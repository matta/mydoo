//! Balance Page
//!
//! Displays the Balance View showing effort distribution across root goals.
//! Users can adjust target percentages via sliders to rebalance their focus.

use crate::components::{LoadErrorView, PageHeader};
use crate::controllers::task_controller;
use crate::hooks::use_balance_interaction::{BalanceItem, use_balance_interaction};
use dioxus::prelude::*;

#[component]
pub fn BalancePage() -> Element {
    let load_error = use_context::<Signal<Option<String>>>();
    let task_controller = task_controller::use_task_controller();
    let (render_items, interaction) =
        use_balance_interaction(EventHandler::new(move |update_map| {
            tracing::info!("set_balance_distribution: {:#?}", update_map);
            task_controller.set_balance_distribution(update_map);
        }));

    rsx! {
        div {
            class: "px-4 pt-4 pb-20 container mx-auto max-w-2xl",
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
                div { class: "text-center py-12 text-gray-500 bg-gray-50 rounded-lg border-2 border-dashed border-gray-200 dark:bg-stone-800 dark:border-stone-700 dark:text-stone-400",
                    p { "No goals to balance." }
                    p { class: "text-base mt-2",
                        "Create root-level tasks in the Plan view to see balance data."
                    }
                }
            } else {
                div { class: "space-y-4",
                    for item in render_items.iter().cloned() {
                        BalanceItemRow {
                            key: "{item.id}",
                            item,
                            on_input: move |(id, val)| interaction.handle_input(id, val),
                            on_change: move || interaction.handle_change(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BalanceItemRow(
    item: BalanceItem,
    on_input: EventHandler<(tasklens_core::types::TaskID, f64)>,
    on_change: EventHandler<()>,
) -> Element {
    // If we have a preview, use it. Otherwise use the item's target (which comes from store).
    let current_target_percent = item.preview_percent.unwrap_or(item.target_percent);

    let display_target_pct = (current_target_percent * 100.0).round() as i32;
    let actual_pct = (item.actual_percent * 100.0).round() as i32;

    let status_class = if item.is_starving {
        "text-red-600 dark:text-red-400"
    } else {
        "text-green-600 dark:text-green-400"
    };

    let status_label = if item.is_starving {
        "Starving"
    } else {
        "Balanced"
    };

    rsx! {
        div {
            class: "p-4 bg-white rounded-lg shadow border border-gray-100 dark:bg-stone-900 dark:border-stone-700",
            "data-testid": "balance-item",
            "data-task-id": "{item.id}",
            "data-starving": "{item.is_starving}",

            div { class: "flex justify-between items-start mb-3",
                div {
                    h3 { class: "font-medium text-gray-900 dark:text-white", "{item.title}" }
                    span {
                        class: "text-xs font-medium {status_class}",
                        "data-testid": "balance-status",
                        "{status_label}"
                    }
                }
                div { class: "text-right text-sm",
                    div { class: "text-gray-600 dark:text-stone-400",
                        "Target: "
                        span { class: "font-medium", "{display_target_pct}%" }
                    }
                    div { class: "text-gray-600 dark:text-stone-400",
                        "Actual: "
                        span { class: "font-medium", "{actual_pct}%" }
                    }
                }
            }

            BalanceBar {
                target_percent: current_target_percent,
                actual_percent: item.actual_percent,
            }

            div { class: "mt-3",
                label { class: "block text-xs text-gray-500 dark:text-stone-500 mb-1",
                    "Adjust Target"
                }
                crate::components::BalanceSlider {
                    min: 0.01,
                    max: 1.0,
                    step: 0.01,
                    value: current_target_percent,
                    oninput: {
                        let id = item.id.clone();
                        move |val| on_input.call((id.clone(), val))
                    },
                    onchange: move |_| on_change.call(()),
                }
            }
        }
    }
}

#[component]
fn BalanceBar(target_percent: f64, actual_percent: f64) -> Element {
    let target_width = format!("{}%", (target_percent * 100.0).clamp(0.0, 100.0));
    let actual_width = format!("{}%", (actual_percent * 100.0).clamp(0.0, 100.0));

    rsx! {
        div { class: "relative h-4 bg-gray-200 rounded-full overflow-hidden dark:bg-stone-700",
            div {
                class: "absolute h-full bg-blue-200 dark:bg-blue-900 rounded-full transition-all duration-300",
                style: "width: {target_width}",
                "data-testid": "target-bar",
            }
            div {
                class: "absolute h-full bg-blue-600 dark:bg-blue-500 rounded-full transition-all duration-300",
                style: "width: {actual_width}",
                "data-testid": "actual-bar",
            }
        }
    }
}

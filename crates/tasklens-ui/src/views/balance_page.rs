//! Balance Page
//!
//! Displays the Balance View showing effort distribution across root goals.
//! Users can adjust target percentages via sliders to rebalance their focus.

use std::collections::HashMap;

use crate::components::{LoadErrorView, PageHeader};
use crate::controllers::task_controller;
use crate::hooks::use_balance_data::use_balance_data;
use dioxus::prelude::*;
use tasklens_core::domain::balance_distribution::redistribute_percentages;
use tasklens_core::types::{BalanceItem, TaskID};

#[component]
pub fn BalancePage() -> Element {
    let load_error = use_context::<Signal<Option<String>>>();
    let balance_data = use_balance_data();
    let mut preview_targets = use_signal::<Option<HashMap<TaskID, f64>>>(|| None);
    let task_controller = task_controller::use_task_controller();

    let on_slider_input = move |(target_id, new_value): (TaskID, f64)| {
        let current_data = balance_data();
        let string_map = if let Some(preview) = preview_targets() {
            preview
        } else {
            // Initialize from current data
            let mut map = HashMap::new();
            for item in &current_data.items {
                map.insert(item.id.clone(), item.target_percent);
            }
            map
        };

        let new_map = redistribute_percentages(&string_map, &target_id, new_value);
        preview_targets.set(Some(new_map));
    };

    let on_slider_change = move |_| {
        if let Some(preview) = preview_targets() {
            // Commit logic: We have percentages, we need to convert to absolute credits.
            let total_desired_sum: f64 =
                balance_data().items.iter().map(|i| i.desired_credits).sum();

            // If total desired is 0, default to 100.0
            let mut base_total = total_desired_sum;
            if base_total < 0.1 {
                base_total = 100.0;
            }

            let mut distribution_update = HashMap::new();
            for (id, pct) in preview {
                let absolute = pct * base_total;
                distribution_update.insert(id, absolute);
            }

            task_controller.set_balance_distribution(distribution_update);
            preview_targets.set(None);
        }
    };

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
            } else if balance_data().items.is_empty() {
                div { class: "text-center py-12 text-gray-500 bg-gray-50 rounded-lg border-2 border-dashed border-gray-200 dark:bg-stone-800 dark:border-stone-700 dark:text-stone-400",
                    p { "No goals to balance." }
                    p { class: "text-base mt-2",
                        "Create root-level tasks in the Plan view to see balance data."
                    }
                }
            } else {
                div { class: "space-y-4",
                    for item in balance_data().items.iter() {
                        BalanceItemRow {
                            key: "{item.id}",
                            item: item.clone(),
                            preview_percent: preview_targets.as_ref().and_then(|m| m.get(&item.id).copied()),
                            on_input: on_slider_input,
                            on_change: on_slider_change,
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
    preview_percent: Option<f64>,
    on_input: EventHandler<(tasklens_core::types::TaskID, f64)>,
    on_change: EventHandler<()>,
) -> Element {
    // If we have a preview, use it. Otherwise use the item's target (which comes from store).
    let current_target_percent = preview_percent.unwrap_or(item.target_percent);

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

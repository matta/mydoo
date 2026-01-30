//! Balance Page
//!
//! Displays the Balance View showing effort distribution across root goals.
//! Users can adjust target percentages via sliders to rebalance their focus.

use crate::components::{LoadErrorView, PageHeader};
use crate::controllers::task_controller;
use crate::hooks::use_balance_data::use_balance_data;
use dioxus::prelude::*;
use tasklens_core::types::BalanceItem;

#[component]
pub fn BalancePage() -> Element {
    let load_error = use_context::<Signal<Option<String>>>();
    let balance_data = use_balance_data();

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
                        }
                    }
                }

                div { class: "mt-6 p-4 bg-gray-50 rounded-lg dark:bg-stone-800",
                    div { class: "text-sm text-gray-600 dark:text-stone-400",
                        "Total Credits: "
                        span { class: "font-medium text-gray-900 dark:text-white",
                            "{balance_data().total_credits:.1}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BalanceItemRow(item: BalanceItem) -> Element {
    let task_controller = task_controller::use_task_controller();

    let target_pct = (item.target_percent * 100.0).round() as i32;
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

    let on_slider_change = {
        let id = item.id.clone();
        move |new_value: f64| {
            task_controller.update_desired_credits(id.clone(), new_value);
        }
    };

    rsx! {
        div {
            class: "p-4 bg-white rounded-lg shadow border border-gray-100 dark:bg-stone-900 dark:border-stone-700",
            "data-testid": "balance-item",
            "data-task-id": "{item.id}",
            "data-starving": "{item.is_starving}",

            div { class: "flex justify-between items-start mb-3",
                div {
                    h3 { class: "font-medium text-gray-900 dark:text-white",
                        "{item.title}"
                    }
                    span {
                        class: "text-xs font-medium {status_class}",
                        "data-testid": "balance-status",
                        "{status_label}"
                    }
                }
                div { class: "text-right text-sm",
                    div { class: "text-gray-600 dark:text-stone-400",
                        "Target: "
                        span { class: "font-medium", "{target_pct}%" }
                    }
                    div { class: "text-gray-600 dark:text-stone-400",
                        "Actual: "
                        span { class: "font-medium", "{actual_pct}%" }
                    }
                }
            }

            BalanceBar {
                target_percent: item.target_percent,
                actual_percent: item.actual_percent,
            }

            div { class: "mt-3",
                label { class: "block text-xs text-gray-500 dark:text-stone-500 mb-1",
                    "Adjust Target ({item.desired_credits:.1} credits)"
                }
                input {
                    r#type: "range",
                    class: "w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-stone-700",
                    min: "0.1",
                    max: "10.0",
                    step: "0.1",
                    value: "{item.desired_credits}",
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<f64>() {
                            on_slider_change(val);
                        }
                    },
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

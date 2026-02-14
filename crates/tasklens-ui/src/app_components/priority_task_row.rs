use crate::dioxus_components::badge::{Badge, BadgeVariant};
use crate::dioxus_components::checkbox::Checkbox;
use crate::router::Route;
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
use tasklens_core::types::{ComputedTask, TaskID, TaskStatus, UrgencyStatus};

#[component]
pub(crate) fn PriorityTaskRow(
    task: ComputedTask,
    on_toggle: EventHandler<TaskID>,
    on_title_tap: EventHandler<TaskID>,
) -> Element {
    let is_done = task.status == TaskStatus::Done;
    let task_id_toggle = task.id.clone();
    let task_id_tap = task.id.clone();

    let urgency_variant = match task.urgency_status {
        UrgencyStatus::Overdue => Some(BadgeVariant::Destructive),
        UrgencyStatus::Urgent => Some(BadgeVariant::Secondary),
        UrgencyStatus::Active => Some(BadgeVariant::Primary),
        UrgencyStatus::Upcoming => Some(BadgeVariant::Primary),
        UrgencyStatus::None => None,
    };

    let urgency_label = match task.urgency_status {
        UrgencyStatus::Overdue => "Overdue",
        UrgencyStatus::Urgent => "Urgent",
        UrgencyStatus::Active => "Active",
        UrgencyStatus::Upcoming => "Upcoming",
        UrgencyStatus::None => "",
    };
    let score_label = format!("{:.3}", task.score);

    rsx! {
        div {
            class: "flex items-center gap-2 p-3 border border-base-200 rounded-lg shadow-sm transition-colors group",
            "data-testid": "task-item",
            "data-urgency": "{task.urgency_status:?}",

            Checkbox {
                checked: Some(if is_done {
                    CheckboxState::Checked
                } else {
                    CheckboxState::Unchecked
                }),
                on_checked_change: move |_| on_toggle.call(task_id_toggle.clone()),
            }

            span {
                class: format_args!(
                    "flex-grow cursor-pointer select-none text-base font-medium {}",
                    if is_done { "line-through text-base-content/50" } else { "text-base-content" },
                ),
                "data-testid": "task-title",
                onclick: move |_| on_title_tap.call(task_id_tap.clone()),
                "{task.title}"
            }

            Link {
                class: "text-xs text-base-content/50 hover:text-base-content/80",
                to: Route::ScoreTracePage {
                    task_id: task.id.clone(),
                },
                "data-testid": "task-score",
                "Score {score_label}"
            }

            if !is_done {
                if let Some(variant) = urgency_variant {
                    Badge {
                        variant,
                        class: "ml-2",
                        "data-testid": "urgency-badge",
                        "data-urgency": "{task.urgency_status:?}".to_lowercase(),
                        "{urgency_label}"
                    }
                }
            }
        }
    }
}

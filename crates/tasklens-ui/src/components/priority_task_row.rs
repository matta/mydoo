use crate::components::checkbox::Checkbox;
use crate::router::Route;
use dioxus::prelude::*;
use tasklens_core::types::{ComputedTask, TaskID, TaskStatus, UrgencyStatus};

#[component]
pub fn PriorityTaskRow(
    task: ComputedTask,
    on_toggle: EventHandler<TaskID>,
    on_title_tap: EventHandler<TaskID>,
) -> Element {
    let is_done = task.status == TaskStatus::Done;
    let task_id_toggle = task.id.clone();
    let task_id_tap = task.id.clone();

    let urgency_class = match task.urgency_status {
        UrgencyStatus::Overdue => "badge-error",
        UrgencyStatus::Urgent => "badge-warning",
        UrgencyStatus::Active => "badge-info",
        UrgencyStatus::Upcoming => "badge-success",
        UrgencyStatus::None => "hidden",
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
            class: "flex items-center gap-2 p-3 bg-base-100 border border-base-200 rounded-lg shadow-sm hover:bg-base-200/50 transition-colors group",
            "data-testid": "task-item",
            "data-urgency": "{task.urgency_status:?}",

            Checkbox {
                checked: is_done,
                onchange: move |_| on_toggle.call(task_id_toggle.clone()),
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

            if !is_done && !urgency_label.is_empty() {
                span {
                    class: format_args!("badge badge-sm ml-2 {}", urgency_class),
                    "data-testid": "urgency-badge",
                    "data-urgency": "{task.urgency_status:?}".to_lowercase(),
                    "{urgency_label}"
                }
            }
        }
    }
}

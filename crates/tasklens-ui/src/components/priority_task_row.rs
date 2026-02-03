use crate::components::checkbox::Checkbox;
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

    rsx! {
        div {
            class: "card card-compact card-bordered card-side bg-base-100 shadow-sm p-3 mb-2 items-center hover:bg-base-200/50 transition-colors group",
            "data-testid": "task-item",
            "data-urgency": "{task.urgency_status:?}",

            Checkbox {
                checked: is_done,
                onchange: move |_| on_toggle.call(task_id_toggle.clone()),
                class: "mr-3",
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

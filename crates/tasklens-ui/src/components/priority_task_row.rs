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
        UrgencyStatus::Overdue => "text-red-600 font-bold",
        UrgencyStatus::Urgent => "text-orange-600 font-semibold",
        UrgencyStatus::Active => "text-yellow-600",
        UrgencyStatus::Upcoming => "text-green-600",
        UrgencyStatus::None => "text-gray-500",
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
            class: "flex items-center p-3 mb-2 bg-white rounded-lg shadow-sm border border-gray-100 hover:bg-gray-50 group",
            "data-testid": "task-item",
            "data-urgency": "{task.urgency_status:?}",

            Checkbox {
                checked: is_done,
                onchange: move |_| on_toggle.call(task_id_toggle.clone()),
                class: "mr-3",
            }

            span {
                class: format_args!(
                    "flex-grow cursor-pointer select-none {} {}",
                    if is_done { "line-through text-gray-400" } else { "text-gray-800" },
                    if !is_done && task.urgency_status == UrgencyStatus::Overdue { "font-medium" } else { "" }
                ),
                "data-testid": "task-title",
                onclick: move |_| on_title_tap.call(task_id_tap.clone()),
                "{task.title}"
            }

            if !is_done && !urgency_label.is_empty() {
                span {
                    class: format_args!("text-xs px-2 py-0.5 rounded-full bg-opacity-10 {}", urgency_class),
                    "data-testid": "urgency-badge",
                    "data-urgency": "{task.urgency_status:?}".to_lowercase(),
                    "{urgency_label}"
                }
            }
        }
    }
}

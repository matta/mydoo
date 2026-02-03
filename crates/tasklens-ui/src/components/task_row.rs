use crate::components::checkbox::Checkbox;
use chrono::{Datelike, TimeZone};
use dioxus::prelude::*;
use tasklens_core::domain::dates::{UrgencyStatus, get_urgency_status};
use tasklens_core::types::{PersistedTask, TaskID, TaskStatus};

/// Formats a due date timestamp relative to the current time.
///
/// Returns "Yesterday", "Today", "Tomorrow", day-of-week for dates within 7 days,
/// or a formatted date string for further dates.
fn format_relative_due_date(due_ts: i64, now: i64) -> String {
    let secs = due_ts / 1000;
    let now_secs = now / 1000;

    let (dt, now_dt) = match (
        chrono::Utc.timestamp_opt(secs, 0).single(),
        chrono::Utc.timestamp_opt(now_secs, 0).single(),
    ) {
        (Some(dt), Some(now_dt)) => (dt, now_dt),
        _ => return String::new(),
    };

    let days_diff = (dt.date_naive() - now_dt.date_naive()).num_days();
    match days_diff {
        -1 => "Yesterday".to_string(),
        0 => "Today".to_string(),
        1 => "Tomorrow".to_string(),
        _ if days_diff > 0 && days_diff < 7 => dt.format("%A").to_string(),
        _ if dt.year() == now_dt.year() => dt.format("%b %d").to_string(),
        _ => dt.format("%b %d, %Y").to_string(),
    }
}

#[component]
pub fn TaskRow(
    task: PersistedTask,
    depth: usize,
    on_toggle: EventHandler<PersistedTask>,
    has_children: bool,
    is_expanded: bool,
    on_expand_toggle: EventHandler<TaskID>,
    on_rename: EventHandler<(TaskID, String)>,
    on_delete: EventHandler<TaskID>,
    on_create_subtask: EventHandler<TaskID>,
    on_title_tap: EventHandler<TaskID>,
    #[props(default = false)] is_highlighted: bool,
    effective_due_date: Option<i64>,
    effective_lead_time: Option<i64>,
) -> Element {
    let indentation = depth * 20;
    let is_done = task.status == TaskStatus::Done;

    // Clone IDs and task for closures
    let task_id_expand = task.id.clone();
    let task_id_delete = task.id.clone();
    let task_id_subtask = task.id.clone();
    let task_id_title_tap = task.id.clone();
    let task_toggle = task.clone();

    // Urgency Logic
    let now = js_sys::Date::now() as i64;
    let urgency = get_urgency_status(effective_due_date, effective_lead_time, now);
    let urgency_classes = match urgency {
        UrgencyStatus::Overdue => "text-red-600 flex-grow cursor-pointer font-medium",
        UrgencyStatus::Active | UrgencyStatus::Urgent => "text-orange-600 flex-grow cursor-pointer",
        _ => "text-gray-800 dark:text-stone-100 flex-grow cursor-pointer",
    };

    let title_class = if is_done {
        "line-through text-gray-400 dark:text-stone-500 flex-grow cursor-pointer"
    } else {
        urgency_classes
    };

    let badge_color = match urgency {
        UrgencyStatus::Overdue => "bg-red-500",
        UrgencyStatus::Active | UrgencyStatus::Urgent => "bg-orange-500",
        UrgencyStatus::Upcoming => "bg-yellow-500",
        _ => "bg-gray-300 dark:bg-stone-600",
    };

    rsx! {
        div {
            class: "flex items-center py-3 border-b border-gray-100 dark:border-stone-700 hover:bg-gray-50 dark:hover:bg-stone-800 group pr-2",
            class: if is_highlighted { "animate-flash" } else { "" },
            style: "padding-left: {indentation}px",
            "data-testid": "task-item",
            "data-urgency": match urgency {
                UrgencyStatus::Overdue => "Overdue",
                UrgencyStatus::Active => "Active",
                UrgencyStatus::Urgent => "Urgent",
                UrgencyStatus::Upcoming => "Upcoming",
                _ => "None",
            },

            // Expand/Collapse Chevron
            div { class: "w-10 flex justify-center flex-shrink-0",
                if has_children {
                    button {
                        class: "btn btn-ghost btn-xs btn-circle text-gray-500 dark:text-stone-400",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_expand_toggle.call(task_id_expand.clone());
                        },
                        "aria-label": "Toggle expansion",
                        "data-expanded": "{is_expanded}",
                        if is_expanded {
                            svg {
                                class: "w-4 h-4",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M19 9l-7 7-7-7",
                                }
                            }
                        } else {
                            svg {
                                class: "w-4 h-4",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M9 5l7 7-7 7",
                                }
                            }
                        }
                    }
                } else {
                    div { class: "w-10" } // Spacer
                }
            }

            Checkbox {
                checked: is_done,
                onchange: move |_| {
                    on_toggle.call(task_toggle.clone());
                },
                class: "cursor-pointer mr-2 flex-shrink-0",
            }

            // Urgency indicator badge
            if urgency != UrgencyStatus::None {
                span {
                    "data-testid": "urgency-badge",
                    "data-urgency": match urgency {
                        UrgencyStatus::Overdue => "Overdue",
                        UrgencyStatus::Active => "Active",
                        UrgencyStatus::Urgent => "Urgent",
                        UrgencyStatus::Upcoming => "Upcoming",
                        _ => "None",
                    },
                    class: "w-2 h-2 rounded-full inline-block ml-2 mb-0.5 {badge_color}",
                }
            }

            span {
                class: "{title_class}",
                "data-testid": "task-title",
                onclick: move |_| on_title_tap.call(task_id_title_tap.clone()),
                "{task.title}"
            }

            if let Some(due_ts) = effective_due_date {
                if !is_done {
                    span {
                        class: "text-base text-gray-400 dark:text-stone-500 ml-2",
                        "data-testid": "due-date-text",
                        {format_relative_due_date(due_ts, now)}
                    }
                }
            }

            // Actions
            div { class: "flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-2",
                button {
                    class: "btn btn-ghost btn-xs btn-circle text-gray-500 dark:text-stone-400",
                    title: "Add Subtask",
                    onclick: move |_| on_create_subtask.call(task_id_subtask.clone()),
                    svg {
                        class: "w-4 h-4",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 4v16m8-8H4",
                        }
                    }
                }
                button {
                    class: "btn btn-ghost btn-xs btn-circle text-red-500 hover:bg-red-100",
                    title: "Delete",
                    onclick: move |_| on_delete.call(task_id_delete.clone()),
                    svg {
                        class: "w-4 h-4",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                        }
                    }
                }
            }
        }
    }
}

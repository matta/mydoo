use crate::dioxus_components::badge::{Badge, BadgeVariant};
use crate::dioxus_components::button::{Button, ButtonVariant};
use crate::dioxus_components::checkbox::Checkbox;
use chrono::{Datelike, TimeZone};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
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
pub(crate) fn TaskRow(
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
    #[css_module("/src/app_components/task_row.css")]
    struct Styles;

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

    let (urgency_text_class, badge_variant, urgency_label) = match urgency {
        UrgencyStatus::Overdue => (Styles::text_error, BadgeVariant::Destructive, "Overdue"),
        UrgencyStatus::Urgent => (Styles::text_warning, BadgeVariant::Secondary, "Urgent"),
        UrgencyStatus::Active => (Styles::text_warning, BadgeVariant::Secondary, "Active"),
        UrgencyStatus::Upcoming => (Styles::text_normal, BadgeVariant::Primary, "Upcoming"),
        _ => (Styles::text_normal, BadgeVariant::Outline, ""),
    };

    let data_urgency = if urgency_label.is_empty() {
        "None"
    } else {
        urgency_label
    };

    let title_class = if is_done {
        format_args!("{} {}", Styles::title_base, Styles::title_done)
    } else {
        format_args!("{} {}", Styles::title_base, urgency_text_class)
    };

    let row_class = if is_highlighted {
        format_args!("{} {}", Styles::row_root, Styles::row_highlighted)
    } else {
        format_args!("{}", Styles::row_root)
    };

    rsx! {
        div {
            class: row_class,
            style: "--indent: {indentation}px; padding-left: var(--indent);",
            "data-testid": "task-item",
            "data-depth": "{depth}",
            "data-urgency": "{data_urgency}",

            // Expand/Collapse Chevron
            div { class: Styles::chevron_container,
                if has_children {
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |evt: MouseEvent| {
                            evt.stop_propagation();
                            on_expand_toggle.call(task_id_expand.clone());
                        },
                        "aria-label": "Toggle expansion",
                        "data-expanded": "{is_expanded}",
                        if is_expanded {
                            svg {
                                class: Styles::icon_sm,
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
                                class: Styles::icon_sm,
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
                    div { class: Styles::spacer_w10 } // Spacer
                }
            }

            Checkbox {
                checked: Some(if is_done {
                    CheckboxState::Checked
                } else {
                    CheckboxState::Unchecked
                }),
                on_checked_change: move |_| {
                    on_toggle.call(task_toggle.clone());
                },
                class: Styles::checkbox_custom,
            }

            // Urgency indicator badge
            if urgency != UrgencyStatus::None {
                Badge {
                    variant: badge_variant,
                    "data-testid": "urgency-badge",
                    "data-urgency": "{data_urgency}",
                    class: Styles::urgency_badge,
                    "{urgency_label}"
                }
            }

            span {
                class: title_class,
                "data-testid": "task-title",
                onclick: move |_| on_title_tap.call(task_id_title_tap.clone()),
                "{task.title}"
            }

            if let Some(due_ts) = effective_due_date {
                if !is_done {
                    span {
                        class: Styles::due_date,
                        "data-testid": "due-date-text",
                        {format_relative_due_date(due_ts, now)}
                    }
                }
            }

            // Actions
            div { class: Styles::actions_container,
                Button {
                    variant: ButtonVariant::Ghost,
                    title: "Add Subtask",
                    onclick: move |_| on_create_subtask.call(task_id_subtask.clone()),
                    svg {
                        class: Styles::icon_sm,
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
                Button {
                    variant: ButtonVariant::Destructive,
                    title: "Delete",
                    onclick: move |_| on_delete.call(task_id_delete.clone()),
                    svg {
                        class: Styles::icon_sm,
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

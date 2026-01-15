use crate::components::checkbox::Checkbox;
use dioxus::prelude::*;
use tasklens_core::types::{PersistedTask, TaskID, TaskStatus};

#[component]
pub fn TaskRow(
    task: PersistedTask,
    depth: usize,
    on_toggle: EventHandler<PersistedTask>,
    has_children: bool,
    is_expanded: bool,
    on_expand_toggle: EventHandler<TaskID>,
) -> Element {
    let indentation = depth * 20;
    let is_done = task.status == TaskStatus::Done;
    let task_toggle = task.clone();
    let task_id = task.id.clone();
    let task_id_debug = task.id.to_string();

    rsx! {
        div {
            class: "flex items-center py-2 border-b border-gray-100 hover:bg-gray-50",
            style: "padding-left: {indentation}px",
            "data-testid": "task-item",

            // Expand/Collapse Chevron
            div { class: "w-6 flex justify-center",
                if has_children {
                    div {
                        class: "cursor-pointer p-1 rounded hover:bg-gray-200 text-gray-500",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_expand_toggle.call(task_id.clone());
                        },
                        "aria-label": "Toggle expansion",
                        "data-expanded": "{is_expanded}",
                        if is_expanded {
                            // Down Chevron
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
                            // Right Chevron
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
                    div { class: "w-6 h-6" } // Spacer
                }
            }

            Checkbox {
                checked: is_done,
                onchange: move |_| {
                    on_toggle.call(task_toggle.clone());
                },
                class: "cursor-pointer mr-2",
            }

            span { class: if is_done { "line-through text-gray-400" } else { "text-gray-800" },
                "{task.title}"
            }

            div { class: "ml-auto text-xs text-gray-300 font-mono pr-2", "{task_id_debug}" }
        }
    }
}

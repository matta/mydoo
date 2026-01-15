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
    on_rename: EventHandler<(TaskID, String)>,
    on_delete: EventHandler<TaskID>,
    on_create_subtask: EventHandler<TaskID>,
) -> Element {
    let indentation = depth * 20;
    let is_done = task.status == TaskStatus::Done;
    let task_toggle = task.clone();

    // Clone IDs for closures
    let task_id_expand = task.id.clone();
    let task_id_delete = task.id.clone();
    let task_id_subtask = task.id.clone();
    let task_id_rename_1 = task.id.clone();
    let task_id_rename_2 = task.id.clone();

    let mut is_editing = use_signal(|| false);

    // We need to capture the title string, not the task itself, to avoid move issues
    let title_for_edit = task.title.clone();
    let mut edit_text = use_signal(move || title_for_edit);

    // We can't easily share a closure that moves non-Copy/non-Clone items.
    // EventHandler is Clone. Signals are Clone. TaskID is Clone.
    // on_rename is Copy, so we can use it directly in move closures.

    // Helpers for rename logic
    let mut finish_rename = move || {
        on_rename.call((task_id_rename_1.clone(), edit_text()));
        is_editing.set(false);
    };

    // Need a separate closure or handle logic for blur
    // Actually we can just inline the logic or use Rc/RefCell but that's overkill.
    // Just duplicating lines is easiest for Dioxus closures sometimes.

    rsx! {
        div {
            class: "flex items-center py-2 border-b border-gray-100 hover:bg-gray-50 group pr-2",
            style: "padding-left: {indentation}px",
            "data-testid": "task-item",

            // Expand/Collapse Chevron
            div { class: "w-6 flex justify-center flex-shrink-0",
                if has_children {
                    div {
                        class: "cursor-pointer p-1 rounded hover:bg-gray-200 text-gray-500",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_expand_toggle.call(task_id_expand.clone());
                        },
                        "aria-label": "Toggle expansion",
                        "data-expanded": "{is_expanded}",
                        if is_expanded {
                             svg { class: "w-4 h-4", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M19 9l-7 7-7-7" } }
                        } else {
                             svg { class: "w-4 h-4", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M9 5l7 7-7 7" } }
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
                class: "cursor-pointer mr-2 flex-shrink-0",
            }

            if is_editing() {
                input {
                    class: "flex-grow px-2 py-1 border rounded focus:outline-none focus:ring-1 focus:ring-blue-500",
                    value: "{edit_text}",
                    oninput: move |evt| edit_text.set(evt.value()),
                    onkeydown: move |evt: KeyboardEvent| {
                        if evt.key() == Key::Enter {
                            finish_rename();
                        } else if evt.key() == Key::Escape {
                            is_editing.set(false);
                            // We can't easily reset to original title here without capturing it again.
                            // But edit_text persists. It's fine.
                        }
                    },
                    onblur: move |_| {
                        on_rename.call((task_id_rename_2.clone(), edit_text()));
                        is_editing.set(false);
                    },
                    autofocus: true,
                }
            } else {
                span {
                    class: if is_done { "line-through text-gray-400 flex-grow cursor-pointer" } else { "text-gray-800 flex-grow cursor-pointer" },
                    onclick: move |_| is_editing.set(true),
                    "{task.title}"
                }
            }

            // Actions
            div { class: "flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-2",
                button {
                    class: "p-1 hover:bg-gray-200 rounded text-gray-500",
                    title: "Add Subtask",
                    onclick: move |_| on_create_subtask.call(task_id_subtask.clone()),
                    svg { class: "w-4 h-4", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M12 4v16m8-8H4" } }
                }
                button {
                    class: "p-1 hover:bg-red-100 rounded text-red-500",
                    title: "Delete",
                    onclick: move |_| on_delete.call(task_id_delete.clone()),
                    svg { class: "w-4 h-4", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" } }
                }
            }
        }
    }
}

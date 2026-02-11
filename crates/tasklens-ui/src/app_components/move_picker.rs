use crate::components::dialog::{DialogContent, DialogRoot, DialogTitle};
use crate::dioxus_components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use std::collections::HashSet;
use tasklens_core::types::{PersistedTask, TaskID, TunnelState};

#[component]
pub fn MovePicker(
    task_id: TaskID,
    on_select: EventHandler<Option<TaskID>>,
    on_close: EventHandler<()>,
) -> Element {
    let state = crate::hooks::use_tunnel_state::use_tunnel_state();

    let task_id_clone = task_id.clone();
    let task_title = use_memo(move || {
        state()
            .tasks
            .get(&task_id_clone)
            .map(|t| t.title.clone())
            .unwrap_or_default()
    });

    let task_id_clone2 = task_id.clone();
    let descendants = use_memo(move || {
        tasklens_core::domain::hierarchy::get_descendant_ids(&state(), &task_id_clone2)
    });

    let task_id_clone3 = task_id.clone();
    let flattened_tasks = use_memo(move || {
        let mut result = Vec::new();
        let state_val = state();
        let desc_ids = descendants();

        for root_id in &state_val.root_task_ids {
            flatten_eligible_recursive(
                root_id,
                &state_val,
                0,
                &mut result,
                &task_id_clone3,
                &desc_ids,
            );
        }
        result
    });

    rsx! {
        DialogRoot {
            // Already using DaisyUI 'modal' from component update.
            open: true,
            on_open_change: move |_| on_close.call(()),
            DialogContent {
                // Tailwind: 'max-w-md'. Justification: Modal width.
                class: "max-w-md",
                DialogTitle { "Move \"{task_title}\"" }

                // DaisyUI: 'menu' provides the list styling.
                // Tailwind: 'max-h-[60vh] overflow-y-auto'. Justification: Scrollable area for list.
                div { class: "mt-4 max-h-[60vh] overflow-y-auto border rounded-md menu bg-base-100 p-2",
                    // Option for Root
                    div {
                        // DaisyUI: 'btn btn-ghost' for item styling.
                        // Tailwind: 'w-full justify-start font-normal'. Justification: Align text left.
                        class: "btn btn-ghost btn-sm w-full justify-start font-normal",
                        onclick: move |_| on_select.call(None),
                        span { class: "font-medium", "(Root)" }
                    }

                    if flattened_tasks().is_empty() {
                        div { class: "p-4 text-center text-base-content/60", "No other valid parents found." }
                    } else {
                        for (task, depth) in flattened_tasks() {
                            div {
                                key: "{task.id}",
                                // DaisyUI: 'btn btn-ghost' for item styling.
                                // Tailwind: 'w-full justify-start font-normal'.
                                class: "btn btn-ghost btn-sm w-full justify-start font-normal",
                                // Tailwind: Dynamic indentation via style.
                                style: "padding-left: {12 + depth * 16}px",
                                onclick: {
                                    let id = task.id.clone();
                                    move |_| on_select.call(Some(id.clone()))
                                },
                                span { "{task.title}" }
                            }
                        }
                    }
                }

                div { class: "mt-4 flex justify-end",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn flatten_eligible_recursive(
    id: &TaskID,
    state: &TunnelState,
    depth: usize,
    result: &mut Vec<(PersistedTask, usize)>,
    exclude_id: &TaskID,
    descendant_ids: &HashSet<TaskID>,
) {
    // If this task is the one being moved or its descendant, skip it and its entire subtree
    if id == exclude_id || descendant_ids.contains(id) {
        return;
    }

    if let Some(task) = state.tasks.get(id) {
        result.push((task.clone(), depth));
        for child_id in &task.child_task_ids {
            flatten_eligible_recursive(
                child_id,
                state,
                depth + 1,
                result,
                exclude_id,
                descendant_ids,
            );
        }
    }
}

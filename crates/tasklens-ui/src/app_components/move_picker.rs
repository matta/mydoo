use crate::components::dialog::{DialogContent, DialogRoot, DialogTitle};
use crate::dioxus_components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use std::collections::HashSet;
use tasklens_core::types::{PersistedTask, TaskID, TunnelState};

#[component]
pub(crate) fn MovePicker(
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
            open: true,
            on_open_change: move |_| on_close.call(()),
            DialogContent {
                class: "max-w-md",
                DialogTitle { "Move \"{task_title}\"" }

                div { class: "mt-4 max-h-[60vh] overflow-y-auto border rounded-md bg-app-surface p-2 space-y-1",
                    // Option for Root
                    button {
                        r#type: "button",
                        class: "w-full rounded-md px-3 py-2 text-left font-normal hover:bg-app-surface-muted",
                        onclick: move |_| on_select.call(None),
                        span { class: "font-medium", "(Root)" }
                    }

                    if flattened_tasks().is_empty() {
                        div { class: "p-4 text-center text-app-text/60", "No other valid parents found." }
                    } else {
                        for (task, depth) in flattened_tasks() {
                            button {
                                key: "{task.id}",
                                r#type: "button",
                                class: "w-full rounded-md px-3 py-2 text-left font-normal hover:bg-app-surface-muted",
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

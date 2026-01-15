use crate::components::TaskInput;
use crate::components::task_row::TaskRow;
use crate::controllers::task_controller;
use dioxus::prelude::*;
use tasklens_core::types::{PersistedTask, TaskID, TunnelState};
use tasklens_store::store::AppStore;

#[component]
pub fn PlanPage() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    let sync_tx = use_context::<Coroutine<Vec<u8>>>();

    // Track expanded task IDs.
    let mut expanded_tasks = use_signal(std::collections::HashSet::<TaskID>::new);
    let mut input_text = use_signal(String::new);

    let flattened_tasks = use_memo(move || {
        let store = store.write();
        let state = store
            .hydrate::<TunnelState>()
            .unwrap_or_else(|_| TunnelState::default());

        let expanded = expanded_tasks.read();
        flatten_tasks(&state, &expanded)
    });

    // Helper to trigger sync and save
    let mut trigger_sync = move || {
        let changes_opt = store.write().get_recent_changes();
        if let Some(changes) = changes_opt {
            sync_tx.send(changes);
            let bytes = store.write().export_save();
            spawn(async move {
                let _ = AppStore::save_to_db(bytes).await;
            });
        }
    };

    let mut add_task = move || {
        let text = input_text();
        if text.trim().is_empty() {
            return;
        }

        task_controller::create_task(&mut store, None, text);
        trigger_sync();
        input_text.set(String::new());
    };

    let toggle_task = move |task: PersistedTask| {
        task_controller::toggle_task_status(&mut store, task.id);
        trigger_sync();
    };

    let handle_rename = move |(id, new_title): (TaskID, String)| {
        task_controller::rename_task(&mut store, id, new_title);
        trigger_sync();
    };

    let handle_delete = move |id: TaskID| {
        task_controller::delete_task(&mut store, id);
        trigger_sync();
    };

    let handle_create_subtask = move |parent_id: TaskID| {
        // For now, create a generic subtask or prompt?
        // Plan says "Add Subtask" -> task_controller::create_task(..., Some(id), ...).
        // UX: Maybe adding "New Subtask" string is okay, user can rename it.
        // Or we could eventually trigger the task creation form.
        // Let's create a default one for now to match the "create and then rename" flow used in some apps,
        // or just empty title? Controller checks empty title.
        // Let's create "New Task".

        task_controller::create_task(&mut store, Some(parent_id.clone()), "New Task".to_string());

        // Auto-expand the parent so we can see the child
        let mut expanded = expanded_tasks.write();
        expanded.insert(parent_id);

        trigger_sync();
    };

    let toggle_expand = move |id: TaskID| {
        let mut expanded = expanded_tasks.write();
        if expanded.contains(&id) {
            expanded.remove(&id);
        } else {
            expanded.insert(id);
        }
    };

    rsx! {
        div {
            class: "p-4 container mx-auto max-w-2xl",
            style: "padding-top: var(--safe-top); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",

            div { class: "flex justify-between items-center mb-6",
                h1 { class: "text-2xl font-bold", "Plan" }
            }

            TaskInput { value: input_text, on_add: move |_| add_task() }

            div { class: "bg-white shadow rounded-lg overflow-hidden mt-4",
                if flattened_tasks().is_empty() {
                    div { class: "p-4 text-center text-gray-500",
                        "No tasks found. Try adding seed data? (?seed=true)"
                    }
                } else {
                    for (task , depth , has_children , is_expanded) in flattened_tasks() {
                        TaskRow {
                            key: "{task.id}",
                            task: task.clone(),
                            depth,
                            on_toggle: toggle_task,
                            has_children,
                            is_expanded,
                            on_expand_toggle: toggle_expand,
                            on_rename: handle_rename,
                            on_delete: handle_delete,
                            on_create_subtask: handle_create_subtask,
                        }
                    }
                }
            }
        }
    }
}

fn flatten_tasks(
    state: &TunnelState,
    expanded: &std::collections::HashSet<TaskID>,
) -> Vec<(PersistedTask, usize, bool, bool)> {
    let mut result = Vec::new();
    for root_id in &state.root_task_ids {
        flatten_recursive(root_id, state, 0, &mut result, expanded);
    }
    result
}

fn flatten_recursive(
    id: &TaskID,
    state: &TunnelState,
    depth: usize,
    result: &mut Vec<(PersistedTask, usize, bool, bool)>,
    expanded: &std::collections::HashSet<TaskID>,
) {
    if let Some(task) = state.tasks.get(id) {
        let has_children = !task.child_task_ids.is_empty();
        let is_expanded = expanded.contains(id);

        result.push((task.clone(), depth, has_children, is_expanded));

        if is_expanded {
            for child_id in &task.child_task_ids {
                flatten_recursive(child_id, state, depth + 1, result, expanded);
            }
        }
    }
}

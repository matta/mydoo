use crate::components::task_row::TaskRow;
use dioxus::prelude::*;
use tasklens_core::types::{PersistedTask, TaskID, TaskStatus, TunnelState};
use tasklens_store::actions::{Action, TaskUpdates};
use tasklens_store::store::AppStore;

#[component]
pub fn PlanPage() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    // Track expanded task IDs. Default to empty (collapsed) or pre-expanded?
    // Test says "When I expand 'Project Alpha'". implies it starts collapsed.
    let mut expanded_tasks = use_signal(std::collections::HashSet::<TaskID>::new);

    let flattened_tasks = use_memo(move || {
        let store_read = store.read();
        let state_res = store_read.get_state();
        let expanded = expanded_tasks.read();

        match state_res {
            Ok(state) => flatten_tasks(&state, &expanded),
            Err(e) => {
                tracing::error!("Failed to get state for plan view: {:?}", e);
                Vec::new()
            }
        }
    });

    let toggle_task = move |task: PersistedTask| {
        // ... (existing toggle logic) ...
        let new_status = match task.status {
            TaskStatus::Done => TaskStatus::Pending,
            TaskStatus::Pending => TaskStatus::Done,
        };

        let action = match new_status {
            TaskStatus::Done => Action::CompleteTask { id: task.id },
            TaskStatus::Pending => Action::UpdateTask {
                id: task.id,
                updates: TaskUpdates {
                    status: Some(TaskStatus::Pending),
                    ..Default::default()
                },
            },
        };

        if let Err(e) = store.write().dispatch(action) {
            tracing::error!("Failed to toggle task: {:?}", e);
        } else {
            let sync_tx = use_context::<Coroutine<Vec<u8>>>();
            let changes_opt = store.write().get_recent_changes();
            if let Some(changes) = changes_opt {
                sync_tx.send(changes);
                let bytes = store.write().export_save();
                spawn(async move {
                    let _ = AppStore::save_to_db(bytes).await;
                });
            }
        }
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

            div { class: "bg-white shadow rounded-lg overflow-hidden",
                if flattened_tasks().is_empty() {
                    div { class: "p-4 text-center text-gray-500",
                        "No tasks found. Try adding seed data? (?seed=true)"
                    }
                } else {
                    for (task , depth , has_children , is_expanded) in flattened_tasks() {
                        // TODO: TaskRow needs to support expansion toggle
                        TaskRow {
                            key: "{task.id}",
                            task: task.clone(),
                            depth,
                            on_toggle: toggle_task,
                            has_children,
                            is_expanded,
                            on_expand_toggle: toggle_expand,
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

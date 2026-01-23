use dioxus::prelude::*;
use tasklens_core::types::{TaskID, TaskStatus, TunnelState};
use tasklens_store::actions::{Action, TaskUpdates};
use tasklens_store::store::AppStore;

pub fn create_task(
    mut store: Signal<AppStore>,
    load_error: Option<Signal<Option<String>>>,
    parent_id: Option<TaskID>,
    title: String,
) -> Option<TaskID> {
    let id = TaskID::new();
    let action = Action::CreateTask {
        id: id.clone(),
        parent_id,
        title,
    };
    if let Err(e) = store.write().dispatch(action) {
        tracing::error!("Failed to create task: {}", e);
        if let Some(mut le) = load_error {
            le.set(Some(e.to_string()));
        }
        None
    } else {
        Some(id)
    }
}

pub fn update_task(
    mut store: Signal<AppStore>,
    load_error: Option<Signal<Option<String>>>,
    id: TaskID,
    updates: TaskUpdates,
) {
    let action = Action::UpdateTask { id, updates };
    if let Err(e) = store.write().dispatch(action) {
        tracing::error!("Failed to update task: {}", e);
        if let Some(mut le) = load_error {
            le.set(Some(e.to_string()));
        }
    }
}

pub fn toggle_task_status(
    mut store: Signal<AppStore>,
    mut load_error: Signal<Option<String>>,
    id: TaskID,
) {
    let current_status = store
        .read()
        .hydrate::<TunnelState>()
        .map(|state| state.tasks.get(&id).map(|t| t.status))
        .map_err(|e| {
            tracing::error!("Failed to hydrate state for toggle: {}", e);
            load_error.set(Some(e.to_string()));
            e
        })
        .unwrap_or(None);

    if let Some(status) = current_status {
        match status {
            TaskStatus::Pending => {
                let current_time = chrono::Utc::now().timestamp_millis();
                let action = Action::CompleteTask {
                    id: id.clone(),
                    current_time,
                };
                if let Err(e) = store.write().dispatch(action) {
                    tracing::error!("Failed to complete task: {}", e);
                    load_error.set(Some(e.to_string()));
                }
            }
            TaskStatus::Done => {
                let action = Action::UpdateTask {
                    id: id.clone(),
                    updates: TaskUpdates {
                        status: Some(TaskStatus::Pending),
                        ..Default::default()
                    },
                };

                if let Err(e) = store.write().dispatch(action) {
                    tracing::error!("Failed to toggle task status: {}", e);
                    load_error.set(Some(e.to_string()));
                }
            }
        };
    }
}

// Keep rename_task for backward compatibility if needed, or remove if unused.
// The build error logs didn't show rename_task being missing in the *latest* run (wait, let me check).
// The first build run showed `rename_task` missing in `plan_page.rs`.
// The second build run (after I added it) didn't complain about it.
pub fn rename_task(
    mut store: Signal<AppStore>,
    mut load_error: Signal<Option<String>>,
    id: TaskID,
    new_title: String,
) {
    let action = Action::UpdateTask {
        id,
        updates: TaskUpdates {
            title: Some(new_title),
            ..Default::default()
        },
    };

    if let Err(e) = store.write().dispatch(action) {
        tracing::error!("Failed to rename task: {}", e);
        load_error.set(Some(e.to_string()));
    }
}

pub fn delete_task(
    mut store: Signal<AppStore>,
    mut load_error: Signal<Option<String>>,
    id: TaskID,
) {
    let action = Action::DeleteTask { id };

    if let Err(e) = store.write().dispatch(action) {
        tracing::error!("Failed to delete task: {}", e);
        load_error.set(Some(e.to_string()));
    }
}

pub fn move_task(
    mut store: Signal<AppStore>,
    mut load_error: Signal<Option<String>>,
    id: TaskID,
    new_parent_id: Option<TaskID>,
) {
    let action = Action::MoveTask { id, new_parent_id };

    if let Err(e) = store.write().dispatch(action) {
        tracing::error!("Failed to move task: {}", e);
        load_error.set(Some(e.to_string()));
    }
}

pub fn refresh_lifecycle(mut store: Signal<AppStore>, mut load_error: Signal<Option<String>>) {
    let current_time = chrono::Utc::now().timestamp_millis();
    let action = Action::RefreshLifecycle { current_time };

    if let Err(e) = store.write().dispatch(action) {
        tracing::error!("Failed to refresh lifecycle: {}", e);
        load_error.set(Some(e.to_string()));
    }
}

pub fn indent_task(store: Signal<AppStore>, mut load_error: Signal<Option<String>>, id: TaskID) {
    // 1. Identify current parent and siblings.
    // 2. Find previous sibling.
    // 3. Move to be child of previous sibling.
    let new_parent_opt = {
        let state: TunnelState = match store.read().hydrate() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to hydrate for indent: {}", e);
                load_error.set(Some(e.to_string()));
                return;
            }
        };

        // Find task's parent ID
        let task = match state.tasks.get(&id) {
            Some(t) => t,
            None => return,
        };
        let parent_id = task.parent_id.clone();

        // Get siblings list
        let siblings = if let Some(pid) = &parent_id {
            state
                .tasks
                .get(pid)
                .map(|t| &t.child_task_ids)
                .unwrap_or(&state.root_task_ids)
        } else {
            &state.root_task_ids
        };

        // Find index of self
        let index = siblings.iter().position(|x| *x == id);

        match index {
            Some(i) if i > 0 => {
                // Previous sibling exists
                Some(siblings[i - 1].clone())
            }
            _ => None,
        }
    };

    if let Some(new_parent) = new_parent_opt {
        move_task(store, load_error, id, Some(new_parent));
    }
}

pub fn outdent_task(store: Signal<AppStore>, mut load_error: Signal<Option<String>>, id: TaskID) {
    // 1. Identify current parent.
    // 2. Identify grandparent.
    // 3. Move to grandparent.
    let (should_move, new_parent_id) = {
        let state: TunnelState = match store.read().hydrate() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to hydrate for outdent: {}", e);
                load_error.set(Some(e.to_string()));
                return;
            }
        };

        let task = match state.tasks.get(&id) {
            Some(t) => t,
            None => return,
        };

        if let Some(parent_id) = &task.parent_id {
            // Has parent, so we can outdent to grandparent
            let parent = state.tasks.get(parent_id);
            let grandparent_id = parent.and_then(|p| p.parent_id.clone());
            (true, grandparent_id)
        } else {
            // Already root, cannot outdent
            (false, None)
        }
    };

    if should_move {
        move_task(store, load_error, id, new_parent_id);
    }
}

use dioxus::prelude::*;
use tasklens_core::types::{TaskID, TaskStatus};
use tasklens_store::actions::{Action, TaskUpdates};
use tasklens_store::store::AppStore;

/// Creates a new task with the given title.
///
/// If `parent_id` is provided, the task is created as a child of that task.
/// Empty or whitespace-only titles are silently ignored.
pub fn create_task(
    mut store: Signal<AppStore>,
    parent_id: Option<TaskID>,
    title: String,
) -> Option<TaskID> {
    if title.trim().is_empty() {
        return None;
    }

    let id = TaskID::new();
    if let Err(e) = store.write().dispatch(Action::CreateTask {
        id: id.clone(),
        parent_id,
        title,
    }) {
        tracing::error!("Failed to create task: {:?}", e);
        None
    } else {
        Some(id)
    }
}

/// Toggles the status of a task between `Pending` and `Done`.
///
/// If the task is `Pending`, it will be marked as `Done`.
/// If the task is `Done`, it will be marked as `Pending`.
pub fn toggle_task_status(mut store: Signal<AppStore>, task_id: TaskID) {
    let current_status = {
        let read = store.read();
        if let Ok(state) = read.hydrate::<tasklens_core::types::TunnelState>() {
            state.tasks.get(&task_id).map(|task| task.status)
        } else {
            None
        }
    };

    if let Some(status) = current_status {
        let result = match status {
            TaskStatus::Pending => store.write().dispatch(Action::CompleteTask {
                id: task_id,
                current_time: js_sys::Date::now() as i64,
            }),
            TaskStatus::Done => store.write().dispatch(Action::UpdateTask {
                id: task_id,
                updates: TaskUpdates {
                    status: Some(TaskStatus::Pending),
                    ..Default::default()
                },
            }),
        };

        if let Err(e) = result {
            tracing::error!("Failed to toggle task status: {:?}", e);
        }
    }
}

/// Deletes a task by ID.
pub fn delete_task(mut store: Signal<AppStore>, task_id: TaskID) {
    if let Err(e) = store.write().dispatch(Action::DeleteTask { id: task_id }) {
        tracing::error!("Failed to delete task: {:?}", e);
    }
}

/// Renames a task.
///
/// Empty or whitespace-only titles are silently ignored.
pub fn rename_task(mut store: Signal<AppStore>, task_id: TaskID, new_title: String) {
    if new_title.trim().is_empty() {
        return;
    }

    if let Err(e) = store.write().dispatch(Action::UpdateTask {
        id: task_id,
        updates: TaskUpdates {
            title: Some(new_title),
            ..Default::default()
        },
    }) {
        tracing::error!("Failed to rename task: {:?}", e);
    }
}

/// Updates a task with the given updates.
///
/// If the schedule type is set to `Routinely`, a `repeat_config` must be provided.
pub fn update_task(mut store: Signal<AppStore>, task_id: TaskID, updates: TaskUpdates) {
    // Validation: Routinely tasks MUST have a repeat_config
    if let (Some(tasklens_core::types::ScheduleType::Routinely), Some(None)) =
        (&updates.schedule_type, &updates.repeat_config)
    {
        tracing::error!("Validation failed: Routinely schedule requires a RepeatConfig");
        return;
    }

    if let Err(e) = store.write().dispatch(Action::UpdateTask {
        id: task_id,
        updates,
    }) {
        tracing::error!("Failed to update task: {:?}", e);
    }
}

/// Moves a task to a new parent.
pub fn move_task(mut store: Signal<AppStore>, task_id: TaskID, new_parent_id: Option<TaskID>) {
    if let Err(e) = store.write().dispatch(Action::MoveTask {
        id: task_id,
        new_parent_id,
    }) {
        tracing::error!("Failed to move task: {:?}", e);
    }
}

/// Indents a task: moves it under its previous sibling.
pub fn indent_task(store: Signal<AppStore>, task_id: TaskID) {
    let new_parent_id = {
        let read = store.read();
        if let Ok(state) = read.hydrate::<tasklens_core::types::TunnelState>() {
            tasklens_core::domain::hierarchy::get_previous_sibling(&state, &task_id)
        } else {
            None
        }
    };

    if let Some(pid) = new_parent_id {
        move_task(store, task_id, Some(pid));
    }
}

/// Outdents a task: moves it up one level in the hierarchy.
pub fn outdent_task(store: Signal<AppStore>, task_id: TaskID) {
    let new_parent_id = {
        let read = store.read();
        if let Ok(state) = read.hydrate::<tasklens_core::types::TunnelState>() {
            state.tasks.get(&task_id).and_then(|task| {
                if let Some(parent_id) = &task.parent_id {
                    let parent = state.tasks.get(parent_id)?;
                    parent.parent_id.clone()
                } else {
                    None
                }
            })
        } else {
            None
        }
    };

    // Note: If task is a child of a root task, task.parent_id is Some(root),
    // and root's parent_id is None. So outdenting moves it to root (new_parent_id = None).
    // If task is already root, outdent does nothing (or we should check it).

    let is_root = {
        let read = store.read();
        if let Ok(state) = read.hydrate::<tasklens_core::types::TunnelState>() {
            state
                .tasks
                .get(&task_id)
                .map(|t| t.parent_id.is_none())
                .unwrap_or(true)
        } else {
            true
        }
    };

    if !is_root {
        move_task(store, task_id, new_parent_id);
    }
}

/// Triggers the lifecycle refresh cycle (acknowledge completed tasks and wake up routine tasks).
pub fn refresh_lifecycle(mut store: Signal<AppStore>) {
    let current_time = js_sys::Date::now() as i64;
    if let Err(e) = store
        .write()
        .dispatch(Action::RefreshLifecycle { current_time })
    {
        tracing::error!("Failed to refresh lifecycle: {:?}", e);
    }
}

use dioxus::prelude::*;
use tasklens_core::types::{TaskID, TaskStatus};
use tasklens_store::actions::{Action, TaskUpdates};
use tasklens_store::store::AppStore;

/// Creates a new task with the given title.
///
/// If `parent_id` is provided, the task is created as a child of that task.
/// Empty or whitespace-only titles are silently ignored.
pub fn create_task(store: &mut Signal<AppStore>, parent_id: Option<TaskID>, title: String) {
    if title.trim().is_empty() {
        return;
    }

    if let Err(e) = store
        .write()
        .dispatch(Action::CreateTask { parent_id, title })
    {
        tracing::error!("Failed to create task: {:?}", e);
    }
}

/// Toggles the status of a task between `Pending` and `Done`.
///
/// If the task is `Pending`, it will be marked as `Done`.
/// If the task is `Done`, it will be marked as `Pending`.
pub fn toggle_task_status(store: &mut Signal<AppStore>, task_id: TaskID) {
    let current_status = {
        let read = store.read();
        if let Ok(state) = read.get_state() {
            state.tasks.get(&task_id).map(|task| task.status)
        } else {
            None
        }
    };

    if let Some(status) = current_status {
        let result = match status {
            TaskStatus::Pending => store.write().dispatch(Action::CompleteTask { id: task_id }),
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
pub fn delete_task(store: &mut Signal<AppStore>, task_id: TaskID) {
    if let Err(e) = store.write().dispatch(Action::DeleteTask { id: task_id }) {
        tracing::error!("Failed to delete task: {:?}", e);
    }
}

/// Renames a task.
///
/// Empty or whitespace-only titles are silently ignored.
pub fn rename_task(store: &mut Signal<AppStore>, task_id: TaskID, new_title: String) {
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

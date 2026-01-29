use dioxus::prelude::*;
use tasklens_core::types::{TaskID, TaskStatus, TunnelState};
use tasklens_store::actions::{Action, TaskUpdates};
use tasklens_store::store::AppStore;

#[derive(Clone, Copy, PartialEq)]
pub struct TaskController {
    store: Signal<AppStore>,
    load_error: Signal<Option<String>>,
    tunnel_state: Memo<TunnelState>,
}

pub fn use_task_controller() -> TaskController {
    let store = use_context::<Signal<AppStore>>();
    let load_error = use_context::<Signal<Option<String>>>();
    // We expect this to be provided by App
    let tunnel_state = use_context::<Memo<TunnelState>>();

    TaskController {
        store,
        load_error,
        tunnel_state,
    }
}

impl TaskController {
    pub fn create(&self, parent_id: Option<TaskID>, title: String) -> Option<TaskID> {
        let id = TaskID::new();
        let action = Action::CreateTask {
            id: id.clone(),
            parent_id,
            title,
        };
        let mut store = self.store;
        let mut load_error = self.load_error;
        if let Err(e) = store.write().dispatch(action) {
            tracing::error!("Failed to create task: {}", e);
            load_error.set(Some(e.to_string()));
            None
        } else {
            Some(id)
        }
    }

    pub fn update(&self, id: TaskID, updates: TaskUpdates) {
        let action = Action::UpdateTask { id, updates };
        let mut store = self.store;
        let mut load_error = self.load_error;
        if let Err(e) = store.write().dispatch(action) {
            tracing::error!("Failed to update task: {}", e);
            load_error.set(Some(e.to_string()));
        }
    }

    pub fn rename(&self, id: TaskID, new_title: String) {
        self.update(
            id,
            TaskUpdates {
                title: Some(new_title),
                ..Default::default()
            },
        );
    }

    pub fn delete(&self, id: TaskID) {
        let action = Action::DeleteTask { id };
        let mut store = self.store;
        let mut load_error = self.load_error;
        if let Err(e) = store.write().dispatch(action) {
            tracing::error!("Failed to delete task: {}", e);
            load_error.set(Some(e.to_string()));
        }
    }

    pub fn move_item(&self, id: TaskID, new_parent_id: Option<TaskID>) {
        let action = Action::MoveTask { id, new_parent_id };
        let mut store = self.store;
        let mut load_error = self.load_error;
        if let Err(e) = store.write().dispatch(action) {
            tracing::error!("Failed to move task: {}", e);
            load_error.set(Some(e.to_string()));
        }
    }

    pub fn toggle(&self, id: TaskID) {
        // Use memoized state to avoid hydration
        let state = self.tunnel_state.read();
        let current_status = state.tasks.get(&id).map(|t| t.status);
        let mut store = self.store;
        let mut load_error = self.load_error;

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

    pub fn indent(&self, id: TaskID) {
        let new_parent_opt = {
            let state = self.tunnel_state.read();

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
            self.move_item(id, Some(new_parent));
        }
    }

    pub fn outdent(&self, id: TaskID) {
        let (should_move, new_parent_id) = {
            let state = self.tunnel_state.read();
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
            self.move_item(id, new_parent_id);
        }
    }

    pub fn refresh_lifecycle(&self) {
        let current_time = chrono::Utc::now().timestamp_millis();
        let action = Action::RefreshLifecycle { current_time };
        let mut store = self.store;
        let mut load_error = self.load_error;
        if let Err(e) = store.write().dispatch(action) {
            tracing::error!("Failed to refresh lifecycle: {}", e);
            load_error.set(Some(e.to_string()));
        }
    }
}

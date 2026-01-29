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
    fn dispatch_and_log_error(&self, action: Action, context: &'static str) -> anyhow::Result<()> {
        let mut store = self.store;
        let mut load_error = self.load_error;
        let result = store.write().dispatch(action);
        if let Err(e) = &result {
            tracing::error!("{}: {}", context, e);
            load_error.set(Some(e.to_string()));
        }
        result
    }

    pub fn create(&self, parent_id: Option<TaskID>, title: String) -> Option<TaskID> {
        let id = TaskID::new();
        let action = Action::CreateTask {
            id: id.clone(),
            parent_id,
            title,
        };
        if self
            .dispatch_and_log_error(action, "Failed to create task")
            .is_ok()
        {
            Some(id)
        } else {
            None
        }
    }

    pub fn update(&self, id: TaskID, updates: TaskUpdates) {
        let action = Action::UpdateTask { id, updates };
        let _ = self.dispatch_and_log_error(action, "Failed to update task");
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
        let _ = self.dispatch_and_log_error(action, "Failed to delete task");
    }

    pub fn move_item(&self, id: TaskID, new_parent_id: Option<TaskID>) {
        let action = Action::MoveTask { id, new_parent_id };
        let _ = self.dispatch_and_log_error(action, "Failed to move task");
    }

    pub fn toggle(&self, id: TaskID) {
        // Use memoized state to avoid hydration
        let state = self.tunnel_state.read();
        let current_status = state.tasks.get(&id).map(|t| t.status);

        if let Some(status) = current_status {
            match status {
                TaskStatus::Pending => {
                    let current_time = chrono::Utc::now().timestamp_millis();
                    let action = Action::CompleteTask {
                        id: id.clone(),
                        current_time,
                    };
                    let _ = self.dispatch_and_log_error(action, "Failed to complete task");
                }
                TaskStatus::Done => {
                    let action = Action::UpdateTask {
                        id: id.clone(),
                        updates: TaskUpdates {
                            status: Some(TaskStatus::Pending),
                            ..Default::default()
                        },
                    };
                    let _ = self.dispatch_and_log_error(action, "Failed to toggle task status");
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
        let _ = self.dispatch_and_log_error(action, "Failed to refresh lifecycle");
    }
}

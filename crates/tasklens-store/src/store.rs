pub use crate::actions::{Action, TaskUpdates};
#[cfg(target_arch = "wasm32")]
use crate::storage::IndexedDbStorage;
use anyhow::{Result, anyhow};
use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile};
use std::collections::HashMap;

use tasklens_core::types::{TaskStatus, TunnelState};

/// A wrapper around the Automerge CRDT document.
///
/// This struct manages the application's state using Automerge for CRDT-based
/// persistence and synchronization.
#[derive(Clone, Debug)]
pub struct AppStore {
    /// The underlying Automerge document backend.
    pub doc: AutoCommit,
}

impl AppStore {
    /// Creates a new, empty AppStore.
    pub fn new() -> Self {
        let doc = AutoCommit::new();
        Self { doc }
    }

    /// Initialize the store with default state if empty.
    pub fn init(&mut self) -> Result<()> {
        let current_state: Result<TunnelState, _> = hydrate(&self.doc);
        if current_state.is_err() || current_state.as_ref().unwrap().tasks.is_empty() {
            let initial_state = TunnelState {
                next_task_id: 1.0,
                next_place_id: 1.0,
                tasks: HashMap::new(),
                places: HashMap::new(),
                root_task_ids: Vec::new(),
            };
            reconcile(&mut self.doc, &initial_state)
                .map_err(|e| anyhow!("Init reconciliation failed: {}", e))?;
        }
        Ok(())
    }

    /// Hydrates a Rust struct from the Automerge document.
    pub fn hydrate<T: autosurgeon::Hydrate>(&self) -> Result<T, autosurgeon::HydrateError> {
        autosurgeon::hydrate(&self.doc)
    }

    /// Reconciles a Rust struct with the Automerge document.
    pub fn reconcile<T: autosurgeon::Reconcile>(
        &mut self,
        data: &T,
    ) -> Result<(), autosurgeon::ReconcileError> {
        autosurgeon::reconcile(&mut self.doc, data)
    }

    pub fn get_state(&self) -> Result<TunnelState> {
        hydrate(&self.doc).map_err(|e| anyhow!("Hydration failed: {}", e))
    }

    pub fn dispatch(&mut self, action: Action) -> Result<()> {
        let mut state: TunnelState = self.get_state()?;

        match action {
            Action::CreateTask { parent_id, title } => {
                let parent = parent_id.as_ref().and_then(|pid| state.tasks.get(pid));
                let task = tasklens_core::create_new_task(title, parent);
                let id = task.id.clone();
                state.tasks.insert(id.clone(), task);
                if let Some(pid) = parent_id
                    && let Some(parent) = state.tasks.get_mut(&pid)
                {
                    parent.child_task_ids.push(id);
                } else {
                    state.root_task_ids.push(id);
                }
            }
            Action::UpdateTask { id, updates } => {
                if let Some(task) = state.tasks.get_mut(&id) {
                    if let Some(title) = updates.title {
                        task.title = title;
                    }
                    if let Some(status) = updates.status {
                        task.status = status;
                    }
                    if let Some(place_id) = updates.place_id {
                        task.place_id = place_id;
                    }
                    if let Some(due_date) = updates.due_date {
                        task.schedule.due_date = due_date;
                    }
                    if let Some(schedule_type) = updates.schedule_type {
                        task.schedule.schedule_type = schedule_type;
                    }
                    if let Some(lead_time) = updates.lead_time {
                        task.schedule.lead_time = lead_time;
                    }
                    if let Some(repeat_config) = updates.repeat_config {
                        task.repeat_config = repeat_config;
                    }
                }
            }
            Action::DeleteTask { id } => {
                if let Some(task) = state.tasks.remove(&id) {
                    if let Some(pid) = task.parent_id {
                        if let Some(parent) = state.tasks.get_mut(&pid) {
                            parent.child_task_ids.retain(|cid| cid != &id);
                        }
                    } else {
                        state.root_task_ids.retain(|rid| rid != &id);
                    }
                }
            }
            Action::CompleteTask { id, current_time } => {
                if let Some(task) = state.tasks.get_mut(&id) {
                    task.status = TaskStatus::Done;
                    task.last_completed_at = Some(current_time);
                }
            }
            Action::MoveTask { id, new_parent_id } => {
                let old_parent_id = state.tasks.get(&id).and_then(|t| t.parent_id.clone());

                // Remove from old parent or root
                if let Some(opid) = old_parent_id {
                    if let Some(parent) = state.tasks.get_mut(&opid) {
                        parent.child_task_ids.retain(|cid| cid != &id);
                    }
                } else {
                    state.root_task_ids.retain(|rid| rid != &id);
                }

                // Add to new parent or root
                if let Some(npid) = new_parent_id.clone() {
                    if let Some(parent) = state.tasks.get_mut(&npid) {
                        parent.child_task_ids.push(id.clone());
                    }
                } else {
                    state.root_task_ids.push(id.clone());
                }

                // Update task's parent_id
                if let Some(task) = state.tasks.get_mut(&id) {
                    task.parent_id = new_parent_id;
                }
            }
            Action::RefreshLifecycle { current_time } => {
                tasklens_core::domain::lifecycle::acknowledge_completed_tasks(&mut state);
                tasklens_core::domain::routine_tasks::wake_up_routine_tasks(
                    &mut state,
                    current_time,
                );
            }
        }

        reconcile(&mut self.doc, &state)
            .map_err(|e| anyhow!("Dispatch reconciliation failed: {}", e))?;
        Ok(())
    }

    /// Exports the current Automerge document state as a binary blob.
    pub fn export_save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Loads the persisted state from the browser's IndexedDB.
    #[cfg(target_arch = "wasm32")]
    pub async fn load_from_db() -> Result<Option<Vec<u8>>> {
        IndexedDbStorage::load_from_db().await
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn load_from_db() -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    /// Persists the current state to the browser's IndexedDB.
    #[cfg(target_arch = "wasm32")]
    pub async fn save_to_db(bytes: Vec<u8>) -> Result<()> {
        IndexedDbStorage::save_to_db(bytes).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn save_to_db(_bytes: Vec<u8>) -> Result<()> {
        Ok(())
    }

    /// Replaces the current backend with one loaded from the provided bytes.
    pub fn load_from_bytes(&mut self, bytes: Vec<u8>) {
        match AutoCommit::load(&bytes) {
            Ok(doc) => self.doc = doc,
            Err(e) => tracing::error!("Failed to load returned bytes into AutoCommit: {:?}", e),
        }
    }

    /// Imports incremental changes from the server.
    pub fn import_changes(&mut self, changes: Vec<u8>) {
        if let Err(e) = self.doc.load_incremental(&changes) {
            tracing::error!("Failed to load incremental changes: {:?}", e);
        }
    }

    /// Gets the changes made since the last sync/save.
    pub fn get_recent_changes(&mut self) -> Option<Vec<u8>> {
        let changes = self.doc.save_incremental();
        if changes.is_empty() {
            None
        } else {
            Some(changes)
        }
    }
}

impl Default for AppStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_init() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let state = store.get_state().unwrap();
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn test_dispatch_create() {
        let mut store = AppStore::new();
        store.init().unwrap();

        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "Test Task".to_string(),
            })
            .unwrap();

        let state = store.get_state().unwrap();
        assert_eq!(state.tasks.len(), 1);
        let task = state.tasks.values().next().unwrap();
        assert_eq!(task.title, "Test Task");
        assert!(state.root_task_ids.contains(&task.id));
    }

    #[test]
    fn test_dispatch_update() {
        let mut store = AppStore::new();
        store.init().unwrap();

        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "Original".to_string(),
            })
            .unwrap();

        let id = store.get_state().unwrap().root_task_ids[0].clone();

        store
            .dispatch(Action::UpdateTask {
                id: id.clone(),
                updates: crate::actions::TaskUpdates {
                    title: Some("Updated".to_string()),
                    status: Some(TaskStatus::Done),
                    ..Default::default()
                },
            })
            .unwrap();

        let state = store.get_state().unwrap();
        let task = state.tasks.get(&id).unwrap();
        assert_eq!(task.title, "Updated");
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn test_dispatch_delete() {
        let mut store = AppStore::new();
        store.init().unwrap();

        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "To Delete".to_string(),
            })
            .unwrap();

        let id = store.get_state().unwrap().root_task_ids[0].clone();
        store
            .dispatch(Action::DeleteTask { id: id.clone() })
            .unwrap();

        let state = store.get_state().unwrap();
        assert!(state.tasks.is_empty());
        assert!(state.root_task_ids.is_empty());
    }

    #[test]
    fn test_dispatch_complete() {
        let mut store = AppStore::new();
        store.init().unwrap();

        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "To Complete".to_string(),
            })
            .unwrap();

        let id = store.get_state().unwrap().root_task_ids[0].clone();
        store
            .dispatch(Action::CompleteTask {
                id: id.clone(),
                current_time: 100.0,
            })
            .unwrap();

        let state = store.get_state().unwrap();
        assert_eq!(state.tasks.get(&id).unwrap().status, TaskStatus::Done);
    }

    #[test]
    fn test_dispatch_move() {
        let mut store = AppStore::new();
        store.init().unwrap();

        // Create Parent
        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "Parent".to_string(),
            })
            .unwrap();
        let parent_id = store.get_state().unwrap().root_task_ids[0].clone();

        // Create Child as root task initially
        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "Child".to_string(),
            })
            .unwrap();
        let child_id = store
            .get_state()
            .unwrap()
            .root_task_ids
            .iter()
            .find(|&id| id != &parent_id)
            .unwrap()
            .clone();

        // Move Child under Parent
        store
            .dispatch(Action::MoveTask {
                id: child_id.clone(),
                new_parent_id: Some(parent_id.clone()),
            })
            .unwrap();

        let state = store.get_state().unwrap();
        assert_eq!(state.root_task_ids.len(), 1);
        assert_eq!(state.root_task_ids[0], parent_id);

        let parent = state.tasks.get(&parent_id).unwrap();
        assert!(parent.child_task_ids.contains(&child_id));

        let child = state.tasks.get(&child_id).unwrap();
        assert_eq!(child.parent_id, Some(parent_id));
    }

    #[test]
    fn test_dispatch_refresh_lifecycle() {
        let mut store = AppStore::new();
        store.init().unwrap();

        // Create a Done task
        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "To Acknowledge".to_string(),
            })
            .unwrap();
        let id = store.get_state().unwrap().root_task_ids[0].clone();
        store
            .dispatch(Action::CompleteTask {
                id: id.clone(),
                current_time: 100.0,
            })
            .unwrap();

        assert!(!store.get_state().unwrap().tasks[&id].is_acknowledged);

        // Refresh
        store
            .dispatch(Action::RefreshLifecycle {
                current_time: 100.0,
            })
            .unwrap();

        assert!(store.get_state().unwrap().tasks[&id].is_acknowledged);
    }

    #[test]
    fn test_dispatch_refresh_lifecycle_with_routine() {
        let mut store = AppStore::new();
        store.init().unwrap();

        // Create a Routinely task and complete it
        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "Routine".to_string(),
            })
            .unwrap();
        let id = store.get_state().unwrap().root_task_ids[0].clone();

        // Manually update it to be Routinely with short interval
        let mut state = store.get_state().unwrap();
        let task = state.tasks.get_mut(&id).unwrap();
        task.status = TaskStatus::Done;
        task.schedule.schedule_type = tasklens_core::types::ScheduleType::Routinely;
        task.schedule.lead_time = Some(100.0);
        task.repeat_config = Some(tasklens_core::types::RepeatConfig {
            frequency: tasklens_core::types::Frequency::Daily,
            interval: 1.0,
        });
        task.last_completed_at = Some(1000.0);
        store.reconcile(&state).unwrap();

        // Next due: 1000 + (24*60*60*1000) = 86,401,000
        // Wake up time: 86,401,000 - 100 = 86,400,900

        // Refresh before wake up
        store
            .dispatch(Action::RefreshLifecycle {
                current_time: 1000.0,
            })
            .unwrap();
        assert_eq!(
            store.get_state().unwrap().tasks[&id].status,
            TaskStatus::Done
        );

        // Refresh after wake up
        store
            .dispatch(Action::RefreshLifecycle {
                current_time: 86401000.0,
            })
            .unwrap();
        assert_eq!(
            store.get_state().unwrap().tasks[&id].status,
            TaskStatus::Pending
        );
    }
}

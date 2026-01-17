pub use crate::actions::{Action, TaskUpdates};
use crate::doc_id::{DocumentId, TaskLensUrl};
#[cfg(target_arch = "wasm32")]
use crate::storage::{ActiveDocStorage, IndexedDbStorage};
use anyhow::{Result, anyhow};
use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile};
use std::collections::HashMap;

use tasklens_core::types::{DocMetadata, TaskStatus, TunnelState};

/// A manager for Automerge documents and persistence.
///
/// This struct implements a Repo-like pattern, managing the current
/// document and providing methods for document lifecycle management.
#[derive(Clone, Debug)]
pub struct AppStore {
    /// The ID of the currently loaded document.
    pub current_id: DocumentId,
    /// The underlying Automerge document backend.
    pub doc: AutoCommit,
}

impl AppStore {
    /// Creates a new AppStore with a fresh document.
    pub fn new() -> Self {
        let current_id = DocumentId::new();
        let doc = AutoCommit::new();
        Self { current_id, doc }
    }

    /// Initialize the current document with default state if empty.
    pub fn init(&mut self) -> Result<()> {
        let current_state: Result<TunnelState, _> = hydrate(&self.doc);
        if current_state.is_err() || current_state.as_ref().unwrap().tasks.is_empty() {
            let initial_state = TunnelState {
                next_task_id: 1.0,
                next_place_id: 1.0,
                tasks: HashMap::new(),
                places: HashMap::new(),
                root_task_ids: Vec::new(),
                metadata: None,
            };
            reconcile(&mut self.doc, &initial_state)
                .map_err(|e| anyhow!("Init reconciliation failed: {}", e))?;
        }
        Ok(())
    }

    /// Resets the document to an empty state.
    pub fn reset(&mut self) -> Result<()> {
        self.doc = AutoCommit::new();
        self.init()
    }

    /// Creates a new document and switches to it.
    pub fn create_new(&mut self) -> Result<DocumentId> {
        let new_id = DocumentId::new();
        let mut new_doc = AutoCommit::new();

        let initial_state = TunnelState {
            next_task_id: 1.0,
            next_place_id: 1.0,
            tasks: HashMap::new(),
            places: HashMap::new(),
            root_task_ids: Vec::new(),
            metadata: Some(DocMetadata {
                automerge_url: Some(TaskLensUrl::from(new_id.clone()).to_string()),
            }),
        };
        reconcile(&mut new_doc, &initial_state)
            .map_err(|e| anyhow!("New doc reconciliation failed: {}", e))?;

        self.current_id = new_id.clone();
        self.doc = new_doc;

        #[cfg(target_arch = "wasm32")]
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(new_id.clone()));

        Ok(new_id)
    }

    /// Switches to an existing document by ID.
    #[cfg(target_arch = "wasm32")]
    pub async fn switch_doc(&mut self, id: DocumentId) -> Result<()> {
        if let Some(bytes) = IndexedDbStorage::load_from_db(&id).await? {
            match AutoCommit::load(&bytes) {
                Ok(doc) => {
                    self.current_id = id.clone();
                    self.doc = doc;
                    ActiveDocStorage::save_active_url(&TaskLensUrl::from(id));
                    Ok(())
                }
                Err(e) => Err(anyhow!("Failed to load doc bytes: {:?}", e)),
            }
        } else {
            Err(anyhow!("Document not found: {}", id))
        }
    }

    /// Deletes a document.
    #[cfg(target_arch = "wasm32")]
    pub async fn delete_doc(&mut self, id: DocumentId) -> Result<()> {
        IndexedDbStorage::delete_doc(&id).await?;
        if self.current_id == id {
            // If we deleted the current doc, create a new one
            self.create_new()?;
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn switch_doc(&mut self, _id: DocumentId) -> Result<()> {
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn delete_doc(&mut self, _id: DocumentId) -> Result<()> {
        Ok(())
    }

    /// Hydrates a Rust struct from the Automerge document.
    pub fn hydrate<T: autosurgeon::Hydrate>(&self) -> Result<T> {
        autosurgeon::hydrate(&self.doc).map_err(|e| anyhow!("Hydration failed: {}", e))
    }

    /// Reconciles a Rust struct with the Automerge document.
    pub fn reconcile<T: autosurgeon::Reconcile>(
        &mut self,
        data: &T,
    ) -> Result<(), autosurgeon::ReconcileError> {
        autosurgeon::reconcile(&mut self.doc, data)
    }

    pub fn dispatch(&mut self, action: Action) -> Result<()> {
        let mut state: TunnelState = self.hydrate()?;

        match action {
            Action::CreateTask {
                id,
                parent_id,
                title,
            } => {
                let parent = parent_id.as_ref().and_then(|pid| state.tasks.get(pid));
                let task = tasklens_core::create_new_task(id.clone(), title, parent);
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
                    if let Some(is_seq) = updates.is_sequential {
                        task.is_sequential = is_seq;
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

    /// Loads the persisted state from the browser's IndexedDB for a given ID.
    #[cfg(target_arch = "wasm32")]
    pub async fn load_from_db(doc_id: &DocumentId) -> Result<Option<Vec<u8>>> {
        IndexedDbStorage::load_from_db(doc_id).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn load_from_db(_doc_id: &DocumentId) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    /// Persists the current state to the browser's IndexedDB.
    #[cfg(target_arch = "wasm32")]
    pub async fn save_to_db(&mut self) -> Result<()> {
        let bytes = self.doc.save();
        IndexedDbStorage::save_to_db(&self.current_id, bytes).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn save_to_db(&mut self) -> Result<()> {
        Ok(())
    }

    /// Static helper to load the active document ID preference.
    #[cfg(target_arch = "wasm32")]
    pub fn load_active_doc_id() -> Option<DocumentId> {
        ActiveDocStorage::load_active_url().map(|url| url.document_id)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_active_doc_id() -> Option<DocumentId> {
        None
    }

    /// Static helper to save the active document ID preference.
    #[cfg(target_arch = "wasm32")]
    pub fn save_active_doc_id(id: &DocumentId) {
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(id.clone()));
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_active_doc_id(_id: &DocumentId) {}

    /// Static helper to persist raw document data.
    #[cfg(target_arch = "wasm32")]
    pub async fn save_doc_data_async(id: &DocumentId, data: Vec<u8>) -> Result<()> {
        IndexedDbStorage::save_to_db(id, data).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn save_doc_data_async(_id: &DocumentId, _data: Vec<u8>) -> Result<()> {
        Ok(())
    }

    /// Replaces the current backend with one loaded from the provided bytes.
    pub fn load_from_bytes(&mut self, bytes: Vec<u8>) {
        match AutoCommit::load(&bytes) {
            Ok(doc) => self.doc = doc,
            Err(e) => tracing::error!("Failed to load returned bytes into AutoCommit: {:?}", e),
        }
    }

    /// Imports a document from bytes, preserving its identity if present in metadata.
    /// Returns the (possibly new) DocumentId.
    pub fn import_doc(&mut self, bytes: Vec<u8>) -> Result<DocumentId> {
        let doc = AutoCommit::load(&bytes)
            .map_err(|e| anyhow!("Failed to load Automerge doc: {:?}", e))?;

        let state: TunnelState =
            hydrate(&doc).map_err(|e| anyhow!("Failed to hydrate doc for import: {}", e))?;

        let doc_id = if let Some(meta) = &state.metadata
            && let Some(url_str) = &meta.automerge_url
            && let Ok(url) = url_str.parse::<TaskLensUrl>()
        {
            url.document_id
        } else {
            DocumentId::new()
        };

        self.doc = doc;
        self.current_id = doc_id.clone();

        // Ensure metadata is set/updated in the doc
        let mut new_state = state;
        new_state.metadata = Some(DocMetadata {
            automerge_url: Some(TaskLensUrl::from(doc_id.clone()).to_string()),
        });
        reconcile(&mut self.doc, &new_state)
            .map_err(|e| anyhow!("Failed to reconcile metadata after import: {}", e))?;

        #[cfg(target_arch = "wasm32")]
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(doc_id.clone()));

        Ok(doc_id)
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
    use tasklens_core::TaskID;

    use super::*;

    #[test]
    fn test_store_init() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let state: TunnelState = store.hydrate().unwrap();
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn test_dispatch_create() {
        let mut store = AppStore::new();
        store.init().unwrap();

        store
            .dispatch(Action::CreateTask {
                id: TaskID::new(),
                parent_id: None,
                title: "Test Task".to_string(),
            })
            .unwrap();

        let state: TunnelState = store.hydrate().unwrap();
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
                id: TaskID::new(),
                parent_id: None,
                title: "Original".to_string(),
            })
            .unwrap();

        let id = store.hydrate::<TunnelState>().unwrap().root_task_ids[0].clone();

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

        let state: TunnelState = store.hydrate().unwrap();
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
                id: TaskID::new(),
                parent_id: None,
                title: "To Delete".to_string(),
            })
            .unwrap();

        let id = store.hydrate::<TunnelState>().unwrap().root_task_ids[0].clone();
        store
            .dispatch(Action::DeleteTask { id: id.clone() })
            .unwrap();

        let state: TunnelState = store.hydrate().unwrap();
        assert!(state.tasks.is_empty());
        assert!(state.root_task_ids.is_empty());
    }

    #[test]
    fn test_dispatch_complete() {
        let mut store = AppStore::new();
        store.init().unwrap();

        store
            .dispatch(Action::CreateTask {
                id: TaskID::new(),
                parent_id: None,
                title: "To Complete".to_string(),
            })
            .unwrap();

        let id = store.hydrate::<TunnelState>().unwrap().root_task_ids[0].clone();
        store
            .dispatch(Action::CompleteTask {
                id: id.clone(),
                current_time: 100.0,
            })
            .unwrap();

        let state: TunnelState = store.hydrate().unwrap();
        assert_eq!(state.tasks.get(&id).unwrap().status, TaskStatus::Done);
    }

    #[test]
    fn test_dispatch_move() {
        let mut store = AppStore::new();
        store.init().unwrap();

        // Create Parent
        store
            .dispatch(Action::CreateTask {
                id: TaskID::new(),
                parent_id: None,
                title: "Parent".to_string(),
            })
            .unwrap();
        let parent_id = store.hydrate::<TunnelState>().unwrap().root_task_ids[0].clone();

        // Create Child as root task initially
        store
            .dispatch(Action::CreateTask {
                id: TaskID::new(),
                parent_id: None,
                title: "Child".to_string(),
            })
            .unwrap();
        let child_id = store
            .hydrate::<TunnelState>()
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

        let state: TunnelState = store.hydrate().unwrap();
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
                id: TaskID::new(),
                parent_id: None,
                title: "To Acknowledge".to_string(),
            })
            .unwrap();
        let id = store.hydrate::<TunnelState>().unwrap().root_task_ids[0].clone();
        store
            .dispatch(Action::CompleteTask {
                id: id.clone(),
                current_time: 100.0,
            })
            .unwrap();

        assert!(!store.hydrate::<TunnelState>().unwrap().tasks[&id].is_acknowledged);

        // Refresh
        store
            .dispatch(Action::RefreshLifecycle {
                current_time: 100.0,
            })
            .unwrap();

        assert!(store.hydrate::<TunnelState>().unwrap().tasks[&id].is_acknowledged);
    }

    #[test]
    fn test_dispatch_refresh_lifecycle_with_routine() {
        let mut store = AppStore::new();
        store.init().unwrap();

        // Create a Routinely task and complete it
        store
            .dispatch(Action::CreateTask {
                id: TaskID::new(),
                parent_id: None,
                title: "Routine".to_string(),
            })
            .unwrap();
        let id = store.hydrate::<TunnelState>().unwrap().root_task_ids[0].clone();

        // Manually update it to be Routinely with short interval
        let mut state = store.hydrate::<TunnelState>().unwrap();
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
            store.hydrate::<TunnelState>().unwrap().tasks[&id].status,
            TaskStatus::Done
        );

        // Refresh after wake up
        store
            .dispatch(Action::RefreshLifecycle {
                current_time: 86401000.0,
            })
            .unwrap();
        assert_eq!(
            store.hydrate::<TunnelState>().unwrap().tasks[&id].status,
            TaskStatus::Pending
        );
    }
}

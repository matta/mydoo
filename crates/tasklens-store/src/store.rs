pub use crate::actions::{Action, TaskUpdates};
use crate::doc_id::{DocumentId, TaskLensUrl};
#[cfg(target_arch = "wasm32")]
use crate::storage::{ActiveDocStorage, IndexedDbStorage};
use anyhow::{Result, anyhow};
use automerge::transaction::Transactable;
use automerge::{AutoCommit, ReadDoc};
use autosurgeon::{hydrate, reconcile};
use std::collections::HashMap;

use tasklens_core::types::{
    DocMetadata, PersistedTask, TaskID, TaskStatus, TunnelState, hydrate_optional_task_id,
};

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
                next_task_id: 1,
                next_place_id: 1,
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
            next_task_id: 1,
            next_place_id: 1,
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
    pub fn expensive_reconcile<T: autosurgeon::Reconcile>(
        &mut self,
        data: &T,
    ) -> Result<(), autosurgeon::ReconcileError> {
        reconcile(&mut self.doc, data)
    }

    /// Dispatches an action to modify the application state.
    ///
    /// This method is the primary entry point for state mutations. It implements a
    /// functional core pattern using Autosurgeon's hydration and reconciliation capabilities:
    ///
    /// 1. **Hydrate**: The current Automerge document state is hydrated into a native Rust
    ///    `TunnelState` struct.
    /// 2. **Mutate**: The `Action` is applied to the `TunnelState`, modifying tasks,
    ///    relationships, or other domain models in memory.
    /// 3. **Reconcile**: The modified `TunnelState` is reconciled back into the Automerge
    ///    document. This efficiently calculates the diffs and applies them as Automerge
    ///    operations, ensuring history and conflict resolution are preserved.
    ///
    /// # Arguments
    ///
    /// * `action` - The `Action` to perform, encapsulating the intent of the mutation
    ///   (e.g., creating a task, updating a field, moving a task).
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the action was successfully applied and reconciled, or an `Error`
    /// if hydration or reconciliation failed.
    pub fn dispatch(&mut self, action: Action) -> Result<()> {
        match action {
            Action::CreateTask {
                id,
                parent_id,
                title,
            } => self.handle_create_task(id, parent_id, title),
            Action::UpdateTask { id, updates } => self.handle_update_task(id, updates),
            Action::DeleteTask { id } => self.handle_delete_task(id),
            Action::CompleteTask { id, current_time } => {
                self.handle_complete_task(id, current_time)
            }
            Action::MoveTask { id, new_parent_id } => self.handle_move_task(id, new_parent_id),
            Action::RefreshLifecycle { current_time } => {
                self.handle_refresh_lifecycle(current_time)
            }
        }
    }

    fn handle_create_task(
        &mut self,
        id: TaskID,
        parent_id: Option<TaskID>,
        title: String,
    ) -> Result<()> {
        // 1. Get Tasks Map.
        let tasks_obj_id = ensure_path(&mut self.doc, &automerge::ROOT, vec!["tasks"])?;

        // 2. Resolve parent task.
        let parent = if let Some(pid) = &parent_id {
            let p: Option<PersistedTask> =
                autosurgeon::hydrate_prop(&self.doc, &tasks_obj_id, pid.as_str())
                    .map_err(|e| anyhow!("Failed to hydrate parent task: {}", e))?;
            p
        } else {
            None
        };

        // 3. Create the new task struct.
        let task = tasklens_core::create_new_task(id.clone(), title, parent.as_ref());

        // 4. Reconcile the new task.
        autosurgeon::reconcile_prop(&mut self.doc, &tasks_obj_id, id.as_str(), &task)
            .map_err(|e| anyhow!("Failed to reconcile new task: {}", e))?;

        // 5. Update parent's child list or root list.
        if let Some(pid) = parent_id {
            // Get parent object ID.
            let parent_obj_id = ensure_path(&mut self.doc, &tasks_obj_id, vec![pid.as_str()])?;

            // Hydrate current children list.
            let mut child_ids: Vec<TaskID> =
                autosurgeon::hydrate_prop(&self.doc, &parent_obj_id, "childTaskIds")
                    .unwrap_or_default();

            child_ids.push(id);

            // Reconcile updated children list.
            autosurgeon::reconcile_prop(&mut self.doc, &parent_obj_id, "childTaskIds", &child_ids)
                .map_err(|e| anyhow!("Failed to reconcile child ids: {}", e))?;
        } else {
            // Update root task list.
            let mut root_ids: Vec<TaskID> =
                autosurgeon::hydrate_prop(&self.doc, automerge::ROOT, "rootTaskIds")
                    .unwrap_or_default();

            root_ids.push(id);

            autosurgeon::reconcile_prop(&mut self.doc, automerge::ROOT, "rootTaskIds", &root_ids)
                .map_err(|e| anyhow!("Failed to reconcile root task ids: {}", e))?;
        }

        Ok(())
    }

    fn handle_update_task(&mut self, id: TaskID, updates: TaskUpdates) -> Result<()> {
        let tasks_obj_id = ensure_path(&mut self.doc, &automerge::ROOT, vec!["tasks"])?;

        let task_obj_id = ensure_path(&mut self.doc, &tasks_obj_id, vec![id.as_str()])
            .map_err(|_| anyhow!("Task not found: {}", id))?;

        if let Some(title) = updates.title {
            autosurgeon::reconcile_prop(&mut self.doc, &task_obj_id, "title", title)
                .map_err(|e| anyhow!("Failed to update title: {}", e))?;
        }
        if let Some(status) = updates.status {
            autosurgeon::reconcile_prop(&mut self.doc, &task_obj_id, "status", status)
                .map_err(|e| anyhow!("Failed to update status: {}", e))?;
        }
        if let Some(place_id_update) = updates.place_id {
            autosurgeon::reconcile_prop(&mut self.doc, &task_obj_id, "placeId", place_id_update)
                .map_err(|e| anyhow!("Failed to update placeId: {}", e))?;
        }

        if updates.due_date.is_some()
            || updates.schedule_type.is_some()
            || updates.lead_time.is_some()
        {
            let schedule_obj_id = ensure_path(&mut self.doc, &task_obj_id, vec!["schedule"])?;

            if let Some(due_date_update) = updates.due_date {
                autosurgeon::reconcile_prop(
                    &mut self.doc,
                    &schedule_obj_id,
                    "dueDate",
                    due_date_update,
                )
                .map_err(|e| anyhow!("Failed to update dueDate: {}", e))?;
            }
            if let Some(schedule_type) = updates.schedule_type {
                autosurgeon::reconcile_prop(&mut self.doc, &schedule_obj_id, "type", schedule_type)
                    .map_err(|e| anyhow!("Failed to update schedule type: {}", e))?;
            }
            if let Some(lead_time_update) = updates.lead_time {
                autosurgeon::reconcile_prop(
                    &mut self.doc,
                    &schedule_obj_id,
                    "leadTime",
                    lead_time_update,
                )
                .map_err(|e| anyhow!("Failed to update leadTime: {}", e))?;
            }
        }

        if let Some(repeat_config_update) = updates.repeat_config {
            autosurgeon::reconcile_prop(
                &mut self.doc,
                &task_obj_id,
                "repeatConfig",
                repeat_config_update,
            )
            .map_err(|e| anyhow!("Failed to update repeatConfig: {}", e))?;
        }
        if let Some(is_seq) = updates.is_sequential {
            autosurgeon::reconcile_prop(&mut self.doc, &task_obj_id, "isSequential", is_seq)
                .map_err(|e| anyhow!("Failed to update isSequential: {}", e))?;
        }

        Ok(())
    }

    // TODO: Optimize this to use surgical updates (reconcile_prop) similar to handle_create_task.
    // Currently uses full state hydration which is less efficient.
    fn handle_delete_task(&mut self, id: TaskID) -> Result<()> {
        let tasks_obj_id = ensure_path(&mut self.doc, &automerge::ROOT, vec!["tasks"])?;

        // 1. Get task object ID.
        let task_obj_id = match self.doc.get(&tasks_obj_id, id.as_str())? {
            Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
            _ => return Err(anyhow!("Task not found: {}", id)),
        };

        // 2. Hydrate parent_id to know where to remove it from.
        let parent_id: Option<TaskID> = hydrate_optional_task_id(
            &self.doc,
            &task_obj_id,
            autosurgeon::Prop::Key("parentId".into()),
        )
        .map_err(|e| anyhow!("Failed to hydrate parentId: {}", e))?;

        // 3. Remove from parent's children or rootTaskIds.
        if let Some(pid) = parent_id {
            // Get parent object ID.
            let parent_obj_id = ensure_path(&mut self.doc, &tasks_obj_id, vec![pid.as_str()])?;

            // Hydrate current children list.
            let mut child_ids: Vec<TaskID> =
                autosurgeon::hydrate_prop(&self.doc, &parent_obj_id, "childTaskIds")
                    .unwrap_or_default();

            child_ids.retain(|cid| cid != &id);

            // Reconcile updated children list.
            autosurgeon::reconcile_prop(&mut self.doc, &parent_obj_id, "childTaskIds", &child_ids)
                .map_err(|e| anyhow!("Failed to reconcile parent's childTaskIds: {}", e))?;
        } else {
            // Update root task list.
            let mut root_ids: Vec<TaskID> =
                autosurgeon::hydrate_prop(&self.doc, &automerge::ROOT, "rootTaskIds")
                    .unwrap_or_default();

            root_ids.retain(|rid| rid != &id);

            autosurgeon::reconcile_prop(&mut self.doc, &automerge::ROOT, "rootTaskIds", &root_ids)
                .map_err(|e| anyhow!("Failed to reconcile rootTaskIds: {}", e))?;
        }

        // 4. Delete the task object itself from the tasks map.
        self.doc
            .delete(&tasks_obj_id, id.as_str())
            .map_err(|e| anyhow!("Failed to delete task object: {}", e))?;

        Ok(())
    }

    // TODO: Optimize this to use surgical updates (reconcile_prop) similar to handle_create_task.
    // Currently uses full state hydration which is less efficient.
    fn handle_complete_task(&mut self, id: TaskID, current_time: i64) -> Result<()> {
        let mut state: TunnelState = self.hydrate()?;
        if let Some(task) = state.tasks.get_mut(&id) {
            task.status = TaskStatus::Done;
            task.last_completed_at = Some(current_time);
        }
        // TODO: use reconcile_prop()
        reconcile(&mut self.doc, &state)
            .map_err(|e| anyhow!("Dispatch reconciliation failed: {}", e))?;
        Ok(())
    }

    // TODO: Optimize this to use surgical updates (reconcile_prop) similar to handle_create_task.
    // Currently uses full state hydration which is less efficient.
    fn handle_move_task(&mut self, id: TaskID, new_parent_id: Option<TaskID>) -> Result<()> {
        let mut state: TunnelState = self.hydrate()?;
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
        // TODO: use reconcile_prop()
        reconcile(&mut self.doc, &state)
            .map_err(|e| anyhow!("Dispatch reconciliation failed: {}", e))?;
        Ok(())
    }

    // TODO: Optimize this to use surgical updates (reconcile_prop) similar to
    // handle_create_task. Currently uses full state hydration which is less
    // efficient. Note: this may be difficult or infaesible!
    fn handle_refresh_lifecycle(&mut self, current_time: i64) -> Result<()> {
        let mut state: TunnelState = self.hydrate()?;
        tasklens_core::domain::lifecycle::acknowledge_completed_tasks(&mut state);
        tasklens_core::domain::routine_tasks::wake_up_routine_tasks(&mut state, current_time);
        // TODO: use reconcile_prop()
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
        let metadata = DocMetadata {
            automerge_url: Some(TaskLensUrl::from(doc_id.clone()).to_string()),
        };
        autosurgeon::reconcile_prop(&mut self.doc, automerge::ROOT, "metadata", &metadata)
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

/// Helper to ensure a path of map objects exists in the document.
///
/// Returns the `ObjId` of the final object in the path.
/// Creates intermediate maps if they are missing.
fn ensure_path(
    doc: &mut automerge::AutoCommit,
    root: &automerge::ObjId,
    path: Vec<&str>,
) -> Result<automerge::ObjId> {
    let mut current = root.clone();
    for key in path {
        let val = doc.get(&current, key)?;
        current = match val {
            Some((automerge::Value::Object(_), id)) => id,
            None => doc.put_object(&current, key, automerge::ObjType::Map)?,
            _ => return Err(anyhow!("Path key '{}' is not an object", key)),
        };
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use automerge_test::{assert_doc, list, map};
    use tasklens_core::TaskID;

    use super::*;

    #[test]
    fn test_ensure_path() {
        let mut doc = AutoCommit::new();

        // 1. Ensure path on clean doc
        let id1 = ensure_path(&mut doc, &automerge::ROOT, vec!["a", "b", "c"]).unwrap();

        // Verify structure
        assert_doc!(
            &doc,
            map! {
                "a" => {
                    map! {
                        "b" => {
                            map! {
                                "c" => { map!{} }
                            }
                        }
                    }
                }
            }
        );

        // 2. Ensure existing path returns same ID
        let id2 = ensure_path(&mut doc, &automerge::ROOT, vec!["a", "b", "c"]).unwrap();
        assert_eq!(id1, id2);

        // 3. Ensure path with some existing parts
        let id3 = ensure_path(&mut doc, &automerge::ROOT, vec!["a", "b", "d"]).unwrap();
        assert_doc!(
            &doc,
            map! {
                "a" => {
                    map! {
                        "b" => {
                            map! {
                                "c" => { map!{} },
                                "d" => { map!{} }
                            }
                        }
                    }
                }
            }
        );
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_store_init() {
        let mut store = AppStore::new();
        store.init().unwrap();

        // Verify empty state structure
        assert_doc!(
            &store.doc,
            map! {
                "tasks" => { map!{} },
                "places" => { map!{} },
                "rootTaskIds" => { list![] },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );

        let state: TunnelState = store.hydrate().unwrap();
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn test_dispatch_create() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let task_id = TaskID::new();
        let task_id_str = task_id.to_string();

        store
            .dispatch(Action::CreateTask {
                id: task_id.clone(),
                parent_id: None,
                title: "Test Task".to_string(),
            })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        task_id_str.as_str() => {
                            map! {
                                "id" => { task_id_str.as_str() },
                                "title" => { "Test Task" },
                                "childTaskIds" => { list![] },
                                "status" => { "Pending" },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { automerge::ScalarValue::Null },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { task_id_str.as_str() }
                    ]
                },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null },
                "places" => { map!{} }
            }
        );

        let state: TunnelState = store.hydrate().unwrap();
        assert_eq!(state.tasks.len(), 1);
        let task = state.tasks.values().next().unwrap();
        assert_eq!(task.title, "Test Task");
        assert!(state.root_task_ids.contains(&task.id));
    }

    #[test]
    fn test_dispatch_create_with_parent() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let parent_id = TaskID::new();
        let parent_id_str = parent_id.to_string();
        let child1_id = TaskID::new();
        let child1_id_str = child1_id.to_string();
        let child2_id = TaskID::new();
        let child2_id_str = child2_id.to_string();

        store
            .dispatch(Action::CreateTask {
                id: parent_id.clone(),
                parent_id: None,
                title: "Parent".to_string(),
            })
            .unwrap();

        store
            .dispatch(Action::CreateTask {
                id: child1_id.clone(),
                parent_id: Some(parent_id.clone()),
                title: "Child 1".to_string(),
            })
            .unwrap();

        store
            .dispatch(Action::CreateTask {
                id: child2_id.clone(),
                parent_id: Some(parent_id.clone()),
                title: "Child 2".to_string(),
            })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        parent_id_str.as_str() => {
                            map! {
                                "id" => { parent_id_str.as_str() },
                                "title" => { "Parent" },
                                "childTaskIds" => {
                                    list![
                                        { child1_id_str.as_str() },
                                        { child2_id_str.as_str() }
                                    ]
                                },
                                "status" => { "Pending" },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { automerge::ScalarValue::Null },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        },
                        child1_id_str.as_str() => {
                            map! {
                                "id" => { child1_id_str.as_str() },
                                "title" => { "Child 1" },
                                "childTaskIds" => { list![] },
                                "status" => { "Pending" },
                                "notes" => { "" },
                                "parentId" => { parent_id_str.as_str() },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { automerge::ScalarValue::Null },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        },
                        child2_id_str.as_str() => {
                            map! {
                                "id" => { child2_id_str.as_str() },
                                "title" => { "Child 2" },
                                "childTaskIds" => { list![] },
                                "status" => { "Pending" },
                                "notes" => { "" },
                                "parentId" => { parent_id_str.as_str() },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { automerge::ScalarValue::Null },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { parent_id_str.as_str() }
                    ]
                },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null },
                "places" => { map!{} }
            }
        );

        let state: TunnelState = store.hydrate().unwrap();

        let parent = state.tasks.get(&parent_id).unwrap();
        assert_eq!(parent.child_task_ids.len(), 2);
        assert_eq!(parent.child_task_ids[0], child1_id);
        assert_eq!(parent.child_task_ids[1], child2_id);

        let child1 = state.tasks.get(&child1_id).unwrap();
        assert_eq!(child1.parent_id, Some(parent_id.clone()));

        let child2 = state.tasks.get(&child2_id).unwrap();
        assert_eq!(child2.parent_id, Some(parent_id.clone()));

        assert!(state.root_task_ids.contains(&parent_id));
        assert!(!state.root_task_ids.contains(&child1_id));
        assert!(!state.root_task_ids.contains(&child2_id));
    }

    #[test]
    fn test_dispatch_create_multiple_roots() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let root1_id = TaskID::new();
        let root2_id = TaskID::new();

        store
            .dispatch(Action::CreateTask {
                id: root1_id.clone(),
                parent_id: None,
                title: "Root 1".to_string(),
            })
            .unwrap();

        store
            .dispatch(Action::CreateTask {
                id: root2_id.clone(),
                parent_id: None,
                title: "Root 2".to_string(),
            })
            .unwrap();

        let state: TunnelState = store.hydrate().unwrap();
        assert_eq!(state.root_task_ids.len(), 2);
        assert_eq!(state.root_task_ids[0], root1_id);
        assert_eq!(state.root_task_ids[1], root2_id);
    }

    #[test]
    fn test_dispatch_update_all_fields() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let task_id = TaskID::new();

        store
            .dispatch(Action::CreateTask {
                id: task_id.clone(),
                parent_id: None,
                title: "Original".to_string(),
            })
            .unwrap();

        let place_id = tasklens_core::types::PlaceID::new();
        let repeat_config = tasklens_core::types::RepeatConfig {
            frequency: tasklens_core::types::Frequency::Daily,
            interval: 2,
        };

        store
            .dispatch(Action::UpdateTask {
                id: task_id.clone(),
                updates: crate::actions::TaskUpdates {
                    title: Some("Updated Title".to_string()),
                    status: Some(TaskStatus::Done),
                    place_id: Some(Some(place_id.clone())),
                    due_date: Some(Some(1234567890)),
                    schedule_type: Some(tasklens_core::types::ScheduleType::Routinely),
                    lead_time: Some(Some(3600)),
                    repeat_config: Some(Some(repeat_config.clone())),
                    is_sequential: Some(true),
                },
            })
            .unwrap();

        let state: TunnelState = store.hydrate().unwrap();
        let task = state.tasks.get(&task_id).unwrap();

        assert_eq!(task.title, "Updated Title");
        assert_eq!(task.status, TaskStatus::Done);
        assert_eq!(task.place_id, Some(place_id));
        assert_eq!(task.schedule.due_date, Some(1234567890));
        assert_eq!(
            task.schedule.schedule_type,
            tasklens_core::types::ScheduleType::Routinely
        );
        assert_eq!(task.schedule.lead_time, Some(3600));
        assert_eq!(task.repeat_config, Some(repeat_config));
        assert!(task.is_sequential);
    }

    #[test]
    fn test_dispatch_update() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let task_id = TaskID::new();
        let task_id_str = task_id.to_string();

        store
            .dispatch(Action::CreateTask {
                id: task_id.clone(),
                parent_id: None,
                title: "Original".to_string(),
            })
            .unwrap();

        store
            .dispatch(Action::UpdateTask {
                id: task_id.clone(),
                updates: crate::actions::TaskUpdates {
                    title: Some("Updated".to_string()),
                    status: Some(TaskStatus::Done),
                    ..Default::default()
                },
            })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        task_id_str.as_str() => {
                            map! {
                                "id" => { task_id_str.as_str() },
                                "title" => { "Updated" },
                                "status" => { "Done" },
                                "childTaskIds" => { list![] },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { automerge::ScalarValue::Null },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { task_id_str.as_str() }
                    ]
                },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );

        let state: TunnelState = store.hydrate().unwrap();
        let task = state.tasks.get(&task_id).unwrap();
        assert_eq!(task.title, "Updated");
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn test_dispatch_delete() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let task_id = TaskID::new();

        store
            .dispatch(Action::CreateTask {
                id: task_id.clone(),
                parent_id: None,
                title: "To Delete".to_string(),
            })
            .unwrap();

        store
            .dispatch(Action::DeleteTask {
                id: task_id.clone(),
            })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => { map!{} },
                "rootTaskIds" => { list![] },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );

        let state: TunnelState = store.hydrate().unwrap();
        assert!(state.tasks.is_empty());
        assert!(state.root_task_ids.is_empty());
    }

    #[test]
    fn test_dispatch_delete_with_parent() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let parent_id = TaskID::new();
        let child_id = TaskID::new();

        store
            .dispatch(Action::CreateTask {
                id: parent_id.clone(),
                parent_id: None,
                title: "Parent".to_string(),
            })
            .unwrap();

        store
            .dispatch(Action::CreateTask {
                id: child_id.clone(),
                parent_id: Some(parent_id.clone()),
                title: "Child".to_string(),
            })
            .unwrap();

        // Verify setup
        {
            let state: TunnelState = store.hydrate().unwrap();
            let parent = state.tasks.get(&parent_id).unwrap();
            assert!(parent.child_task_ids.contains(&child_id));
        }

        // Delete child
        store
            .dispatch(Action::DeleteTask {
                id: child_id.clone(),
            })
            .unwrap();

        // Verify child is gone from tasks and parent's children
        let state: TunnelState = store.hydrate().unwrap();
        assert!(!state.tasks.contains_key(&child_id));
        let parent = state.tasks.get(&parent_id).unwrap();
        assert!(!parent.child_task_ids.contains(&child_id));

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        parent_id.as_str() => {
                            map! {
                                "id" => { parent_id.as_str() },
                                "title" => { "Parent" },
                                "childTaskIds" => { list![] },
                                "status" => { "Pending" },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { automerge::ScalarValue::Null },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![ { parent_id.as_str() } ]
                },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );
    }

    #[test]
    fn test_dispatch_complete() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let task_id = TaskID::new();
        let task_id_str = task_id.to_string();

        store
            .dispatch(Action::CreateTask {
                id: task_id.clone(),
                parent_id: None,
                title: "To Complete".to_string(),
            })
            .unwrap();

        store
            .dispatch(Action::CompleteTask {
                id: task_id.clone(),
                current_time: 100,
            })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        task_id_str.as_str() => {
                            map! {
                                "id" => { task_id_str.as_str() },
                                "title" => { "To Complete" },
                                "status" => { "Done" },
                                "childTaskIds" => { list![] },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { 100 },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { task_id_str.as_str() }
                    ]
                },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );

        let state: TunnelState = store.hydrate().unwrap();
        assert_eq!(state.tasks.get(&task_id).unwrap().status, TaskStatus::Done);
    }

    #[test]
    fn test_dispatch_move() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let parent_id = TaskID::new();
        let parent_id_str = parent_id.to_string();
        let child_id = TaskID::new();
        let child_id_str = child_id.to_string();

        // Create Parent
        store
            .dispatch(Action::CreateTask {
                id: parent_id.clone(),
                parent_id: None,
                title: "Parent".to_string(),
            })
            .unwrap();

        // Create Child as root task initially
        store
            .dispatch(Action::CreateTask {
                id: child_id.clone(),
                parent_id: None,
                title: "Child".to_string(),
            })
            .unwrap();

        // Move Child under Parent
        store
            .dispatch(Action::MoveTask {
                id: child_id.clone(),
                new_parent_id: Some(parent_id.clone()),
            })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        parent_id_str.as_str() => {
                            map! {
                                "id" => { parent_id_str.as_str() },
                                "title" => { "Parent" },
                                "status" => { "Pending" },
                                "childTaskIds" => {
                                    list![
                                        { child_id_str.as_str() }
                                    ]
                                },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { automerge::ScalarValue::Null },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        },
                        child_id_str.as_str() => {
                            map! {
                                "id" => { child_id_str.as_str() },
                                "title" => { "Child" },
                                "status" => { "Pending" },
                                "childTaskIds" => { list![] },
                                "notes" => { "" },
                                "parentId" => { parent_id_str.as_str() },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { automerge::ScalarValue::Null },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { parent_id_str.as_str() }
                    ]
                },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );

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

        let task_id = TaskID::new();
        let task_id_str = task_id.to_string();

        // Create a Done task
        store
            .dispatch(Action::CreateTask {
                id: task_id.clone(),
                parent_id: None,
                title: "To Acknowledge".to_string(),
            })
            .unwrap();

        store
            .dispatch(Action::CompleteTask {
                id: task_id.clone(),
                current_time: 100,
            })
            .unwrap();

        // Verify not acknowledged yet
        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        task_id_str.as_str() => {
                            map! {
                                "id" => { task_id_str.as_str() },
                                "title" => { "To Acknowledge" },
                                "status" => { "Done" },
                                "childTaskIds" => { list![] },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { 100 },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { task_id_str.as_str() }
                    ]
                },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );

        // Refresh
        store
            .dispatch(Action::RefreshLifecycle { current_time: 100 })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        task_id_str.as_str() => {
                            map! {
                                "id" => { task_id_str.as_str() },
                                "title" => { "To Acknowledge" },
                                "status" => { "Done" },
                                "childTaskIds" => { list![] },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => { automerge::ScalarValue::Null },
                                "isSequential" => { false },
                                "isAcknowledged" => { true },
                                "lastCompletedAt" => { 100 },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 28800000 },
                                        "type" => { "Once" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { task_id_str.as_str() }
                    ]
                },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );

        assert!(store.hydrate::<TunnelState>().unwrap().tasks[&task_id].is_acknowledged);
    }

    #[test]
    fn test_dispatch_refresh_lifecycle_with_routine() {
        let mut store = AppStore::new();
        store.init().unwrap();

        let task_id = TaskID::new();
        let task_id_str = task_id.to_string();

        // Create a Routinely task
        store
            .dispatch(Action::CreateTask {
                id: task_id.clone(),
                parent_id: None,
                title: "Routine".to_string(),
            })
            .unwrap();

        {
            let mut state: TunnelState = store.hydrate().unwrap();
            let task = state.tasks.get_mut(&task_id).unwrap();
            task.status = TaskStatus::Done;
            task.schedule.schedule_type = tasklens_core::types::ScheduleType::Routinely;
            task.schedule.lead_time = Some(100);
            task.repeat_config = Some(tasklens_core::types::RepeatConfig {
                frequency: tasklens_core::types::Frequency::Daily,
                interval: 1,
            });
            task.last_completed_at = Some(1000);
            store.expensive_reconcile(&state).unwrap();
        }

        // Refresh before wake up
        store
            .dispatch(Action::RefreshLifecycle { current_time: 1000 })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        task_id_str.as_str() => {
                            map! {
                                "id" => { task_id_str.as_str() },
                                "title" => { "Routine" },
                                "status" => { "Done" },
                                "childTaskIds" => { list![] },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => {
                                    map! {
                                        "frequency" => { "Daily" },
                                        "interval" => { 1 }
                                    }
                                },
                                "isSequential" => { false },
                                "isAcknowledged" => { true },
                                "lastCompletedAt" => { 1000 },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { automerge::ScalarValue::Null },
                                        "leadTime" => { 100 },
                                        "type" => { "Routinely" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { task_id_str.as_str() }
                    ]
                },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );
        assert_eq!(
            store.hydrate::<TunnelState>().unwrap().tasks[&task_id].status,
            TaskStatus::Done
        );

        // Refresh after wake up
        store
            .dispatch(Action::RefreshLifecycle {
                current_time: 86401000,
            })
            .unwrap();

        assert_doc!(
            &store.doc,
            map! {
                "tasks" => {
                    map! {
                        task_id_str.as_str() => {
                            map! {
                                "id" => { task_id_str.as_str() },
                                "title" => { "Routine" },
                                "status" => { "Pending" },
                                "childTaskIds" => { list![] },
                                "notes" => { "" },
                                "parentId" => { automerge::ScalarValue::Null },
                                "placeId" => { automerge::ScalarValue::Null },
                                "importance" => { 1.0 },
                                "creditIncrement" => { 0.5 },
                                "credits" => { 0.0 },
                                "desiredCredits" => { 1.0 },
                                "creditsTimestamp" => { 0 },
                                "priorityTimestamp" => { 0 },
                                "repeatConfig" => {
                                    map! {
                                        "frequency" => { "Daily" },
                                        "interval" => { 1 }
                                    }
                                },
                                "isSequential" => { false },
                                "isAcknowledged" => { false },
                                "lastCompletedAt" => { 1000 },
                                "schedule" => {
                                    map! {
                                        "dueDate" => { automerge::ScalarValue::Null },
                                        "lastDone" => { 1000 },
                                        "leadTime" => { 100 },
                                        "type" => { "Routinely" }
                                    }
                                }
                            }
                        }
                    }
                },
                "rootTaskIds" => {
                    list![
                        { task_id_str.as_str() }
                    ]
                },
                "places" => { map!{} },
                "nextTaskId" => { 1 },
                "nextPlaceId" => { 1 },
                "metadata" => { automerge::ScalarValue::Null }
            }
        );
        assert_eq!(
            store.hydrate::<TunnelState>().unwrap().tasks[&task_id].status,
            TaskStatus::Pending
        );
    }
}

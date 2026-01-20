pub use crate::actions::{Action, TaskUpdates};
use crate::doc_id::{DocumentId, TaskLensUrl};
#[cfg(target_arch = "wasm32")]
use crate::storage::{ActiveDocStorage, IndexedDbStorage};
use anyhow::{Result, anyhow};
use automerge::transaction::Transactable;
use autosurgeon::{Doc, reconcile};
use std::collections::HashMap;

use tasklens_core::types::{
    DocMetadata, PersistedTask, TaskID, TaskStatus, TunnelState, hydrate_optional_task_id,
};

fn am_get<'a, T: Transactable>(
    doc: &'a T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
) -> Result<Option<(automerge::Value<'a>, automerge::ObjId)>, automerge::AutomergeError> {
    doc.get(obj, prop)
}

fn am_delete<T: Transactable>(
    doc: &mut T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
) -> Result<(), automerge::AutomergeError> {
    doc.delete(obj, prop)
}

fn am_put_object<T: Transactable>(
    doc: &mut T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
    value: automerge::ObjType,
) -> Result<automerge::ObjId, automerge::AutomergeError> {
    doc.put_object(obj, prop, value)
}

/// A manager for Automerge documents and persistence.
///
/// This struct implements a Repo-like pattern, managing the current
/// document and providing methods for document lifecycle management.
#[derive(Clone, Debug)]
pub struct AppStore {
    /// The ID of the currently loaded document.
    pub current_id: Option<DocumentId>,
    pub handle: Option<samod::DocHandle>,
    pub repo: Option<samod::Repo>,
}

impl AppStore {
    /// Creates a new AppStore with a fresh document.
    pub fn new() -> Self {
        Self {
            current_id: None,
            handle: None,
            repo: None,
        }
    }

    /// initialize with a specific repo (useful for tests)
    pub fn with_repo(repo: samod::Repo) -> Self {
        Self {
            current_id: None,
            handle: None,
            repo: Some(repo),
        }
    }

    /// Creates a new document using the provided repo.
    pub async fn create_new_detached(repo: samod::Repo) -> Result<(samod::DocHandle, DocumentId)> {
        // Create new document
        let handle = repo.create(automerge::Automerge::new());
        let handle = handle
            .await
            .map_err(|e| anyhow!("Failed to create doc: {:?}", e))?;
        let id = DocumentId::from(handle.document_id().clone());

        // Initialize with default state
        handle.with_document(|doc| {
            let mut tx = doc.transaction();

            let initial_state = TunnelState {
                next_task_id: 1,
                next_place_id: 1,
                tasks: HashMap::new(),
                places: HashMap::new(),
                root_task_ids: Vec::new(),
                metadata: Some(DocMetadata {
                    automerge_url: Some(TaskLensUrl::from(id.clone()).to_string()),
                }),
            };

            if let Err(e) = reconcile(&mut tx, &initial_state) {
                tracing::error!("Failed to reconcile initial state: {}", e);
            }
            tx.commit();
        });

        Ok((handle, id))
    }

    /// Finds a document using the provided repo.
    pub async fn find_doc_detached(repo: samod::Repo, id: DocumentId) -> Result<Option<samod::DocHandle>> {
        let handle = repo.find(id.into());
        let handle = handle
            .await
            .map_err(|e| anyhow!("Failed to find doc: {:?}", e))?;
        Ok(handle)
    }

    /// Updates the store to track the provided document.
    pub fn set_active_doc(&mut self, handle: samod::DocHandle, id: DocumentId) {
        self.handle = Some(handle);
        self.current_id = Some(id.clone());
        #[cfg(target_arch = "wasm32")]
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(id));
    }

    /// Creates a new document and switches to it.
    pub async fn create_new(&mut self) -> Result<DocumentId> {
        if let Some(repo) = &self.repo {
            let (handle, id) = Self::create_new_detached(repo.clone()).await?;
            self.set_active_doc(handle, id.clone());
            Ok(id)
        } else {
            Err(anyhow!("Repo not initialized"))
        }
    }

    /// Switches to an existing document by ID.
    pub async fn switch_doc(&mut self, id: DocumentId) -> Result<()> {
        if let Some(repo) = &self.repo {
             if let Some(handle) = Self::find_doc_detached(repo.clone(), id.clone()).await? {
                 self.set_active_doc(handle, id);
                 Ok(())
             } else {
                 Err(anyhow!("Document not found in repo"))
             }
        } else {
            Err(anyhow!("Repo not initialized"))
        }
    }

    /// Deletes a document.
    pub async fn delete_doc(&mut self, id: DocumentId) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        IndexedDbStorage::delete_doc(&id).await?;

        if self.current_id == Some(id) {
            // If we deleted the current doc, create a new one
            // This might fail if no repo, but delete_doc implies we had some state
            if self.repo.is_some() {
                self.create_new().await?;
            }
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save_active_doc_id(id: &DocumentId) {
        crate::storage::ActiveDocStorage::save_active_url(&TaskLensUrl::from(id.clone()));
    }

    /// Imports a document from a byte array.
    pub async fn import_doc(&mut self, bytes: Vec<u8>) -> Result<DocumentId> {
        if let Some(repo) = &self.repo {
            let doc = automerge::Automerge::load(&bytes)?;
            let handle = repo
                .create(doc)
                .await
                .map_err(|e| anyhow!("Failed to create (import) doc: {:?}", e))?;
            let id = DocumentId::from(handle.document_id().clone());

            self.handle = Some(handle);
            self.current_id = Some(id.clone());

            #[cfg(target_arch = "wasm32")]
            ActiveDocStorage::save_active_url(&TaskLensUrl::from(id.clone()));

            Ok(id)
        } else {
            Err(anyhow!("Repo not initialized"))
        }
    }
    /// Exports the current document to a byte array.
    pub fn export_save(&self) -> Vec<u8> {
        if let Some(handle) = &self.handle {
            handle.with_document(|doc| doc.save())
        } else {
            Vec::new()
        }
    }

    /// Reconciles a Rust struct with the current document.
    pub fn expensive_reconcile<T: autosurgeon::Reconcile>(
        &mut self,
        data: &T,
    ) -> Result<(), autosurgeon::ReconcileError> {
        if let Some(handle) = &mut self.handle {
            handle.with_document(|doc| {
                let mut tx = doc.transaction();
                reconcile(&mut tx, data)?;
                tx.commit();
                Ok(())
            })
        } else {
            // autosurgeon::ReconcileError's exact variants depend on version.
            // Often it has a catch-all or Automerge wrapper.
            // Let's use a simpler approach that is likely to work or fail with a clearer error.
            Err(autosurgeon::ReconcileError::Automerge(
                automerge::AutomergeError::InvalidObjId("root".to_string()),
            ))
        }
    }

    /// Hydrates a Rust struct from the current document.
    pub fn hydrate<T: autosurgeon::Hydrate>(&self) -> Result<T> {
        if let Some(handle) = &self.handle {
            handle.with_document(|doc| {
                autosurgeon::hydrate(doc).map_err(|e| anyhow!("Hydration failed: {}", e))
            })
        } else {
            Err(anyhow!("No handle available"))
        }
    }

    /// Dispatches an action to modify the application state.
    ///
    /// This method is the primary entry point for state mutations. It implements a
    /// functional core pattern using Autosurgeon's hydration and reconciliation capabilities:
    ///
    /// 1. **Mutate**: The `Action` is applied to the `TunnelState` via specialized handlers.
    /// 2. **Reconcile**: Handlers use surgical reconciliation for efficiency.
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
        let handle = self.handle.as_mut().ok_or_else(|| anyhow!("No handle"))?;
        Self::dispatch_with_handle(handle, action)
    }

    /// Static handler for dispatch that works with a handle directly.
    pub fn dispatch_with_handle(handle: &mut samod::DocHandle, action: Action) -> Result<()> {
        handle.with_document(|doc| {
            let mut tx = doc.transaction();
            let res = match action {
                Action::CreateTask {
                    id,
                    parent_id,
                    title,
                } => Self::handle_create_task(&mut tx, id, parent_id, title),
                Action::UpdateTask { id, updates } => {
                    Self::handle_update_task(&mut tx, id, updates)
                }
                Action::DeleteTask { id } => Self::handle_delete_task(&mut tx, id),
                Action::CompleteTask { id, current_time } => {
                    Self::handle_complete_task(&mut tx, id, current_time)
                }
                Action::MoveTask { id, new_parent_id } => {
                    Self::handle_move_task(&mut tx, id, new_parent_id)
                }
                Action::RefreshLifecycle { current_time } => {
                    Self::handle_refresh_lifecycle(&mut tx, current_time)
                }
            };
            tx.commit();
            res
        })
    }

    fn handle_create_task(
        doc: &mut (impl Transactable + Doc),
        id: TaskID,
        parent_id: Option<TaskID>,
        title: String,
    ) -> Result<()> {
        // 1. Get Tasks Map.
        let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

        // 2. Resolve parent task.
        let parent = if let Some(pid) = &parent_id {
            let p: Option<PersistedTask> =
                autosurgeon::hydrate_prop(doc, &tasks_obj_id, pid.as_str())
                    .map_err(|e| anyhow!("Failed to hydrate parent task: {}", e))?;
            p
        } else {
            None
        };

        // 3. Create the new task struct.
        let task = tasklens_core::create_new_task(id.clone(), title, parent.as_ref());

        // 4. Reconcile the new task.
        autosurgeon::reconcile_prop(doc, &tasks_obj_id, id.as_str(), &task)
            .map_err(|e| anyhow!("Failed to reconcile new task: {}", e))?;

        // 5. Update parent's child list or root list.
        if let Some(pid) = parent_id {
            // Get parent object ID.
            let parent_obj_id = ensure_path(doc, &tasks_obj_id, vec![pid.as_str()])?;

            // Hydrate current children list.
            let mut child_ids: Vec<TaskID> =
                autosurgeon::hydrate_prop(doc, &parent_obj_id, "childTaskIds").unwrap_or_default();

            child_ids.push(id);

            // Reconcile updated children list.
            autosurgeon::reconcile_prop(doc, &parent_obj_id, "childTaskIds", &child_ids)
                .map_err(|e| anyhow!("Failed to reconcile child ids: {}", e))?;
        } else {
            // Update root task list.
            let mut root_ids: Vec<TaskID> =
                autosurgeon::hydrate_prop(doc, automerge::ROOT, "rootTaskIds").unwrap_or_default();

            root_ids.push(id);

            autosurgeon::reconcile_prop(doc, automerge::ROOT, "rootTaskIds", &root_ids)
                .map_err(|e| anyhow!("Failed to reconcile root task ids: {}", e))?;
        }

        Ok(())
    }

    fn handle_update_task(
        doc: &mut (impl Transactable + Doc),
        id: TaskID,
        updates: TaskUpdates,
    ) -> Result<()> {
        let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

        let task_obj_id = ensure_path(doc, &tasks_obj_id, vec![id.as_str()])
            .map_err(|_| anyhow!("Task not found: {}", id))?;

        if let Some(title) = updates.title {
            autosurgeon::reconcile_prop(doc, &task_obj_id, "title", title)
                .map_err(|e| anyhow!("Failed to update title: {}", e))?;
        }
        if let Some(status) = updates.status {
            autosurgeon::reconcile_prop(doc, &task_obj_id, "status", status)
                .map_err(|e| anyhow!("Failed to update status: {}", e))?;
        }
        if let Some(place_id_update) = updates.place_id {
            autosurgeon::reconcile_prop(doc, &task_obj_id, "placeId", place_id_update)
                .map_err(|e| anyhow!("Failed to update placeId: {}", e))?;
        }

        if updates.due_date.is_some()
            || updates.schedule_type.is_some()
            || updates.lead_time.is_some()
        {
            let schedule_obj_id = ensure_path(doc, &task_obj_id, vec!["schedule"])?;

            if let Some(due_date_update) = updates.due_date {
                autosurgeon::reconcile_prop(doc, &schedule_obj_id, "dueDate", due_date_update)
                    .map_err(|e| anyhow!("Failed to update dueDate: {}", e))?;
            }
            if let Some(schedule_type) = updates.schedule_type {
                autosurgeon::reconcile_prop(doc, &schedule_obj_id, "type", schedule_type)
                    .map_err(|e| anyhow!("Failed to update schedule type: {}", e))?;
            }
            if let Some(lead_time_update) = updates.lead_time {
                autosurgeon::reconcile_prop(doc, &schedule_obj_id, "leadTime", lead_time_update)
                    .map_err(|e| anyhow!("Failed to update leadTime: {}", e))?;
            }
        }

        if let Some(repeat_config_update) = updates.repeat_config {
            autosurgeon::reconcile_prop(doc, &task_obj_id, "repeatConfig", repeat_config_update)
                .map_err(|e| anyhow!("Failed to update repeatConfig: {}", e))?;
        }
        if let Some(is_seq) = updates.is_sequential {
            autosurgeon::reconcile_prop(doc, &task_obj_id, "isSequential", is_seq)
                .map_err(|e| anyhow!("Failed to update isSequential: {}", e))?;
        }

        Ok(())
    }

    // TODO: Optimize this to use surgical updates (reconcile_prop) similar to handle_create_task.
    // Currently uses full state hydration which is less efficient.
    fn handle_delete_task(doc: &mut (impl Transactable + Doc), id: TaskID) -> Result<()> {
        let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

        // 1. Get task object ID.
        let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
            Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
            _ => return Err(anyhow!("Task not found: {}", id)),
        };

        // 2. Hydrate parent_id to know where to remove it from.
        let parent_id: Option<TaskID> =
            hydrate_optional_task_id(doc, &task_obj_id, autosurgeon::Prop::Key("parentId".into()))
                .map_err(|e| anyhow!("Failed to hydrate parentId: {}", e))?;

        // 3. Remove from parent's children or rootTaskIds.
        if let Some(pid) = parent_id {
            // Get parent object ID.
            let parent_obj_id = ensure_path(doc, &tasks_obj_id, vec![pid.as_str()])?;

            // Hydrate current children list.
            let mut child_ids: Vec<TaskID> =
                autosurgeon::hydrate_prop(doc, &parent_obj_id, "childTaskIds").unwrap_or_default();

            child_ids.retain(|cid| cid != &id);

            // Reconcile updated children list.
            autosurgeon::reconcile_prop(doc, &parent_obj_id, "childTaskIds", &child_ids)
                .map_err(|e| anyhow!("Failed to reconcile parent's childTaskIds: {}", e))?;
        } else {
            // Update root task list.
            let mut root_ids: Vec<TaskID> =
                autosurgeon::hydrate_prop(doc, &automerge::ROOT, "rootTaskIds").unwrap_or_default();

            root_ids.retain(|rid| rid != &id);

            autosurgeon::reconcile_prop(doc, &automerge::ROOT, "rootTaskIds", &root_ids)
                .map_err(|e| anyhow!("Failed to reconcile rootTaskIds: {}", e))?;
        }

        // 4. Delete the task object itself from the tasks map.
        am_delete(doc, &tasks_obj_id, id.as_str())
            .map_err(|e| anyhow!("Failed to delete task object: {}", e))?;

        Ok(())
    }

    fn handle_complete_task(
        doc: &mut (impl Transactable + Doc),
        id: TaskID,
        current_time: i64,
    ) -> Result<()> {
        let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

        let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
            Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
            _ => return Err(anyhow!("Task not found: {}", id)),
        };

        autosurgeon::reconcile_prop(doc, &task_obj_id, "status", TaskStatus::Done)
            .map_err(|e| anyhow!("Failed to update status: {}", e))?;

        autosurgeon::reconcile_prop(doc, &task_obj_id, "lastCompletedAt", Some(current_time))
            .map_err(|e| anyhow!("Failed to update lastCompletedAt: {}", e))?;

        Ok(())
    }

    // TODO: Optimize this to use surgical updates (reconcile_prop) similar to handle_create_task.
    // Currently uses full state hydration which is less efficient.
    fn handle_move_task(
        doc: &mut (impl Transactable + Doc),
        id: TaskID,
        new_parent_id: Option<TaskID>,
    ) -> Result<()> {
        let mut state: TunnelState = autosurgeon::hydrate(doc)?;
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
        reconcile(doc, &state).map_err(|e| anyhow!("Dispatch reconciliation failed: {}", e))?;
        Ok(())
    }

    // TODO: Optimize this to use surgical updates (reconcile_prop) similar to
    // handle_create_task. Currently uses full state hydration which is less
    // efficient. Note: this may be difficult or infaesible!
    fn handle_refresh_lifecycle(
        doc: &mut (impl Transactable + Doc),
        current_time: i64,
    ) -> Result<()> {
        let mut state: TunnelState = autosurgeon::hydrate(doc)?;
        tasklens_core::domain::lifecycle::acknowledge_completed_tasks(&mut state);
        tasklens_core::domain::routine_tasks::wake_up_routine_tasks(&mut state, current_time);
        // TODO: use reconcile_prop()
        reconcile(doc, &state).map_err(|e| anyhow!("Dispatch reconciliation failed: {}", e))?;
        Ok(())
    }

    // Legacy methods removed
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
fn ensure_path<T: Transactable + Doc>(
    doc: &mut T,
    root: &automerge::ObjId,
    path: Vec<&str>,
) -> Result<automerge::ObjId> {
    let mut current = root.clone();
    for key in path {
        let val = am_get(doc, &current, key)?;
        current = match val {
            Some((automerge::Value::Object(_), id)) => id,
            None => am_put_object(doc, &current, key, automerge::ObjType::Map)?,
            _ => return Err(anyhow!("Path key '{}' is not an object", key)),
        };
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use automerge::AutoCommit;
    use automerge_test::{assert_doc, list, map};
    use tasklens_core::TaskID;

    use super::*;

    /// A shim to support legacy tests with the new static handlers.
    struct AppStore {
        doc: AutoCommit,
    }

    impl AppStore {
        fn new() -> Self {
            Self {
                doc: AutoCommit::new(),
            }
        }

        fn init(&mut self) -> Result<()> {
            let initial_state = TunnelState {
                next_task_id: 1,
                next_place_id: 1,
                tasks: HashMap::new(),
                places: HashMap::new(),
                root_task_ids: Vec::new(),
                metadata: None,
            };
            reconcile(&mut self.doc, &initial_state).map_err(|e| anyhow!("Init failed: {}", e))
        }

        fn dispatch(&mut self, action: Action) -> Result<()> {
            Self::dispatch_static(&mut self.doc, action)
        }

        fn dispatch_static(doc: &mut AutoCommit, action: Action) -> Result<()> {
            match action {
                Action::CreateTask {
                    id,
                    parent_id,
                    title,
                } => super::AppStore::handle_create_task(doc, id, parent_id, title),
                Action::UpdateTask { id, updates } => {
                    super::AppStore::handle_update_task(doc, id, updates)
                }
                Action::DeleteTask { id } => super::AppStore::handle_delete_task(doc, id),
                Action::CompleteTask { id, current_time } => {
                    super::AppStore::handle_complete_task(doc, id, current_time)
                }
                Action::MoveTask { id, new_parent_id } => {
                    super::AppStore::handle_move_task(doc, id, new_parent_id)
                }
                Action::RefreshLifecycle { current_time } => {
                    super::AppStore::handle_refresh_lifecycle(doc, current_time)
                }
            }
        }

        fn hydrate<T: autosurgeon::Hydrate>(&self) -> Result<T> {
            autosurgeon::hydrate(&self.doc).map_err(|e| anyhow!("Hydration failed: {}", e))
        }

        fn expensive_reconcile(&mut self, state: &TunnelState) -> Result<()> {
            reconcile(&mut self.doc, state).map_err(|e| anyhow!("Reconciliation failed: {}", e))
        }
    }

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

#[cfg(test)]
mod tests_async {
    use super::*;
    use samod::runtime::LocalRuntimeHandle;
    use std::future::Future;
    use std::pin::Pin;

    #[derive(Clone, Debug)]
    struct TestRuntime;

    impl LocalRuntimeHandle for TestRuntime {
        fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
            tokio::task::spawn_local(future);
        }
    }

    async fn create_test_store() -> AppStore {
        // Use load_local() as create() seems unavailable or trait-gated
        let repo = samod::RepoBuilder::new(TestRuntime).load_local().await;
        // Use with_repo for testing
        let mut store = AppStore::with_repo(repo);
        // Create new doc to ensure store is ready
        store.create_new().await.unwrap();
        store
    }

    #[tokio::test]
    async fn test_dispatch_create_task() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let mut store = create_test_store().await;
                let task_id = TaskID::new();

                store
                    .dispatch(Action::CreateTask {
                        id: task_id.clone(),
                        parent_id: None,
                        title: "Dispatch Test".to_string(),
                    })
                    .unwrap();

                let state: TunnelState = store.hydrate().unwrap();
                assert_eq!(state.tasks[&task_id].title, "Dispatch Test");
            })
            .await;
    }

    #[tokio::test]
    async fn test_dispatch_update_task() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let mut store = create_test_store().await;
                let task_id = TaskID::new();
                store
                    .dispatch(Action::CreateTask {
                        id: task_id.clone(),
                        parent_id: None,
                        title: "Update Test".to_string(),
                    })
                    .unwrap();

                store
                    .dispatch(Action::UpdateTask {
                        id: task_id.clone(),
                        updates: TaskUpdates {
                            title: Some("Updated Title".to_string()),
                            status: Some(TaskStatus::Done),
                            ..Default::default()
                        },
                    })
                    .unwrap();

                let state: TunnelState = store.hydrate().unwrap();
                assert_eq!(state.tasks[&task_id].title, "Updated Title");
                assert_eq!(state.tasks[&task_id].status, TaskStatus::Done);
            })
            .await;
    }

    #[tokio::test]
    async fn test_dispatch_move_task() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let mut store = create_test_store().await;
                let parent1 = TaskID::new();
                let parent2 = TaskID::new();
                let child = TaskID::new();

                store
                    .dispatch(Action::CreateTask {
                        id: parent1.clone(),
                        parent_id: None,
                        title: "P1".into(),
                    })
                    .unwrap();
                store
                    .dispatch(Action::CreateTask {
                        id: parent2.clone(),
                        parent_id: None,
                        title: "P2".into(),
                    })
                    .unwrap();
                store
                    .dispatch(Action::CreateTask {
                        id: child.clone(),
                        parent_id: Some(parent1.clone()),
                        title: "Child".into(),
                    })
                    .unwrap();

                // Verify initial state
                let state: TunnelState = store.hydrate().unwrap();
                assert!(state.tasks[&parent1].child_task_ids.contains(&child));

                // Move
                store
                    .dispatch(Action::MoveTask {
                        id: child.clone(),
                        new_parent_id: Some(parent2.clone()),
                    })
                    .unwrap();

                let state: TunnelState = store.hydrate().unwrap();
                assert!(!state.tasks[&parent1].child_task_ids.contains(&child));
                assert!(state.tasks[&parent2].child_task_ids.contains(&child));
                assert_eq!(state.tasks[&child].parent_id, Some(parent2));
            })
            .await;
    }
}

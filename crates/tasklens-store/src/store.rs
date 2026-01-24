pub use crate::actions::{Action, TaskUpdates};
use crate::doc_id::{DocumentId, TaskLensUrl};
#[cfg(target_arch = "wasm32")]
use crate::storage::{ActiveDocStorage, IndexedDbStorage};
use anyhow::{Result, anyhow};
use automerge::ReadDoc;
use automerge::transaction::Transactable;
use autosurgeon::{Doc, MaybeMissing, reconcile};
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
    pub async fn create_new(repo: samod::Repo) -> Result<(samod::DocHandle, DocumentId)> {
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
    pub async fn find_doc(repo: samod::Repo, id: DocumentId) -> Result<Option<samod::DocHandle>> {
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

    #[cfg(target_arch = "wasm32")]
    pub fn save_active_doc_id(id: &DocumentId) {
        crate::storage::ActiveDocStorage::save_active_url(&TaskLensUrl::from(id.clone()));
    }

    /// Imports a document from a byte array.
    /// This is detached from the store instance to avoid holding locks during async operations.
    pub async fn import_doc(
        repo: samod::Repo,
        bytes: Vec<u8>,
    ) -> Result<(samod::DocHandle, DocumentId)> {
        let doc = automerge::Automerge::load(&bytes)?;

        // Try to extract existing ID from metadata to preserve identity
        let target_id = autosurgeon::hydrate::<_, TunnelState>(&doc)
            .ok()
            .and_then(|state| state.metadata)
            .and_then(|meta| meta.automerge_url)
            .and_then(|url_str| url_str.parse::<TaskLensUrl>().ok())
            .map(|url| url.document_id);

        if let Some(id) = target_id {
            tracing::info!("Importing document with existing ID: {}", id);

            #[cfg(target_arch = "wasm32")]
            {
                use crate::samod_storage::SamodStorage;
                use samod::storage::LocalStorage;

                // Manually inject into storage to bypass Repo::create generating a new ID
                let storage = SamodStorage::new("tasklens_samod", "documents");
                // Samod keys are typically the string representation of the ID
                if let Ok(key) = samod::storage::StorageKey::from_parts(vec![id.to_string()]) {
                    storage.put(key, bytes.clone()).await;

                    // Now find it via Repo (which should look in storage)
                    if let Ok(Some(handle)) = Self::find_doc(repo.clone(), id.clone()).await {
                        return Ok((handle, id));
                    }
                }
            }
        }

        let handle = repo
            .create(doc)
            .await
            .map_err(|e| anyhow!("Failed to create (import) doc: {:?}", e))?;
        let id = DocumentId::from(handle.document_id().clone());

        #[cfg(target_arch = "wasm32")]
        ActiveDocStorage::save_active_url(&TaskLensUrl::from(id.clone()));

        Ok((handle, id))
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

    /// A "total hack" repair utility that fixes tasks with "DoDonee" status,
    /// changing them to "Done". This should be called if hydration fails
    /// because of an "unexpected DoDonee" error.
    pub fn repair_dodonee(&mut self) -> Result<()> {
        let handle = self.handle.as_mut().ok_or_else(|| anyhow!("No handle"))?;
        handle.with_document(|doc| {
            let mut tx = doc.transaction();

            // 1. Get tasks map
            let tasks_obj_id = match am_get(&tx, &automerge::ROOT, "tasks")? {
                Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
                _ => return Ok(()),
            };

            // 2. Iterate keys and find those needing repair
            let mut tasks_to_repair = Vec::new();
            {
                let keys = tx.keys(&tasks_obj_id);
                for task_id in keys {
                    let task_obj_id = match am_get(&tx, &tasks_obj_id, task_id.as_str())? {
                        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
                        _ => continue,
                    };

                    // Check status
                    let status_val = am_get(&tx, &task_obj_id, "status")?;
                    let is_dodonee = match status_val {
                        Some((automerge::Value::Scalar(scalar), _)) => match scalar.as_ref() {
                            automerge::ScalarValue::Str(s) => s.as_str() == "DoDonee",
                            _ => false,
                        },
                        Some((automerge::Value::Object(automerge::ObjType::Text), id)) => {
                            tx.text(id)? == "DoDonee"
                        }
                        _ => false,
                    };

                    if is_dodonee {
                        tasks_to_repair.push(task_obj_id);
                    }
                }
            }

            // 3. Repair them
            for task_obj_id in tasks_to_repair {
                tracing::info!("Repairing task {:?}: DoDonee -> Done", task_obj_id);
                // Set it to Done.
                autosurgeon::reconcile_prop(&mut tx, task_obj_id, "status", TaskStatus::Done)
                    .map_err(|e| anyhow!("Repair reconciliation failed: {}", e))?;
            }
            tx.commit();
            Ok(())
        })
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
                match autosurgeon::hydrate_prop(doc, &parent_obj_id, "childTaskIds") {
                    Ok(ids) => match ids {
                        MaybeMissing::Missing => Vec::new(),
                        MaybeMissing::Present(ids) => ids,
                    },
                    Err(e) => return Err(anyhow!("Failed to hydrate child ids: {}", e)),
                };

            child_ids.push(id);

            // Reconcile updated children list.
            autosurgeon::reconcile_prop(doc, &parent_obj_id, "childTaskIds", &child_ids)
                .map_err(|e| anyhow!("Failed to reconcile child ids: {}", e))?;
        } else {
            // Update root task list.
            let mut root_ids: Vec<TaskID> =
                match autosurgeon::hydrate_prop(doc, automerge::ROOT, "rootTaskIds") {
                    Ok(ids) => match ids {
                        MaybeMissing::Missing => Vec::new(),
                        MaybeMissing::Present(ids) => ids,
                    },
                    Err(e) => return Err(anyhow!("Failed to hydrate root task ids: {}", e)),
                };

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
                match autosurgeon::hydrate_prop(doc, &parent_obj_id, "childTaskIds") {
                    Ok(ids) => match ids {
                        MaybeMissing::Missing => Vec::new(),
                        MaybeMissing::Present(ids) => ids,
                    },
                    Err(e) => {
                        return Err(anyhow!("Failed to hydrate parent's childTaskIds: {}", e));
                    }
                };

            child_ids.retain(|cid| cid != &id);

            // Reconcile updated children list.
            autosurgeon::reconcile_prop(doc, &parent_obj_id, "childTaskIds", &child_ids)
                .map_err(|e| anyhow!("Failed to reconcile parent's childTaskIds: {}", e))?;
        } else {
            // Update root task list.
            let mut root_ids: Vec<TaskID> =
                match autosurgeon::hydrate_prop(doc, &automerge::ROOT, "rootTaskIds") {
                    Ok(ids) => match ids {
                        MaybeMissing::Missing => Vec::new(),
                        MaybeMissing::Present(ids) => ids,
                    },
                    Err(e) => return Err(anyhow!("Failed to hydrate rootTaskIds: {}", e)),
                };

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
mod tests;
#[cfg(test)]
mod tests_async;

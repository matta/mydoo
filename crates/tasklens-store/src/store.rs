use crate::actions::Action;
#[cfg(target_arch = "wasm32")]
use crate::storage::IndexedDbStorage;
use anyhow::{Result, anyhow};
use autosurgeon::{hydrate, reconcile};
use futures::StreamExt;
#[cfg(not(target_arch = "wasm32"))]
use samod::runtime::RuntimeHandle;
#[cfg(not(target_arch = "wasm32"))]
use samod::storage::InMemoryStorage;
use samod::{DocHandle, Repo};
use std::collections::HashMap;

use tasklens_core::types::{
    PersistedTask, Schedule, ScheduleType, TaskID, TaskStatus, TunnelState,
};

#[derive(Clone, Debug)]
pub struct AppStore {
    repo: Repo,
    root_handle: Option<DocHandle>,
}

impl AppStore {
    #[cfg(target_arch = "wasm32")]
    pub async fn new() -> Result<Self> {
        let storage = IndexedDbStorage::new("tasklens");
        let runtime = WasmRuntime;

        let repo = Repo::builder(runtime)
            .with_storage(storage)
            .load_local()
            .await;

        Ok(Self {
            repo,
            root_handle: None,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new<R: RuntimeHandle>(runtime: R) -> Result<Self> {
        let storage = InMemoryStorage::default();

        let repo = Repo::builder(runtime).with_storage(storage).load().await;

        Ok(Self {
            repo,
            root_handle: None,
        })
    }

    /// Initialize the store by creating a new document or loading an existing one.
    pub async fn init(&mut self) -> Result<()> {
        // TODO: In a real app, we'd probably try to find an existing document ID in storage
        // or accept one as an argument. For Milestone 2.1, we'll try to load or create.

        let handle = self
            .repo
            .create(automerge::Automerge::new())
            .await
            .map_err(|_| anyhow!("Failed to create document"))?;

        // Seed with initial state if empty
        let mut result = Ok(());
        handle.with_document(|doc| {
            let current_state: Result<TunnelState, _> = hydrate(doc);
            if current_state.is_err() || current_state.as_ref().unwrap().tasks.is_empty() {
                let initial_state = TunnelState {
                    next_task_id: 1.0,
                    next_place_id: 1.0,
                    tasks: HashMap::new(),
                    places: HashMap::new(),
                    root_task_ids: Vec::new(),
                };
                let mut tx = doc.transaction();
                let res = reconcile(&mut tx, &initial_state)
                    .map_err(|e| anyhow!("Init reconciliation failed: {}", e));
                if res.is_ok() {
                    tx.commit();
                }
                result = res;
            }
        });
        result?;

        self.root_handle = Some(handle);
        Ok(())
    }

    pub fn get_state(&self) -> Result<TunnelState> {
        let handle = self
            .root_handle
            .as_ref()
            .ok_or_else(|| anyhow!("AppStore not initialized"))?;

        let mut state = Err(anyhow!("Hydration failed internal"));
        handle.with_document(|doc| {
            state = hydrate(doc).map_err(|e| anyhow!("Hydration failed: {}", e));
        });
        state
    }

    pub fn dispatch(&self, action: Action) -> Result<()> {
        let handle = self
            .root_handle
            .as_ref()
            .ok_or_else(|| anyhow!("AppStore not initialized"))?;

        let mut result = Ok(());
        handle.with_document(|doc| {
            let mut state: TunnelState = match hydrate(doc) {
                Ok(s) => s,
                Err(e) => {
                    result = Err(anyhow!("Dispatch hydration failed: {}", e));
                    return;
                }
            };

            match action {
                Action::CreateTask { parent_id, title } => {
                    let id = TaskID::new();
                    let task = PersistedTask {
                        id: id.clone(),
                        title,
                        notes: String::new(),
                        parent_id: parent_id.clone(),
                        child_task_ids: Vec::new(),
                        place_id: None,
                        status: TaskStatus::Pending,
                        importance: 1.0,
                        credit_increment: None,
                        credits: 0.0,
                        desired_credits: 1.0,
                        credits_timestamp: 0.0,
                        priority_timestamp: 0.0,
                        schedule: Schedule {
                            schedule_type: ScheduleType::Once,
                            due_date: None,
                            lead_time: Some(0.0),
                            last_done: None,
                        },
                        repeat_config: None,
                        is_sequential: false,
                        is_acknowledged: false,
                        last_completed_at: None,
                    };
                    state.tasks.insert(id.clone(), task);
                    if let Some(pid) = parent_id {
                        if let Some(parent) = state.tasks.get_mut(&pid) {
                            parent.child_task_ids.push(id);
                        }
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
                Action::CompleteTask { id } => {
                    if let Some(task) = state.tasks.get_mut(&id) {
                        task.status = TaskStatus::Done;
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
            }
            let mut tx = doc.transaction();
            if let Err(e) = reconcile(&mut tx, &state) {
                result = Err(anyhow!("Dispatch reconciliation failed: {}", e));
            } else {
                tx.commit();
            }
        });
        result
    }

    /// Subscribe to state changes.
    ///
    /// Returns a stream that yields the current state whenever the underlying
    /// Automerge document changes. The stream will yield the initial state
    /// immediately, then subsequent states as changes occur.
    pub fn subscribe(&self) -> impl futures::Stream<Item = TunnelState> + '_ {
        let handle = self.root_handle.as_ref().expect("Store not initialized");

        let initial_state = self.get_state().unwrap_or_else(|_| TunnelState {
            next_task_id: 1.0,
            next_place_id: 1.0,
            tasks: HashMap::new(),
            places: HashMap::new(),
            root_task_ids: Vec::new(),
        });

        let changes_stream = handle.changes();
        let handle_for_map = handle.clone();

        let updates = changes_stream.map(move |_| {
            let mut state = None;
            handle_for_map.with_document(|doc| {
                if let Ok(s) = hydrate(doc) {
                    state = Some(s);
                } else {
                    tracing::warn!("Failed to hydrate state in subscribe");
                }
            });
            state.unwrap_or(TunnelState {
                next_task_id: 1.0,
                next_place_id: 1.0,
                tasks: HashMap::new(),
                places: HashMap::new(),
                root_task_ids: Vec::new(),
            })
        });

        futures::stream::once(async move { initial_state }).chain(updates)
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct WasmRuntime;

#[cfg(target_arch = "wasm32")]
impl samod::runtime::RuntimeHandle for WasmRuntime {
    fn spawn(&self, f: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>) {
        wasm_bindgen_futures::spawn_local(f);
    }
}

#[cfg(target_arch = "wasm32")]
impl samod::runtime::LocalRuntimeHandle for WasmRuntime {
    fn spawn(&self, f: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>) {
        wasm_bindgen_futures::spawn_local(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::LocalPool;

    #[test]
    fn test_store_init() {
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();

        let store = pool.run_until(async move {
            let mut store = AppStore::new(spawner).await.unwrap();
            store.init().await.unwrap();
            store
        });

        let state = store.get_state().unwrap();
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn test_dispatch_create() {
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();

        let store = pool.run_until(async move {
            let mut store = AppStore::new(spawner).await.unwrap();
            store.init().await.unwrap();
            store
        });

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
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();
        let store = pool.run_until(async move {
            let mut store = AppStore::new(spawner).await.unwrap();
            store.init().await.unwrap();
            store
        });

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
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();
        let store = pool.run_until(async move {
            let mut store = AppStore::new(spawner).await.unwrap();
            store.init().await.unwrap();
            store
        });

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
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();
        let store = pool.run_until(async move {
            let mut store = AppStore::new(spawner).await.unwrap();
            store.init().await.unwrap();
            store
        });

        store
            .dispatch(Action::CreateTask {
                parent_id: None,
                title: "To Complete".to_string(),
            })
            .unwrap();

        let id = store.get_state().unwrap().root_task_ids[0].clone();
        store
            .dispatch(Action::CompleteTask { id: id.clone() })
            .unwrap();

        let state = store.get_state().unwrap();
        assert_eq!(state.tasks.get(&id).unwrap().status, TaskStatus::Done);
    }

    #[test]
    fn test_dispatch_move() {
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();
        let store = pool.run_until(async move {
            let mut store = AppStore::new(spawner).await.unwrap();
            store.init().await.unwrap();
            store
        });

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
    fn test_subscribe() {
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();

        pool.run_until(async move {
            // Setup store
            let mut store = AppStore::new(spawner).await.unwrap();
            store.init().await.unwrap();

            // Subscribe
            let stream = store.subscribe();
            let mut stream = Box::pin(stream);

            // Initial state
            let initial = stream.next().await.unwrap();
            assert!(initial.tasks.is_empty());

            // Dispatch
            store
                .dispatch(Action::CreateTask {
                    parent_id: None,
                    title: "Reactive Task".to_string(),
                })
                .unwrap();

            // Expect update
            let updated = stream.next().await.unwrap();
            assert_eq!(updated.tasks.len(), 1);
            assert_eq!(
                updated.tasks.values().next().unwrap().title,
                "Reactive Task"
            );
        });
    }
}

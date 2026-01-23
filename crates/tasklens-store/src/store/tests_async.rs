use samod::runtime::LocalRuntimeHandle;
use std::future::Future;
use std::pin::Pin;
use tasklens_core::{TaskID, TaskStatus, TunnelState};

use crate::store::{Action, AppStore, TaskUpdates};

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
    let mut store = AppStore::with_repo(repo.clone());
    // Create new doc using the detached pattern
    let (handle, id) = AppStore::create_new(repo).await.unwrap();
    store.set_active_doc(handle, id);
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

use crate::actions::{Action, TaskUpdates};
use crate::adapter::{
    self,
    tests::adapter_test_common::{check_invariants, init_doc},
};
use tasklens_core::types::{TaskID, TunnelState};

#[test]
fn test_create_task_on_absent_key() {
    let mut doc = init_doc().expect("Init failed");

    // Verify that Creating a task succeeds when the target ID is not already
    // present. This ensures that the existence check ("upsert" logic) correctly
    // handles missing keys in the Automerge tasks map.
    let res = adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Test".into(),
        },
    );

    assert!(res.is_ok(), "Action failed on absent key: {:?}", res.err());
}

#[test]
fn test_create_task_non_existent_parent() {
    let mut doc = init_doc().expect("Init failed");

    let res = adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: Some(TaskID::from("non-existent")),
            title: "Task with bad parent".into(),
        },
    );

    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("non-existent"));
}

#[test]
fn test_create_task_fails_if_exists() {
    let mut doc = init_doc().expect("Init failed");

    // Create a task
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Task 1".into(),
        },
    )
    .unwrap();

    // Create it again - should fail
    let res = adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Task 1 Updated".into(),
        },
    );

    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn test_create_task_cannot_move() {
    let mut doc = init_doc().expect("Init failed");

    // Create Task A as root
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-a"),
            parent_id: None,
            title: "Task A".into(),
        },
    )
    .unwrap();

    // Create Task P as root
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-p"),
            parent_id: None,
            title: "Task P".into(),
        },
    )
    .unwrap();

    // Re-create Task A as child of P (move via create) - should fail because A exists
    let res = adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-a"),
            parent_id: Some(TaskID::from("task-p")),
            title: "Task A moved".into(),
        },
    );

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("already exists"),
        "Expected 'already exists' error, got: {}",
        err_msg
    );
}

#[test]
fn test_update_task_non_existent() {
    let mut doc = init_doc().expect("Init failed");

    let res = adapter::dispatch(
        &mut doc,
        Action::UpdateTask {
            id: TaskID::from("non-existent"),
            updates: TaskUpdates {
                title: Some("New Title".into()),
                ..Default::default()
            },
        },
    );

    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Task not found"));

    // Verify it didn't create anything
    let state: TunnelState = autosurgeon::hydrate(&doc).unwrap();
    assert!(!state.tasks.contains_key(&TaskID::from("non-existent")));
}

#[test]
fn test_delete_task_non_existent() {
    let mut doc = init_doc().expect("Init failed");

    let res = adapter::dispatch(
        &mut doc,
        Action::DeleteTask {
            id: TaskID::from("non-existent"),
        },
    );

    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Task not found"));
}

#[test]
fn test_complete_task_non_existent() {
    let mut doc = init_doc().expect("Init failed");

    let res = adapter::dispatch(
        &mut doc,
        Action::CompleteTask {
            id: TaskID::from("non-existent"),
            current_time: 12345,
        },
    );

    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Task not found"));
}

#[test]
fn test_move_task_validations() {
    let mut doc = init_doc().expect("Init failed");

    // Create Task A
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-a"),
            parent_id: None,
            title: "Task A".into(),
        },
    )
    .unwrap();

    // 1. Move non-existent task
    let res = adapter::dispatch(
        &mut doc,
        Action::MoveTask {
            id: TaskID::from("non-existent"),
            new_parent_id: None,
        },
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("non-existent"));

    // 2. Move to non-existent parent
    let res = adapter::dispatch(
        &mut doc,
        Action::MoveTask {
            id: TaskID::from("task-a"),
            new_parent_id: Some(TaskID::from("non-existent")),
        },
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("non-existent"));

    // 3. Move task to itself
    let res = adapter::dispatch(
        &mut doc,
        Action::MoveTask {
            id: TaskID::from("task-a"),
            new_parent_id: Some(TaskID::from("task-a")),
        },
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("itself"));
}

#[test]
fn test_move_task_duplicate_prevention() {
    let mut doc = init_doc().expect("Init failed");

    // Create Task A as root
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-a"),
            parent_id: None,
            title: "Task A".into(),
        },
    )
    .unwrap();

    // Move to root again (should be no-op regarding list content)
    adapter::dispatch(
        &mut doc,
        Action::MoveTask {
            id: TaskID::from("task-a"),
            new_parent_id: None,
        },
    )
    .unwrap();

    let state: TunnelState = autosurgeon::hydrate(&doc).unwrap();
    assert_eq!(
        state
            .root_task_ids
            .iter()
            .filter(|id| id.as_str() == "task-a")
            .count(),
        1
    );
}

#[test]
fn test_move_cycle_bug() {
    let mut doc = init_doc().expect("Init failed");

    // 1. Create Task A
    adapter::dispatch(
        &mut doc,
        crate::actions::Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Task A".into(),
        },
    )
    .unwrap();

    // 2. Create Task B as child of A
    adapter::dispatch(
        &mut doc,
        crate::actions::Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: Some(TaskID::from("task-1")),
            title: "Task B".into(),
        },
    )
    .unwrap();

    // 3. Move A to be child of B (this should fail but might not)
    let _ = adapter::dispatch(
        &mut doc,
        crate::actions::Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-2")),
        },
    );

    // 4. Check invariants
    if let Err(msg) = check_invariants(&doc) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

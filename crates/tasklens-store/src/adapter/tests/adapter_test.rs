use crate::actions::{Action, TaskUpdates};
use crate::adapter::{
    self, AdapterError,
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

    assert!(matches!(
        res,
        Err(AdapterError::ParentNotFound(ref id)) if id.as_str() == "non-existent"
    ));
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

    assert!(matches!(
        res,
        Err(AdapterError::TaskExists(ref id)) if id.as_str() == "task-1"
    ));
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

    assert!(matches!(
        res,
        Err(AdapterError::TaskExists(ref id)) if id.as_str() == "task-a"
    ));
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

    assert!(matches!(
        res,
        Err(AdapterError::TaskNotFound(ref id)) if id.as_str() == "non-existent"
    ));

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

    assert!(matches!(
        res,
        Err(AdapterError::TaskNotFound(ref id)) if id.as_str() == "non-existent"
    ));
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

    assert!(matches!(
        res,
        Err(AdapterError::TaskNotFound(ref id)) if id.as_str() == "non-existent"
    ));
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
    assert!(matches!(
        res,
        Err(AdapterError::TaskNotFound(ref id)) if id.as_str() == "non-existent"
    ));

    // 2. Move to non-existent parent
    let res = adapter::dispatch(
        &mut doc,
        Action::MoveTask {
            id: TaskID::from("task-a"),
            new_parent_id: Some(TaskID::from("non-existent")),
        },
    );
    assert!(matches!(
        res,
        Err(AdapterError::ParentNotFound(ref id)) if id.as_str() == "non-existent"
    ));

    // 3. Move task to itself
    let res = adapter::dispatch(
        &mut doc,
        Action::MoveTask {
            id: TaskID::from("task-a"),
            new_parent_id: Some(TaskID::from("task-a")),
        },
    );
    assert!(matches!(
        res,
        Err(AdapterError::MoveToSelf(ref id, ref nid)) if id.as_str() == "task-a" && nid.as_str() == "task-a"
    ));
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

#[test]
fn test_routine_lead_time_overflow() {
    let mut doc = init_doc().expect("Init failed");
    use tasklens_core::types::{Frequency, RepeatConfig, ScheduleType};

    // 1. Create Task
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-5"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();

    // 2. Complete Task at a negative time
    adapter::dispatch(
        &mut doc,
        Action::CompleteTask {
            id: TaskID::from("task-5"),
            current_time: -280414227423804083,
        },
    )
    .unwrap();

    // 3. Update to Routinely with large lead_time
    adapter::dispatch(
        &mut doc,
        Action::UpdateTask {
            id: TaskID::from("task-5"),
            updates: TaskUpdates {
                schedule_type: Some(ScheduleType::Routinely),
                lead_time: Some(8942958376128571726),
                repeat_config: Some(Some(RepeatConfig {
                    frequency: Frequency::Minutes,
                    interval: 1,
                })),
                ..Default::default()
            },
        },
    )
    .unwrap();

    // 4. Refresh Lifecycle at time 0 - this should not panic
    adapter::dispatch(&mut doc, Action::RefreshLifecycle { current_time: 0 }).unwrap();
}

#[test]
fn test_move_task_to_root_removes_parent_id() {
    let mut doc = init_doc().expect("Init failed");

    // 1. Create Task 1 as root
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();

    // 2. Create Task 5 as child of Task 1
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-5"),
            parent_id: Some(TaskID::from("task-1")),
            title: "".into(),
        },
    )
    .unwrap();

    // 3. Move Task 5 to root
    adapter::dispatch(
        &mut doc,
        Action::MoveTask {
            id: TaskID::from("task-5"),
            new_parent_id: None,
        },
    )
    .unwrap();

    // 4. Check invariants
    if let Err(msg) = check_invariants(&doc) {
        panic!("Invariant Failure!\n{}", msg);
    }

    // 5. Verify parent_id is None in hydrated state
    let state: TunnelState = autosurgeon::hydrate(&doc).unwrap();
    let task = state.tasks.get(&TaskID::from("task-5")).unwrap();
    assert!(
        task.parent_id.is_none(),
        "Expected parent_id to be None, got Some({:?})",
        task.parent_id
    );
}

#[test]
fn test_concurrent_create_child_and_delete_parent_leak() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create Task 4 as root
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-4"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Create Task 5 as child of Task 4
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-5"),
            parent_id: Some(TaskID::from("task-4")),
            title: "".into(),
        },
    )
    .unwrap();

    // 3. Concurrent B: Delete Task 4
    adapter::dispatch(
        &mut doc_b,
        Action::DeleteTask {
            id: TaskID::from("task-4"),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Check invariants - this is where it should fail
    if let Err(msg) = check_invariants(&doc_a) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

#[test]
fn test_concurrent_delete_same_task_root() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create Task 3 and Task 4
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-3"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-4"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Delete Task 3
    adapter::dispatch(
        &mut doc_a,
        Action::DeleteTask {
            id: TaskID::from("task-3"),
        },
    )
    .unwrap();

    // 3. Concurrent B: Delete Task 3
    adapter::dispatch(
        &mut doc_b,
        Action::DeleteTask {
            id: TaskID::from("task-3"),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Check invariants
    if let Err(msg) = check_invariants(&doc_a) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

#[test]
fn test_concurrent_complete_task_status_merge() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create Task 5
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-5"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Complete Task 5
    adapter::dispatch(
        &mut doc_a,
        Action::CompleteTask {
            id: TaskID::from("task-5"),
            current_time: 1000,
        },
    )
    .unwrap();

    // 3. Concurrent B: Complete Task 5
    adapter::dispatch(
        &mut doc_b,
        Action::CompleteTask {
            id: TaskID::from("task-5"),
            current_time: 2000,
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Check invariants - this should fail during hydration if "Done" + "Done" = "DoDonee"
    if let Err(msg) = check_invariants(&doc_a) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

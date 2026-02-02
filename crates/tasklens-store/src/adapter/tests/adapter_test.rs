use crate::adapter::{
    self, AdapterError,
    tests::adapter_test_common::{HydrationStrategy, check_invariants, init_doc},
};
use tasklens_core::{
    Action, TaskUpdates,
    types::{TaskID, TunnelState},
};

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
    let state: TunnelState = crate::adapter::hydrate_tunnel_state_and_heal(&doc).unwrap();
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

    let state: TunnelState = crate::adapter::hydrate_tunnel_state_and_heal(&doc).unwrap();
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
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Task A".into(),
        },
    )
    .unwrap();

    // 2. Create Task B as child of A
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: Some(TaskID::from("task-1")),
            title: "Task B".into(),
        },
    )
    .unwrap();

    // 3. Move A to be child of B (this should fail but might not)
    let _ = adapter::dispatch(
        &mut doc,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-2")),
        },
    );

    // 4. Check invariants
    if let Err(msg) = check_invariants(&doc, HydrationStrategy::Strict) {
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
    if let Err(msg) = check_invariants(&doc, HydrationStrategy::Strict) {
        panic!("Invariant Failure!\n{}", msg);
    }

    // 5. Verify parent_id is None in hydrated state
    let state: TunnelState = crate::adapter::hydrate_tunnel_state_and_heal(&doc).unwrap();
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

    // 5. Check invariants - using Heal strategy because concurrent delete is a known structural hazard
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
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

    // 5. Check invariants - using Heal strategy because concurrent delete is a known structural hazard
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
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
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Strict) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

#[test]
fn test_delete_task_cascades() {
    let mut doc = init_doc().expect("Init failed");

    // 1. Create Parent
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("parent"),
            parent_id: None,
            title: "Parent".into(),
        },
    )
    .unwrap();

    // 2. Create Child
    adapter::dispatch(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from("child"),
            parent_id: Some(TaskID::from("parent")),
            title: "Child".into(),
        },
    )
    .unwrap();

    // 3. Delete Parent
    adapter::dispatch(
        &mut doc,
        Action::DeleteTask {
            id: TaskID::from("parent"),
        },
    )
    .unwrap();

    // 4. Verify both are gone from the tasks map
    let state: TunnelState = crate::adapter::hydrate_tunnel_state_and_heal(&doc).unwrap();
    assert!(
        !state.tasks.contains_key(&TaskID::from("parent")),
        "Parent should be deleted"
    );
    assert!(
        !state.tasks.contains_key(&TaskID::from("child")),
        "Child should be deleted (cascaded)"
    );

    // 5. Verify child is NOT in root_task_ids (common bug when promoting instead of cascading)
    assert!(
        !state.root_task_ids.contains(&TaskID::from("child")),
        "Child should NOT be promoted to root"
    );

    // 6. Verify there are no tasks at all.
    assert!(state.tasks.is_empty(), "Tasks should be empty");
    assert!(
        state.root_task_ids.is_empty(),
        "Root task ids should be empty"
    );
}

#[test]
fn test_concurrent_move_and_delete_child_link_leak() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create Task 1 (the victim)
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Victim".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Create Task 5 and move Task 1 under it
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-5"),
            parent_id: None,
            title: "New Parent".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-5")),
        },
    )
    .unwrap();

    // 3. Concurrent B: Delete Task 1
    adapter::dispatch(
        &mut doc_b,
        Action::DeleteTask {
            id: TaskID::from("task-1"),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Assert: Invariants held (using Heal strategy)
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

#[test]
fn test_concurrent_create_grandchild_and_delete_ancestor_leak() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create Task 0
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-0"),
            parent_id: None,
            title: "Ancestor".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Delete Task 0
    adapter::dispatch(
        &mut doc_a,
        Action::DeleteTask {
            id: TaskID::from("task-0"),
        },
    )
    .unwrap();

    // 3. Concurrent B: Create Task 1 (child of 0) and Task 2 (child of 1)
    adapter::dispatch(
        &mut doc_b,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: Some(TaskID::from("task-0")),
            title: "Child".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_b,
        Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: Some(TaskID::from("task-1")),
            title: "Grandchild".into(),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Check invariants with Heal
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

#[test]
fn test_concurrent_move_to_locally_deleted_parent_leak() {
    let mut doc_a = init_doc().expect("Init failed");

    // Setup: 3 tasks
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-0"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: None,
            title: "".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // Concurrent A: Delete task-0
    adapter::dispatch(
        &mut doc_a,
        Action::DeleteTask {
            id: TaskID::from("task-0"),
        },
    )
    .unwrap();

    // Concurrent B:
    // Move task-1 to parent task-2
    adapter::dispatch(
        &mut doc_b,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-2")),
        },
    )
    .unwrap();
    // Delete task-2
    adapter::dispatch(
        &mut doc_b,
        Action::DeleteTask {
            id: TaskID::from("task-2"),
        },
    )
    .unwrap();

    doc_a.merge(&mut doc_b).expect("Merge failed");
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

#[test]
fn test_concurrent_move_to_root_duplicate_id() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create Task P as root, and Task A as child of P
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-p"),
            parent_id: None,
            title: "Parent".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-a"),
            parent_id: Some(TaskID::from("task-p")),
            title: "Child A".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Move Task A to root
    adapter::dispatch(
        &mut doc_a,
        Action::MoveTask {
            id: TaskID::from("task-a"),
            new_parent_id: None,
        },
    )
    .unwrap();

    // 3. Concurrent B: Move Task A to root
    adapter::dispatch(
        &mut doc_b,
        Action::MoveTask {
            id: TaskID::from("task-a"),
            new_parent_id: None,
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Check invariants
    // This is expected to PASS because Heal strategy deduplicates the concurrent rootTaskIds entries.
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

#[test]
fn test_concurrent_move_inconsistency_regression() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create Task 1 and Task 2
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Task 1".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: None,
            title: "Task 2".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Create Task 3 and move Task 1 under it
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-3"),
            parent_id: None,
            title: "Task 3".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-3")),
        },
    )
    .unwrap();

    // 3. Concurrent B: Move Task 1 under Task 2
    adapter::dispatch(
        &mut doc_b,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-2")),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Assert: Invariants held
    // We expect this to fail with Strict hydration if the fix is not present.
    // We use Heal strategy here to verify that our healing logic handles the concurrent move inconsistency.
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!("Invariant Failure!\n{}", msg);
    }
}

#[test]
fn test_id_corruption_minimal_repro() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create Task 1 and Task 2. rootTaskIds = ["task-1", "task-2"]
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Task 1".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: None,
            title: "Task 2".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Delete Task 1. Replica A's rootTaskIds becomes ["task-2"]
    adapter::dispatch(
        &mut doc_a,
        Action::DeleteTask {
            id: TaskID::from("task-1"),
        },
    )
    .unwrap();

    // 3. Concurrent B: Delete Task 1. Replica B's rootTaskIds becomes ["task-2"]
    adapter::dispatch(
        &mut doc_b,
        Action::DeleteTask {
            id: TaskID::from("task-1"),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Assert: Invariants held
    // This will pass now because we've fixed ID corruption (scalar)
    // and we're using Heal strategy for structural inconsistencies.
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!("Invariant Failure (ID corruption fix failed)!\n{}", msg);
    }
}

#[test]
fn test_multiple_parents_minimal_repro() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create tasks 1, 2, 3 as roots
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Task 1".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: None,
            title: "Task 2".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-3"),
            parent_id: None,
            title: "Task 3".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Move task-1 to parent task-2
    adapter::dispatch(
        &mut doc_a,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-2")),
        },
    )
    .unwrap();

    // 3. Concurrent B: Move task-1 to parent task-3
    adapter::dispatch(
        &mut doc_b,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-3")),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Assert: Invariants held
    // We use Heal strategy here to verify that our healing logic handles the concurrent move inconsistency.
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!("Invariant Failure (Multiple Parents heal failed)!\n{}", msg);
    }
}

#[test]
fn test_multiple_parents_child_list_minimal_repro() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create task-0, and tasks 1, 2, 3 as children of task-0
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-0"),
            parent_id: None,
            title: "Root".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: Some(TaskID::from("task-0")),
            title: "Child 1".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: Some(TaskID::from("task-0")),
            title: "Child 2".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-3"),
            parent_id: Some(TaskID::from("task-0")),
            title: "Child 3".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Move task-1 to parent task-2
    adapter::dispatch(
        &mut doc_a,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-2")),
        },
    )
    .unwrap();

    // 3. Concurrent B: Move task-1 to parent task-3
    adapter::dispatch(
        &mut doc_b,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-3")),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Assert: Invariants held
    // We use Heal strategy here to verify that our healing logic handles the concurrent move inconsistency.
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!(
            "Invariant Failure (Child list Multiple Parents heal failed)!\n{}",
            msg
        );
    }
}

/// Regression test for cycle detection after merge.
///
/// This reproduces a bug found by proptest where concurrent moves create a
/// cycle that is unreachable from roots:
///
/// - Replica A moves task-1 under task-2
/// - Replica B moves task-2 under task-1
///
/// After merge, task-1 → task-2 → task-1 forms a closed loop.
#[test]
fn test_concurrent_moves_create_cycle() {
    let mut doc_a = init_doc().expect("Init failed");

    // 1. Setup: Create task-1 and task-2 as root tasks
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-1"),
            parent_id: None,
            title: "Task 1".into(),
        },
    )
    .unwrap();
    adapter::dispatch(
        &mut doc_a,
        Action::CreateTask {
            id: TaskID::from("task-2"),
            parent_id: None,
            title: "Task 2".into(),
        },
    )
    .unwrap();

    let mut doc_b = doc_a.fork();

    // 2. Concurrent A: Move task-1 under task-2
    adapter::dispatch(
        &mut doc_a,
        Action::MoveTask {
            id: TaskID::from("task-1"),
            new_parent_id: Some(TaskID::from("task-2")),
        },
    )
    .unwrap();

    // 3. Concurrent B: Move task-2 under task-1
    adapter::dispatch(
        &mut doc_b,
        Action::MoveTask {
            id: TaskID::from("task-2"),
            new_parent_id: Some(TaskID::from("task-1")),
        },
    )
    .unwrap();

    // 4. Merge
    doc_a.merge(&mut doc_b).expect("Merge failed");

    // 5. Assert: Invariants held - the healing logic must break the cycle
    if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
        panic!("Invariant Failure (Cycle after concurrent moves)!\n{}", msg);
    }
}

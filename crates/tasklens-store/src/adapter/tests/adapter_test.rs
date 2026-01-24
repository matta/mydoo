use crate::actions::Action;
use crate::adapter::{
    self,
    tests::adapter_test_common::{check_invariants, init_doc},
};
use tasklens_core::types::TaskID;

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
#[ignore]
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

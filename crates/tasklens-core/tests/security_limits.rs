use automerge::AutoCommit;
use tasklens_core::domain::dispatch::{DispatchError, run_action};
use tasklens_core::domain::doc_bridge;
use tasklens_core::types::{PlaceID, TaskID, TunnelState};
use tasklens_core::Action;
use std::collections::HashMap;

#[test]
fn test_long_task_id_creation_fails() {
    let mut doc = AutoCommit::new();
    let initial = TunnelState {
        tasks: HashMap::new(),
        places: HashMap::new(),
        root_task_ids: Vec::new(),
        metadata: None,
    };
    doc_bridge::reconcile_tunnel_state(&mut doc, &initial).unwrap();

    let long_id = "a".repeat(101); // MAX_ID_LENGTH is 100
    let result = run_action(
        &mut doc,
        Action::CreateTask {
            id: TaskID::from(long_id),
            parent_id: None,
            title: "Task with long ID".to_string(),
        },
    );

    assert!(matches!(result, Err(DispatchError::InvalidInput(_))));
}

#[test]
fn test_long_place_id_creation_fails() {
    let mut doc = AutoCommit::new();
    let initial = TunnelState {
        tasks: HashMap::new(),
        places: HashMap::new(),
        root_task_ids: Vec::new(),
        metadata: None,
    };
    doc_bridge::reconcile_tunnel_state(&mut doc, &initial).unwrap();

    let long_id = "a".repeat(101); // MAX_ID_LENGTH is 100
    let result = run_action(
        &mut doc,
        Action::CreatePlace {
            id: PlaceID::from(long_id),
            name: "Place with long ID".to_string(),
            hours: r#"{"mode":"always_open"}"#.to_string(),
            included_places: vec![],
        },
    );

    assert!(matches!(result, Err(DispatchError::InvalidInput(_))));
}

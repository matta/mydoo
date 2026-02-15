#[cfg(test)]
mod tests {
    use crate::domain::dispatch::{DispatchError, run_action};
    use crate::domain::doc_bridge;
    use crate::types::{TaskID, TaskStatus, TunnelState};
    use crate::{Action, TaskUpdates};
    use automerge::AutoCommit;
    use std::collections::HashMap;

    fn new_doc() -> AutoCommit {
        let mut doc = AutoCommit::new();
        let initial = TunnelState {
            tasks: HashMap::new(),
            places: HashMap::new(),
            root_task_ids: Vec::new(),
            metadata: None,
        };
        doc_bridge::reconcile_tunnel_state(&mut doc, &initial).unwrap();
        doc
    }

    #[test]
    fn test_create_task_empty_title() {
        let mut doc = new_doc();
        let result = run_action(
            &mut doc,
            Action::CreateTask {
                id: TaskID::new(),
                parent_id: None,
                title: "".to_string(), // Empty title
            },
        );
        assert!(matches!(result, Err(DispatchError::InvalidInput(_))));
    }

    #[test]
    fn test_update_task_invalid_title() {
        let mut doc = new_doc();
        let id = TaskID::new();
        run_action(
            &mut doc,
            Action::CreateTask {
                id: id.clone(),
                parent_id: None,
                title: "Valid Title".to_string(),
            },
        )
        .unwrap();

        let result = run_action(
            &mut doc,
            Action::UpdateTask {
                id: id.clone(),
                updates: TaskUpdates {
                    title: Some("".to_string()),
                    ..Default::default()
                },
            },
        );
        assert!(matches!(result, Err(DispatchError::InvalidInput(_))));
    }

    #[test]
    fn test_update_task_huge_notes() {
        let mut doc = new_doc();
        let id = TaskID::new();
        run_action(
            &mut doc,
            Action::CreateTask {
                id: id.clone(),
                parent_id: None,
                title: "Valid Title".to_string(),
            },
        )
        .unwrap();

        // MAX_NOTES_LENGTH is 50,000
        let huge_notes = "a".repeat(50_001);
        let result = run_action(
            &mut doc,
            Action::UpdateTask {
                id: id.clone(),
                updates: TaskUpdates {
                    notes: Some(huge_notes),
                    ..Default::default()
                },
            },
        );
        assert!(matches!(result, Err(DispatchError::InvalidInput(_))));
    }

    #[test]
    fn test_create_place_invalid_name() {
        let mut doc = new_doc();
        use crate::types::PlaceID;
        let result = run_action(
            &mut doc,
            Action::CreatePlace {
                id: PlaceID::new(),
                name: "".to_string(),
                hours: "{}".to_string(),
                included_places: vec![],
            },
        );
        assert!(matches!(result, Err(DispatchError::InvalidInput(_))));
    }

    #[test]
    fn test_update_place_huge_name() {
        let mut doc = new_doc();
        use crate::types::PlaceID;
        let id = PlaceID::new();
        run_action(
            &mut doc,
            Action::CreatePlace {
                id: id.clone(),
                name: "Valid Place".to_string(),
                hours: "{}".to_string(),
                included_places: vec![],
            },
        )
        .unwrap();

        // MAX_PLACE_NAME_LENGTH is 100
        let huge_name = "a".repeat(101);
        use crate::PlaceUpdates;
        let result = run_action(
            &mut doc,
            Action::UpdatePlace {
                id,
                updates: PlaceUpdates {
                    name: Some(huge_name),
                    ..Default::default()
                },
            },
        );
        assert!(matches!(result, Err(DispatchError::InvalidInput(_))));
    }

    #[test]
    fn test_create_task_huge_title() {
        let mut doc = new_doc();
        // MAX_TITLE_LENGTH is 500
        let huge_title = "a".repeat(501);
        let result = run_action(
            &mut doc,
            Action::CreateTask {
                id: TaskID::new(),
                parent_id: None,
                title: huge_title,
            },
        );
        assert!(matches!(result, Err(DispatchError::InvalidInput(_))));
    }
}

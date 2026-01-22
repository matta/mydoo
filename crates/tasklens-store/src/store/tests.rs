use std::collections::HashMap;

use anyhow::{Result, anyhow};
use automerge::AutoCommit;
use automerge_test::{assert_doc, list, map};
use tasklens_core::{TaskID, TaskStatus, TunnelState};

use crate::store::{Action, ensure_path};
use automerge_test::RealizedObject;
use std::collections::BTreeSet;

fn am_text(s: &str) -> RealizedObject {
    let seq = s
        .chars()
        .map(|c| {
            let mut set = BTreeSet::new();
            set.insert(RealizedObject::from(c.to_string().as_str()));
            set
        })
        .collect();
    RealizedObject::Sequence(seq)
}

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
        autosurgeon::reconcile(&mut self.doc, &initial_state)
            .map_err(|e| anyhow!("Init failed: {}", e))
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
        autosurgeon::reconcile(&mut self.doc, state)
            .map_err(|e| anyhow!("Reconciliation failed: {}", e))
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
                            "id" => { am_text(&task_id_str) },
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
                    { am_text(&task_id_str) }
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
                            "id" => { am_text(&parent_id_str) },
                            "title" => { "Parent" },
                            "childTaskIds" => {
                                list![
                                    { am_text(&child1_id_str) },
                                    { am_text(&child2_id_str) }
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
                            "id" => { am_text(&child1_id_str) },
                            "title" => { "Child 1" },
                            "childTaskIds" => { list![] },
                            "status" => { "Pending" },
                            "notes" => { "" },
                            "parentId" => { am_text(&parent_id_str) },
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
                            "id" => { am_text(&child2_id_str) },
                            "title" => { "Child 2" },
                            "childTaskIds" => { list![] },
                            "status" => { "Pending" },
                            "notes" => { "" },
                            "parentId" => { am_text(&parent_id_str) },
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
                    { am_text(&parent_id_str) }
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
                            "id" => { am_text(&task_id_str) },
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
                    { am_text(&task_id_str) }
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
                            "id" => { am_text(parent_id.as_str()) },
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
                list![ { am_text(parent_id.as_str()) } ]
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
                            "id" => { am_text(&task_id_str) },
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
                    { am_text(&task_id_str) }
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
                            "id" => { am_text(&parent_id_str) },
                            "title" => { "Parent" },
                            "status" => { "Pending" },
                            "childTaskIds" => {
                                list![
                                    { am_text(&child_id_str) }
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
                            "id" => { am_text(&child_id_str) },
                            "title" => { "Child" },
                            "status" => { "Pending" },
                            "childTaskIds" => { list![] },
                            "notes" => { "" },
                            "parentId" => { am_text(&parent_id_str) },
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
                    { am_text(&parent_id_str) }
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
                            "id" => { am_text(&task_id_str) },
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
                    { am_text(&task_id_str) }
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
                            "id" => { am_text(&task_id_str) },
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
                    { am_text(&task_id_str) }
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
                            "id" => { am_text(&task_id_str) },
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
                    { am_text(&task_id_str) }
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
                            "id" => { am_text(&task_id_str) },
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
                    { am_text(&task_id_str) }
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

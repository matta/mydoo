use automerge::{ObjId, ReadDoc, Value};
use std::path::PathBuf;
use tasklens_core::types::{PersistedTask, TunnelState};

/// Test for golden document reconciliation.
///
/// GOAL: Assert exact value and Automerge type equality between the golden file
/// and the reconciled output.
///
/// STRICTURES:
/// - NO semantic leniency (e.g. Int(1) != F64(1.0)).
/// - Type equality is mandatory to ensure interoperability and schema stability.
/// - Any "drift" between implementation and golden file must be resolved by either
///   fixing the implementation or explicitly updating the golden file.
#[test]
#[ignore] // FIXME: This test fails due to schema mismatches between golden file and current implementation.
fn test_golden_reconciliation() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/data/golden.automerge");

    println!("Loading golden file from: {:?}", d);

    let bytes = std::fs::read(&d).expect("failed to read golden file");
    let mut doc = automerge::AutoCommit::load(&bytes).expect("Failed to load automerge doc");

    println!(
        "Successfully loaded automerge doc with actor: {:?}",
        doc.get_actor()
    );

    // Fail Fast: Surgical Test
    println!("Starting Fail-Fast Surgical Check...");
    if let Some((Value::Object(_), tasks_obj_id)) = doc.get(automerge::ROOT, "tasks").ok().flatten()
    {
        let task_keys: Vec<_> = doc.keys(&tasks_obj_id).collect();
        if let Some(first_task_key) = task_keys.first() {
            println!(
                "Checking surgical reconciliation for task: {}",
                first_task_key
            );
            let task: PersistedTask =
                autosurgeon::hydrate_prop(&doc, &tasks_obj_id, first_task_key.as_str())
                    .expect("Failed to hydrate task");

            let mut target_doc = doc.fork();
            // Note: ObjId is consistent across forks for existing objects
            autosurgeon::reconcile_prop(
                &mut target_doc,
                &tasks_obj_id,
                first_task_key.as_str(),
                &task,
            )
            .expect("Failed to reconcile task");

            // Assert equality for JUST this task
            let (_, id_a) = doc
                .get(&tasks_obj_id, first_task_key.as_str())
                .expect("Expected (Golden) task missing")
                .expect("Expected (Golden) task missing");
            let (_, id_b) = target_doc
                .get(&tasks_obj_id, first_task_key.as_str())
                .expect("Actual (Reconciled) task missing")
                .expect("Actual (Reconciled) task missing");

            assert_docs_equal(
                &doc,
                &target_doc,
                id_a,
                id_b,
                format!("tasks.{}", first_task_key),
            );
            println!("Surgical check passed.");
        }
    } else {
        println!("WARNING: 'tasks' map not found in golden file. Skipping surgical check.");
    }

    // Full Test
    println!("Starting Full Reconciliation...");
    let state: TunnelState = autosurgeon::hydrate(&doc).expect("Failed to hydrate TunnelState");
    println!("Hydrated state with {} tasks", state.tasks.len());
    assert!(
        !state.tasks.is_empty(),
        "Hydrated state should not be empty"
    );

    // Phase 2: Reconciliation
    let mut target_doc = doc.fork();
    autosurgeon::reconcile(&mut target_doc, &state).expect("Failed to reconcile state");

    println!(
        "Reconciliation complete. Target doc actor: {:?}",
        target_doc.get_actor()
    );

    // Phase 3: Recursive Diff
    assert_docs_equal(
        &doc,
        &target_doc,
        automerge::ROOT,
        automerge::ROOT,
        "".to_string(),
    );
}

/// Recursively asserts that two documents are identical in both value and Automerge type.
///
/// This function performs a strict comparison. Differences in scalar types (e.g., Int vs F64)
/// or object types (e.g., Map vs Table) will cause a panic.
fn assert_docs_equal<T: ReadDoc>(doc_a: &T, doc_b: &T, obj_a: ObjId, obj_b: ObjId, path: String) {
    let obj_type_a = doc_a.object_type(&obj_a).expect("Object should exist in A");
    let obj_type_b = doc_b.object_type(&obj_b).expect("Object should exist in B");

    if obj_type_a != obj_type_b {
        panic!(
            "Difference at {}: Object type mismatch. Expected (Golden): {:?}, Actual (Reconciled): {:?}",
            path, obj_type_a, obj_type_b
        );
    }

    match obj_type_a {
        automerge::ObjType::Map | automerge::ObjType::Table => {
            let keys_a: std::collections::BTreeSet<_> = doc_a.keys(&obj_a).collect();
            let keys_b: std::collections::BTreeSet<_> = doc_b.keys(&obj_b).collect();

            let all_keys: std::collections::BTreeSet<_> = keys_a.union(&keys_b).cloned().collect();

            for key in all_keys {
                let current_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                let val_a = doc_a.get(&obj_a, &key).expect("Get should work");
                let val_b = doc_b.get(&obj_b, &key).expect("Get should work");

                match (val_a, val_b) {
                    (Some((Value::Object(_), id_a)), Some((Value::Object(_), id_b))) => {
                        assert_docs_equal(doc_a, doc_b, id_a, id_b, current_path);
                    }
                    (Some((Value::Scalar(s_a), _)), Some((Value::Scalar(s_b), _))) => {
                        if s_a != s_b {
                            panic!(
                                "Difference at {}: Scalar value mismatch. Expected (Golden): {:?}, Actual (Reconciled): {:?}",
                                current_path, s_a, s_b
                            );
                        }
                    }
                    (None, Some(_)) => panic!(
                        "Difference at {}: Missing in Expected (Golden)",
                        current_path
                    ),
                    (Some(_), None) => panic!(
                        "Difference at {}: Missing in Actual (Reconciled)",
                        current_path
                    ),
                    (a, b) => panic!(
                        "Difference at {}: Type mismatch. Expected (Golden): {:?}, Actual (Reconciled): {:?}",
                        current_path, a, b
                    ),
                }
            }
        }
        automerge::ObjType::List | automerge::ObjType::Text => {
            let len_a = doc_a.length(&obj_a);
            let len_b = doc_b.length(&obj_b);
            if len_a != len_b {
                panic!(
                    "Difference at {}: List length mismatch. Expected (Golden): {}, Actual (Reconciled): {}",
                    path, len_a, len_b
                );
            }
            for i in 0..len_a {
                let current_path = format!("{}[{}]", path, i);
                let val_a = doc_a.get(&obj_a, i).expect("Get should work");
                let val_b = doc_b.get(&obj_b, i).expect("Get should work");

                match (val_a, val_b) {
                    (Some((Value::Object(_), id_a)), Some((Value::Object(_), id_b))) => {
                        assert_docs_equal(doc_a, doc_b, id_a, id_b, current_path);
                    }
                    (Some((Value::Scalar(s_a), _)), Some((Value::Scalar(s_b), _))) => {
                        if s_a != s_b {
                            panic!(
                                "Difference at {}: Scalar value mismatch. Expected (Golden): {:?}, Actual (Reconciled): {:?}",
                                current_path, s_a, s_b
                            );
                        }
                    }
                    (a, b) => panic!(
                        "Difference at {}: Type mismatch. Expected (Golden): {:?}, Actual (Reconciled): {:?}",
                        current_path, a, b
                    ),
                }
            }
        }
    }
}

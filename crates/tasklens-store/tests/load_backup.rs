use automerge::ReadDoc;
use std::path::PathBuf;
use tasklens_store::store::AppStore;

#[test]
fn test_load_golden_file() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/data/golden.automerge");

    println!("Loading golden file from: {:?}", d);

    let bytes = std::fs::read(&d).expect("failed to read golden file");

    // Load directly into AutoCommit to inspect without hydration error
    let doc = automerge::AutoCommit::load(&bytes).expect("Failed to load automerge doc");

    // Debug: Inspect root keys and values
    println!("DEBUG: Inspecting root values:");
    let keys: Vec<_> = doc.keys(automerge::ROOT).collect();
    for key in keys {
        let val = doc.get(automerge::ROOT, &key);
        println!("Key: {}, Value: {:?}", key, val);
        if let Ok(Some((automerge::Value::Scalar(s), _))) = val {
            println!("  Type: {:?}", s);
        }
    }

    // Inspect specific tasks to find the text mismatch
    if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), tasks_id))) =
        doc.get(automerge::ROOT, "tasks")
    {
        let task_keys: Vec<_> = doc.keys(&tasks_id).collect();
        println!("Found {} tasks", task_keys.len());
        if let Some(first_task_key) = task_keys.first() {
            println!("Inspecting first task: {}", first_task_key);
            if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), task_obj))) =
                doc.get(&tasks_id, first_task_key)
            {
                for prop in doc.keys(&task_obj) {
                    let val = doc.get(&task_obj, &prop);
                    println!("  Prop: {}, Value: {:?}", prop, val);
                    if let Ok(Some((automerge::Value::Scalar(s), _))) = val {
                        println!("    Type: {:?}", s);
                    }
                }

                // Inspect schedule specifically
                if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), sched_id))) =
                    doc.get(&task_obj, "schedule")
                {
                    println!("  Inspecting Schedule:");
                    for prop in doc.keys(&sched_id) {
                        let val = doc.get(&sched_id, &prop);
                        println!("    Sched Prop: {}, Value: {:?}", prop, val);
                    }
                }
            }
        } // closes if first_task_key

        // Scan ALL tasks for anomalies
        println!(
            "Scanning all {} tasks for type anomalies...",
            task_keys.len()
        );
        for key in task_keys {
            if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), task_obj))) =
                doc.get(&tasks_id, &key)
            {
                // Check specific fields that might be Maps unexpectedly
                for prop in [
                    "status",
                    "id",
                    "title",
                    "importance",
                    "credits",
                    "creditsTimestamp",
                    "repeatConfig",
                ] {
                    if let Ok(Some((val, _))) = doc.get(&task_obj, prop)
                        && matches!(val, automerge::Value::Object(automerge::ObjType::Map))
                    {
                        if prop != "repeatConfig" {
                            // repeatConfig IS expected to be a map if present
                            println!(
                                "ALERT: Task {} field {} is a MAP! Value: {:?}",
                                key, prop, val
                            );
                        } else {
                            // Inspect repeatConfig content
                            println!("Task {} has repeatConfig. Inspecting...", key);
                            if let Ok(Some((
                                automerge::Value::Object(automerge::ObjType::Map),
                                rc_id,
                            ))) = doc.get(&task_obj, "repeatConfig")
                            {
                                for rc_prop in doc.keys(&rc_id) {
                                    let rc_val = doc.get(&rc_id, &rc_prop);
                                    println!("    RC Prop: {}, Value: {:?}", rc_prop, rc_val);
                                    if let Ok(Some((
                                        automerge::Value::Object(automerge::ObjType::Map),
                                        _,
                                    ))) = rc_val
                                    {
                                        println!(
                                            "    ALERT: repeatConfig field {} is a MAP!",
                                            rc_prop
                                        );
                                    }
                                }
                            }
                        }
                    }
                }

                // Inspect childTaskIds for one task
                if let Ok(Some((
                    automerge::Value::Object(automerge::ObjType::List),
                    child_ids_id,
                ))) = doc.get(&task_obj, "childTaskIds")
                {
                    let len = doc.length(&child_ids_id);
                    if len > 0 {
                        println!("Task {} has {} child tasks. Inspecting first...", key, len);
                        let val = doc.get(&child_ids_id, 0);
                        println!("  Child ID [0]: {:?}", val);
                        if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), _))) =
                            val
                        {
                            println!("  ALERT: Child ID is a MAP!");
                        }
                    }
                }
            } else {
                println!("ALERT: Task {} is NOT a Map!", key);
            }
        }
    } // closes if tasks_id

    // Inspect rootTaskIds
    if let Ok(Some((automerge::Value::Object(automerge::ObjType::List), root_ids_id))) =
        doc.get(automerge::ROOT, "rootTaskIds")
    {
        let len = doc.length(&root_ids_id);
        println!("Found {} root tasks", len);
        if len > 0 {
            let val = doc.get(&root_ids_id, 0);
            println!("  Root Task ID [0]: {:?}", val);
            if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), _))) = val {
                println!("  ALERT: Root Task ID is a MAP!");
            }
        }
    }

    // Inspect Metadata
    if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), meta_id))) =
        doc.get(automerge::ROOT, "metadata")
    {
        println!("Inspecting Metadata:");
        for prop in doc.keys(&meta_id) {
            let val = doc.get(&meta_id, &prop);
            println!("  Meta Prop: {}, Value: {:?}", prop, val);
        }
    }

    // Inspect places
    if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), places_id))) =
        doc.get(automerge::ROOT, "places")
    {
        let place_keys: Vec<_> = doc.keys(&places_id).collect();
        println!("Found {} places", place_keys.len());
        if let Some(first_place_key) = place_keys.first() {
            println!("Inspecting first place: {}", first_place_key);
            if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), place_obj))) =
                doc.get(&places_id, first_place_key)
            {
                for prop in doc.keys(&place_obj) {
                    let val = doc.get(&place_obj, &prop);
                    println!("  Place Prop: {}, Value: {:?}", prop, val);
                }
            }
        }
    }

    // Attempt import to trigger the error (for now, we expect this to fail until we fix it)
    let mut store = AppStore::new();
    match store.import_doc(bytes) {
        Ok(_) => println!("Import successful!"),
        Err(e) => {
            println!("Import failed as expected: {:?}", e);
            // Don't panic here so we can see the output
        }
    }

    // Additional assertions could go here
    let state: tasklens_core::types::TunnelState = store.hydrate().expect("Failed to hydrate");
    println!("Successfully loaded. Task count: {}", state.tasks.len());
}

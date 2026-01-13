use serde_json::json;
use tasklens_core::TunnelState;

fn values_are_equivalent(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match (a, b) {
        (serde_json::Value::Number(an), serde_json::Value::Number(bn)) => {
            an.as_f64() == bn.as_f64()
        }
        (serde_json::Value::Object(ao), serde_json::Value::Object(bo)) => {
            if ao.len() != bo.len() {
                return false;
            }
            for (k, av) in ao {
                match bo.get(k) {
                    Some(bv) => {
                        if !values_are_equivalent(av, bv) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            true
        }
        (serde_json::Value::Array(aa), serde_json::Value::Array(ba)) => {
            if aa.len() != ba.len() {
                return false;
            }
            for (av, bv) in aa.iter().zip(ba.iter()) {
                if !values_are_equivalent(av, bv) {
                    return false;
                }
            }
            true
        }
        _ => a == b,
    }
}

#[test]
fn test_backup_roundtrip() {
    let json_str = include_str!("fixtures/medieval_tasks.json");
    let original_value: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let state: TunnelState = serde_json::from_str(json_str).unwrap();
    let serialized_value = serde_json::to_value(&state).unwrap();

    if !values_are_equivalent(&serialized_value, &original_value) {
        // Print a small part of the diff to help debug
        if let (Some(orig_tasks), Some(ser_tasks)) =
            (original_value.get("tasks"), serialized_value.get("tasks"))
        {
            if !values_are_equivalent(orig_tasks, ser_tasks) {
                eprintln!("Diff in tasks!");
                if let (Some(orig_obj), Some(ser_obj)) =
                    (orig_tasks.as_object(), ser_tasks.as_object())
                {
                    for (k, v) in orig_obj {
                        let sv = ser_obj.get(k).unwrap_or(&json!(null));
                        if !values_are_equivalent(v, sv) {
                            eprintln!("Task {} differs!", k);
                            eprintln!("Original: {}", serde_json::to_string_pretty(v).unwrap());
                            eprintln!("Serialized: {}", serde_json::to_string_pretty(sv).unwrap());
                            break;
                        }
                    }
                }
            }
        } else {
            eprintln!("Top level diff!");
        }
        panic!("Information loss during roundtrip!");
    }
}

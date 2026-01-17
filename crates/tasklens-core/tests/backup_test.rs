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
        let mut err_msg = String::from("Information loss during roundtrip!\n");

        if let (Some(orig_obj), Some(ser_obj)) =
            (original_value.as_object(), serialized_value.as_object())
        {
            let orig_keys: std::collections::HashSet<_> = orig_obj.keys().collect();
            let ser_keys: std::collections::HashSet<_> = ser_obj.keys().collect();

            let missing: Vec<_> = orig_keys.difference(&ser_keys).collect();
            let extra: Vec<_> = ser_keys.difference(&orig_keys).collect();

            if !missing.is_empty() {
                err_msg.push_str(&format!("Missing keys in serialized: {:?}\n", missing));
            }
            if !extra.is_empty() {
                err_msg.push_str(&format!("Extra keys in serialized: {:?}\n", extra));
            }

            for (k, v) in orig_obj {
                let sv = ser_obj.get(k).unwrap_or(&serde_json::json!(null));
                if !values_are_equivalent(v, sv) {
                    err_msg.push_str(&format!("Key '{}' differs!\n", k));

                    if k == "tasks" {
                        if let (Some(orig_tasks_obj), Some(ser_tasks_obj)) =
                            (v.as_object(), sv.as_object())
                        {
                            for (tk, tv) in orig_tasks_obj {
                                let tsv = ser_tasks_obj.get(tk).unwrap_or(&serde_json::json!(null));
                                if !values_are_equivalent(tv, tsv) {
                                    err_msg.push_str(&format!("  Task {} differs!\n", tk));
                                    err_msg.push_str(&format!(
                                        "  Original: {}\n",
                                        serde_json::to_string_pretty(tv).unwrap()
                                    ));
                                    err_msg.push_str(&format!(
                                        "  Serialized: {}\n",
                                        serde_json::to_string_pretty(tsv).unwrap()
                                    ));
                                    break;
                                }
                            }
                        }
                    } else {
                        err_msg.push_str(&format!(
                            "Original: {}\n",
                            serde_json::to_string_pretty(v).unwrap()
                        ));
                        err_msg.push_str(&format!(
                            "Serialized: {}\n",
                            serde_json::to_string_pretty(sv).unwrap()
                        ));
                    }
                }
            }
        } else {
            err_msg.push_str("Top level is not an object!\n");
        }
        panic!("{}", err_msg);
    }
}

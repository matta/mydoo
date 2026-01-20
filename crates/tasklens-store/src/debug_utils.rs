use automerge::{ReadDoc, Value};
use serde_json::{Value as JsonValue, json};

pub fn inspect_automerge_doc_full<D: ReadDoc>(doc: &D) -> JsonValue {
    inspect_obj_id(doc, &automerge::ROOT)
}

fn inspect_obj_id<D: ReadDoc>(doc: &D, obj_id: &automerge::ObjId) -> JsonValue {
    let mut map = serde_json::Map::new();

    // Use keys() iterator which is stable and reliable
    for key in doc.keys(obj_id) {
        // get returns Option<(Value, ObjId)>
        if let Ok(Some((value, child_id))) = doc.get(obj_id, &key) {
            let inspected = match value {
                Value::Object(_) => {
                    // Recurse using the child_id returned by get()
                    let content = inspect_obj_id(doc, &child_id);
                    json!({
                        "type": "Object", // We could be more specific but Object covers Map/List/Text
                        "objId": child_id.to_string(),
                        "content": content
                    })
                }
                Value::Scalar(s) => match s.as_ref() {
                    automerge::ScalarValue::Bytes(b) => {
                        json!({"type": "Bytes", "value": format!("{:?}", b)})
                    }
                    automerge::ScalarValue::Str(s) => json!({"type": "Str", "value": s}),
                    automerge::ScalarValue::Int(i) => json!({"type": "Int", "value": i}),
                    automerge::ScalarValue::Uint(u) => json!({"type": "Uint", "value": u}),
                    automerge::ScalarValue::F64(f) => json!({"type": "F64", "value": f}),
                    automerge::ScalarValue::Counter(c) => {
                        json!({"type": "Counter", "value": format!("{:?}", c)})
                    }
                    automerge::ScalarValue::Timestamp(t) => {
                        json!({"type": "Timestamp", "value": t})
                    }
                    automerge::ScalarValue::Boolean(b) => json!({"type": "Boolean", "value": b}),
                    automerge::ScalarValue::Null => json!({"type": "Null", "value": null}),
                    automerge::ScalarValue::Unknown { type_code, .. } => {
                        json!({"type": "Unknown", "code": type_code})
                    }
                },
            };
            map.insert(key, inspected);
        }
    }

    JsonValue::Object(map)
}

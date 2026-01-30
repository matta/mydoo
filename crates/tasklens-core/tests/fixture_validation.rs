use assert_json_diff::assert_json_eq;
use automerge::AutoCommit;
use std::fs;
use tasklens_core::domain::doc_bridge;
use tasklens_core::types::TunnelState;

/// Normalize JSON values by converting all numbers to f64.
/// This allows comparing JSON with integer values (e.g., 1) to JSON with float values (e.g., 1.0).
fn normalize_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Number(n) => {
            // Convert all numbers to f64 for consistent comparison
            if let Some(f) = n.as_f64() {
                serde_json::json!(f)
            } else {
                serde_json::Value::Number(n)
            }
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, normalize_json(v)))
                .collect(),
        ),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(normalize_json).collect())
        }
        other => other,
    }
}

#[test]
fn test_medieval_fixture_json_roundtrip() {
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/medieval_tasks.json"
    );
    let json_str = fs::read_to_string(fixture_path).expect("Failed to read fixture");

    // 1. Serde Roundtrip
    let state: TunnelState =
        serde_json::from_str(&json_str).expect("Failed to parse JSON into TunnelState");
    let roundtrip_json =
        serde_json::to_value(&state).expect("Failed to serialize TunnelState back to JSON");
    let original_json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse original JSON");

    // Normalize both values to handle integer vs float differences
    let normalized_original = normalize_json(original_json);
    let normalized_roundtrip = normalize_json(roundtrip_json);

    // Compare values
    assert_json_eq!(normalized_original, normalized_roundtrip);
}

#[test]
fn test_medieval_fixture_autosurgeon_roundtrip() {
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/medieval_tasks.json"
    );
    let json_str = fs::read_to_string(fixture_path).expect("Failed to read fixture");

    // Parse into TunnelState first to get a Reconcile-able type
    let state: TunnelState =
        serde_json::from_str(&json_str).expect("Failed to parse JSON into TunnelState");

    // 2. Automerge Roundtrip
    let mut doc = AutoCommit::new();
    doc_bridge::reconcile_tunnel_state(&mut doc, &state)
        .expect("Failed to reconcile TunnelState into Automerge");

    let hydrated: TunnelState = doc_bridge::hydrate_tunnel_state(&doc)
        .expect("Failed to hydrate TunnelState from Automerge");

    assert_eq!(state, hydrated, "Autosurgeon hydration roundtrip failed");
}

#[test]
fn test_medieval_fixture_cross_format_parity() {
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/medieval_tasks.json"
    );
    let json_str = fs::read_to_string(fixture_path).expect("Failed to read fixture");

    let original_json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse original JSON");
    let state: TunnelState =
        serde_json::from_str(&json_str).expect("Failed to parse JSON into TunnelState");

    // 3. Cross-Format Parity: Original JSON vs Hydrated JSON
    let mut doc = AutoCommit::new();
    doc_bridge::reconcile_tunnel_state(&mut doc, &state)
        .expect("Failed to reconcile state into Automerge");

    let hydrated_state: TunnelState =
        doc_bridge::hydrate_tunnel_state(&doc).expect("Failed to hydrate state from Automerge");
    let json_from_hydrated =
        serde_json::to_value(&hydrated_state).expect("Failed to serialize hydrated state");

    // Normalize both values to handle integer vs float differences
    let normalized_original = normalize_json(original_json);
    let normalized_hydrated = normalize_json(json_from_hydrated);

    assert_json_eq!(normalized_original, normalized_hydrated);
}

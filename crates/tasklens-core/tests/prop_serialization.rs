use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile};
use proptest::prelude::*;
use tasklens_core::types::TunnelState;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn test_serialization_parity(state: TunnelState) {
        // 1. Serde Roundtrip
        let json = serde_json::to_value(&state).expect("serde serialization failed");
        let deserialized: TunnelState = serde_json::from_value(json.clone()).expect("serde deserialization failed");
        prop_assert_eq!(&state, &deserialized, "Serde roundtrip failed");

        // 2. Autosurgeon Roundtrip
        let mut doc = AutoCommit::new();
        reconcile(&mut doc, &state).expect("reconcile failed");
        let hydrated: TunnelState = hydrate(&doc).expect("hydrate failed");
        prop_assert_eq!(&state, &hydrated, "Autosurgeon roundtrip failed");

        // 3. Cross-Format Parity: Original JSON vs Hydrated JSON
        // This ensures that the state we get back from Automerge produces the EXACT same JSON structure
        // as the original state, confirming value preservation across the board.
        let json_from_hydrated = serde_json::to_value(&hydrated).expect("serde serialization of hydrated state failed");
        prop_assert_eq!(json, json_from_hydrated, "JSON parity failed: Original vs Hydrated");
    }
}

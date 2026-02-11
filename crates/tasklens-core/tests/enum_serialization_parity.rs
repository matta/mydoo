//! Integration tests ensuring enum serialization parity between serde (JSON) and autosurgeon
//! (Automerge).
//!
//! The autosurgeon Reconcile format is canonical. Serde must produce identical string
//! representations so that `rust → JSON → automerge` and `rust → automerge` yield the same
//! document, and round-tripping through either path is lossless.

use automerge::{AutoCommit, ReadDoc};
use autosurgeon::{Hydrate, Reconcile};
use tasklens_core::types::{PriorityMode, ScheduleSource};

#[derive(Debug, PartialEq, Hydrate, Reconcile)]
struct ScheduleSourceWrapper {
    value: ScheduleSource,
}

#[derive(Debug, PartialEq, Hydrate, Reconcile)]
struct PriorityModeWrapper {
    value: PriorityMode,
}

/// Reconcile a ScheduleSource into an Automerge doc and read back the raw string.
fn reconciled_schedule_source(value: ScheduleSource) -> String {
    let wrapper = ScheduleSourceWrapper { value };
    let mut doc = AutoCommit::new();
    autosurgeon::reconcile(&mut doc, &wrapper).expect("reconcile failed");
    let (val, _) = doc
        .get(automerge::ROOT, "value")
        .expect("get failed")
        .expect("missing value key");
    val.into_scalar()
        .expect("expected scalar")
        .into_string()
        .expect("expected string")
}

/// Reconcile a PriorityMode into an Automerge doc and read back the raw string.
fn reconciled_priority_mode(value: PriorityMode) -> String {
    let wrapper = PriorityModeWrapper { value };
    let mut doc = AutoCommit::new();
    autosurgeon::reconcile(&mut doc, &wrapper).expect("reconcile failed");
    let (val, _) = doc
        .get(automerge::ROOT, "value")
        .expect("get failed")
        .expect("missing value key");
    val.into_scalar()
        .expect("expected scalar")
        .into_string()
        .expect("expected string")
}

/// Round-trip ScheduleSource through autosurgeon.
fn roundtrip_schedule_source(value: ScheduleSource) -> ScheduleSource {
    let wrapper = ScheduleSourceWrapper { value };
    let mut doc = AutoCommit::new();
    autosurgeon::reconcile(&mut doc, &wrapper).expect("reconcile failed");
    let hydrated: ScheduleSourceWrapper = autosurgeon::hydrate(&doc).expect("hydrate failed");
    hydrated.value
}

/// Round-trip PriorityMode through autosurgeon.
fn roundtrip_priority_mode(value: PriorityMode) -> PriorityMode {
    let wrapper = PriorityModeWrapper { value };
    let mut doc = AutoCommit::new();
    autosurgeon::reconcile(&mut doc, &wrapper).expect("reconcile failed");
    let hydrated: PriorityModeWrapper = autosurgeon::hydrate(&doc).expect("hydrate failed");
    hydrated.value
}

// --- ScheduleSource ---

#[test]
fn schedule_source_serde_matches_reconcile() {
    for value in [ScheduleSource::Myself, ScheduleSource::Ancestor] {
        let json_str = serde_json::to_value(value)
            .expect("serde serialization failed")
            .as_str()
            .expect("expected JSON string")
            .to_owned();
        let am_str = reconciled_schedule_source(value);
        assert_eq!(
            json_str, am_str,
            "for {value:?}: serde produced {json_str:?} but autosurgeon produced {am_str:?}"
        );
    }
}

#[test]
fn schedule_source_canonical_strings() {
    assert_eq!(reconciled_schedule_source(ScheduleSource::Myself), "Myself");
    assert_eq!(
        reconciled_schedule_source(ScheduleSource::Ancestor),
        "Ancestor"
    );
}

#[test]
fn schedule_source_serde_roundtrip() {
    for value in [ScheduleSource::Myself, ScheduleSource::Ancestor] {
        let json = serde_json::to_value(value).expect("serialize failed");
        let back: ScheduleSource = serde_json::from_value(json).expect("deserialize failed");
        assert_eq!(value, back, "serde roundtrip failed for {value:?}");
    }
}

#[test]
fn schedule_source_autosurgeon_roundtrip() {
    for value in [ScheduleSource::Myself, ScheduleSource::Ancestor] {
        let back = roundtrip_schedule_source(value);
        assert_eq!(value, back, "autosurgeon roundtrip failed for {value:?}");
    }
}

#[test]
fn schedule_source_cross_format_parity() {
    for value in [ScheduleSource::Myself, ScheduleSource::Ancestor] {
        let json_original = serde_json::to_value(value).expect("serialize original failed");
        let hydrated = roundtrip_schedule_source(value);
        let json_hydrated =
            serde_json::to_value(hydrated).expect("serialize hydrated value failed");
        assert_eq!(
            json_original, json_hydrated,
            "cross-format parity failed for {value:?}"
        );
    }
}

// --- PriorityMode ---

#[test]
fn priority_mode_serde_matches_reconcile() {
    for value in [PriorityMode::DoList, PriorityMode::PlanOutline] {
        let json_str = serde_json::to_value(value)
            .expect("serde serialization failed")
            .as_str()
            .expect("expected JSON string")
            .to_owned();
        let am_str = reconciled_priority_mode(value);
        assert_eq!(
            json_str, am_str,
            "for {value:?}: serde produced {json_str:?} but autosurgeon produced {am_str:?}"
        );
    }
}

#[test]
fn priority_mode_canonical_strings() {
    assert_eq!(reconciled_priority_mode(PriorityMode::DoList), "DoList");
    assert_eq!(
        reconciled_priority_mode(PriorityMode::PlanOutline),
        "PlanOutline"
    );
}

#[test]
fn priority_mode_serde_roundtrip() {
    for value in [PriorityMode::DoList, PriorityMode::PlanOutline] {
        let json = serde_json::to_value(value).expect("serialize failed");
        let back: PriorityMode = serde_json::from_value(json).expect("deserialize failed");
        assert_eq!(value, back, "serde roundtrip failed for {value:?}");
    }
}

#[test]
fn priority_mode_autosurgeon_roundtrip() {
    for value in [PriorityMode::DoList, PriorityMode::PlanOutline] {
        let back = roundtrip_priority_mode(value);
        assert_eq!(value, back, "autosurgeon roundtrip failed for {value:?}");
    }
}

#[test]
fn priority_mode_cross_format_parity() {
    for value in [PriorityMode::DoList, PriorityMode::PlanOutline] {
        let json_original = serde_json::to_value(value).expect("serialize original failed");
        let hydrated = roundtrip_priority_mode(value);
        let json_hydrated =
            serde_json::to_value(hydrated).expect("serialize hydrated value failed");
        assert_eq!(
            json_original, json_hydrated,
            "cross-format parity failed for {value:?}"
        );
    }
}

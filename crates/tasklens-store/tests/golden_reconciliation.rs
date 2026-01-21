use std::path::PathBuf;
use tasklens_core::types::TunnelState;

#[test]
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
}

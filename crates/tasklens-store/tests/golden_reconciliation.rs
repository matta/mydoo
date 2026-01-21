use std::path::PathBuf;

#[test]
fn test_golden_reconciliation() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/data/golden.automerge");

    println!("Loading golden file from: {:?}", d);

    let bytes = std::fs::read(&d).expect("failed to read golden file");
    let doc = automerge::AutoCommit::load(&bytes).expect("Failed to load automerge doc");

    println!(
        "Successfully loaded automerge doc with actor: {:?}",
        doc.get_actor()
    );
}

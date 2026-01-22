use automerge::AutoCommit;
use autosurgeon::reconcile;
use proptest::prelude::*;
use std::fs;
use std::process::Command;
use tasklens_core::types::TunnelState;

#[test]
fn test_cross_compat_fuzz() {
    // We run the fuzzer inside a normal test.
    // Proptest will run multiple cases.
    let config = ProptestConfig::with_cases(100); // 100 cases is a good balance for CI
    let mut runner = proptest::test_runner::TestRunner::new(config);

    // Create a temporary directory for the fuzzy documents
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    println!("Fuzzing documents into: {:?}", temp_path);

    runner
        .run(&any::<TunnelState>(), |state| {
            let mut doc = AutoCommit::new();
            reconcile(&mut doc, &state).expect("Failed to reconcile state");

            let bytes = doc.save();
            let file_name = format!("state_{}.automerge", uuid::Uuid::new_v4());
            let file_path = temp_path.join(file_name);

            fs::write(&file_path, bytes).expect("Failed to write fuzzy document");
            Ok(())
        })
        .expect("Proptest failed");

    // All documents generated, now call the TS validator
    println!("Invoking TS validator...");

    // We assume we are running from the workspace root or crate root.
    // Base command: pnpm run validate-compliance
    // We need to find the root of the workspace.
    let mut manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !manifest_dir.join("package.json").exists() {
        if !manifest_dir.pop() {
            panic!("Could not find workspace root (no package.json found)");
        }
    }

    let validator_output = Command::new("pnpm")
        .arg("run")
        .arg("--filter")
        .arg("@mydoo/tasklens")
        .arg("validate-compliance")
        .arg(temp_path)
        .current_dir(&manifest_dir)
        .output()
        .expect("Failed to execute TS validator command");

    if !validator_output.status.success() {
        eprintln!("TS validator failed compatibility check!");
        eprintln!(
            "STDOUT:\n{}",
            String::from_utf8_lossy(&validator_output.stdout)
        );
        eprintln!(
            "STDERR:\n{}",
            String::from_utf8_lossy(&validator_output.stderr)
        );
        panic!("TS validator failed compatibility check");
    }
}

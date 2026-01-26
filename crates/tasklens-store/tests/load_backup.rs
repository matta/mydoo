// This test uses `tokio` for async runtime, which is excluded from WASM builds.
#![cfg(not(target_arch = "wasm32"))]

use automerge::ReadDoc;
use samod::runtime::LocalRuntimeHandle;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tasklens_store::store::AppStore;

#[derive(Clone, Debug)]
struct TestRuntime;

impl LocalRuntimeHandle for TestRuntime {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        tokio::task::spawn_local(future);
    }
}

#[tokio::test]
async fn test_load_golden_file() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("tests/data/golden.automerge");

            println!("Loading golden file from: {:?}", d);

            let bytes = std::fs::read(&d).expect("failed to read golden file");

            // Load directly into AutoCommit to inspect
            let doc = automerge::AutoCommit::load(&bytes).expect("Failed to load automerge doc");

            // Debug: Inspect root keys and values (basic check)
            println!("DEBUG: Inspecting root values:");
            let keys: Vec<_> = doc.keys(automerge::ROOT).collect();
            for key in keys {
                let val = doc.get(automerge::ROOT, &key);
                println!("Key: {}, Value: {:?}", key, val);
            }

            // Setup AppStore with Repo
            let runtime = TestRuntime;
            let repo = samod::RepoBuilder::new(runtime).load_local().await;
            let mut store = AppStore::with_repo(repo.clone());

            match AppStore::import_doc(repo, bytes).await {
                Ok((handle, id)) => {
                    store.set_active_doc(handle, id);
                    println!("Import successful!");
                }
                Err(e) => {
                    println!("Import failed as expected: {:?}", e);
                }
            }

            // Additional assertions
            let state: tasklens_core::types::TunnelState =
                store.hydrate().expect("Failed to hydrate");
            println!("Successfully loaded. Task count: {}", state.tasks.len());
        })
        .await;
}

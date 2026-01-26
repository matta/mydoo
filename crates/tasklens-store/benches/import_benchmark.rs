//! Run with:
//! cargo bench -p tasklens-store --bench import_benchmark

// This benchmark is excluded from WASM builds because it depends on `criterion` and `tokio`.
#[cfg(not(target_arch = "wasm32"))]
mod bench {
    use criterion::{Criterion, criterion_group, criterion_main};
    use samod::runtime::LocalRuntimeHandle;
    use std::fs;
    use std::future::Future;
    use std::path::PathBuf;
    use std::pin::Pin;
    use tasklens_store::store::AppStore;

    #[derive(Clone, Debug)]
    struct BenchRuntime;

    impl LocalRuntimeHandle for BenchRuntime {
        fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
            tokio::task::spawn_local(future);
        }
    }

    fn benchmark_import_doc(c: &mut Criterion) {
        // Locate the golden.automerge file relative to the package root
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data/golden.automerge");

        let bytes = fs::read(&path).expect("Failed to read golden.automerge");

        let mut group = c.benchmark_group("import_group");

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        group.bench_function("import_doc", |b| {
            b.to_async(&rt).iter(|| {
                let bytes = bytes.clone();
                async move {
                    let local = tokio::task::LocalSet::new();
                    local
                        .run_until(async move {
                            // Create Repo INSIDE LocalSet so background tasks are correctly polled
                            let repo = samod::RepoBuilder::new(BenchRuntime).load_local().await;
                            let mut store = AppStore::with_repo(repo.clone());
                            let (handle, id) =
                                AppStore::import_doc(repo, std::hint::black_box(bytes))
                                    .await
                                    .unwrap();
                            store.set_active_doc(handle, id);
                        })
                        .await;
                }
            })
        });
        group.finish();
    }

    fn config() -> Criterion {
        Criterion::default().measurement_time(std::time::Duration::from_secs(10))
    }

    criterion_group! {
        name = benches;
        config = config();
        targets = benchmark_import_doc
    }
    criterion_main!(benches);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

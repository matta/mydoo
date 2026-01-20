//! Run with:
//! cargo bench -p tasklens-store --bench import_benchmark

use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use tasklens_store::store::AppStore;

fn benchmark_import_doc(c: &mut Criterion) {
    // Locate the golden.automerge file relative to the package root
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data/golden.automerge");

    let bytes = fs::read(&path).expect("Failed to read golden.automerge");

    let mut group = c.benchmark_group("import_group");

    group.bench_function("import_doc", |b| {
        b.iter(|| {
            let mut store = AppStore::new();
            store.import_doc(black_box(bytes.clone())).unwrap();
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

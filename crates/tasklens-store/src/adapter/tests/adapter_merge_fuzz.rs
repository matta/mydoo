//! Property-based merge tests for TaskLens.
//!
//! These tests use `proptest` to generate random sequences of concurrent
//! actions, merge the resulting documents, and verify that system
//! invariants (like hydratability and parent/child consistency) are
//! preserved.
//!
//! ### How to Run
//!
//! Run all merge tests in this module:
//! ```bash
//! cargo test -p tasklens-store adapter::tests::adapter_merge_fuzz
//! ```
//!
//! To run only the fuzz test with custom case count:
//! ```bash
//! PROPTEST_CASES=500 cargo test -p tasklens-store adapter::tests::adapter_merge_fuzz::test_merge_invariants_fuzz
//! ```

use crate::adapter::{
    self,
    tests::adapter_test_common::{any_action, check_invariants, init_doc},
};
use proptest::prelude::*;

proptest! {
    #[test]
    #[ignore]  // FIXME: enable this test
    fn test_merge_invariants_fuzz(
        setup in prop::collection::vec(any_action(), 1..10),
        concurrent_a in prop::collection::vec(any_action(), 1..10),
        concurrent_b in prop::collection::vec(any_action(), 1..10),
    ) {
        let mut doc_a = init_doc().expect("Init failed");

        // 1. Initial State
        for action in setup {
            let _ = adapter::dispatch(&mut doc_a, action);
        }

        // 2. Fork
        let mut doc_b = doc_a.fork();

        // 3. Mutate concurrently
        for action in concurrent_a {
            let _ = adapter::dispatch(&mut doc_a, action);
        }
        for action in concurrent_b {
            let _ = adapter::dispatch(&mut doc_b, action);
        }

        // 4. Merge
        doc_a.merge(&mut doc_b).expect("Merge failed");

        // 5. Assert: Invariants held
        if let Err(msg) = check_invariants(&doc_a) {
            panic!("Invariant Failure!\n{}", msg);
        }
    }
}

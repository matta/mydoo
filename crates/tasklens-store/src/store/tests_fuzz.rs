//! Property-based sequential fuzz tests for TaskLens.
//!
//! These tests verify that simple sequences of actions (Create, Update, Delete, Move)
//! on a single document preserve system invariants without any concurrent merges.
//!
//! ### How to Run
//!
//! ```bash
//! cargo test -p tasklens-store store::tests_fuzz
//! ```
//!
//! To run with custom case count:
//! ```bash
//! PROPTEST_CASES=500 cargo test -p tasklens-store store::tests_fuzz::test_sequential_invariants_fuzz
//! ```

use crate::adapter;
use crate::store::test_common::*;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_sequential_invariants_fuzz(
        actions in prop::collection::vec(any_action(), 1..100),
    ) {
        let mut doc = init_doc().expect("Init failed");

        for action in actions {
            let _ = adapter::dispatch(&mut doc, action);

            // Check invariants after EVERY step for sequential fuzzing
            if let Err(msg) = check_invariants(&doc) {
                panic!("Invariant Failure at step!\n{}", msg);
            }
        }
    }
}

//! Property-based sequential fuzz tests for TaskLens.
//!
//! These tests verify that simple sequences of actions (Create, Update, Delete, Move)
//! on a single document preserve system invariants without any concurrent merges.
//!
//! ### How to Run
//!
//! ```bash
//! cargo test -p tasklens-store adapter::tests::adapter_fuzz
//! ```
//!
//! To run with custom case count:
//! ```bash
//! PROPTEST_CASES=500 cargo test -p tasklens-store adapter::tests::adapter_fuzz::test_sequential_invariants_fuzz
//! ```

use crate::adapter::tests::adapter_test_common::{
    any_action, check_invariants, dispatch_and_validate, init_doc,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sequential_invariants_fuzz(
        actions in prop::collection::vec(any_action(), 1..10),
    ) {
        let mut doc = init_doc().expect("Init failed");

        for action in actions {
            dispatch_and_validate(&mut doc, action, "sequential fuzz");

            // Check invariants after EVERY step for sequential fuzzing
            if let Err(msg) = check_invariants(&doc) {
                panic!("Invariant Failure at step!\n{}", msg);
            }
        }
    }
}

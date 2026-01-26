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
    AbstractAction, FuzzState, HydrationStrategy, any_abstract_action, check_invariants,
    dispatch_and_validate, init_doc, interpret_actions,
};
use proptest::prelude::*;
use proptest::strategy::{Strategy, ValueTree};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn test_sequential_invariants_fuzz(
        abstract_actions in prop::collection::vec(any_abstract_action(0.9), 1..15),
    ) {
        let (actions, _final_state) = interpret_actions(FuzzState::default(), abstract_actions, "s-");
        let mut doc = init_doc().expect("Init failed");

        for action in actions {
            dispatch_and_validate(&mut doc, action, "sequential fuzz");

            // Check invariants after EVERY step for sequential fuzzing
            if let Err(msg) = check_invariants(&doc, HydrationStrategy::Strict) {
                panic!("Invariant Failure at step!\n{}", msg);
            }
        }
    }
}

#[test]
fn test_strategy_distribution_regression() {
    let mut runner = proptest::test_runner::TestRunner::default();
    let strategy = any_abstract_action(0.9);
    let mut valid_count = 0;
    let samples = 10_000;

    for _ in 0..samples {
        let action = strategy
            .new_tree(&mut runner)
            .expect("Strategy failure")
            .current();

        if let AbstractAction::Chaos(_) = action {
            // No-op
        } else {
            valid_count += 1;
        }
    }

    let ratio = valid_count as f64 / samples as f64;
    println!("Observed valid ratio: {}", ratio);

    // Expected 0.9. With n=1000, 5-sigma is approx +/- 0.05.
    // Range 0.85 to 0.95.
    assert!(ratio > 0.85, "Ratio {} too low (expected ~0.9)", ratio);
    assert!(ratio < 0.95, "Ratio {} too high (expected ~0.9)", ratio);
}

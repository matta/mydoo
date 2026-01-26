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

use crate::adapter::tests::adapter_test_common::{
    FuzzState, HydrationStrategy, any_abstract_action, check_invariants, dispatch_and_validate,
    init_doc, interpret_actions,
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn test_merge_invariants_fuzz(
        setup_abstracts in prop::collection::vec(any_abstract_action(0.9), 1..10),
        concurrent_a_abstracts in prop::collection::vec(any_abstract_action(0.9), 1..10),
        concurrent_b_abstracts in prop::collection::vec(any_abstract_action(0.9), 1..10),
    ) {
        // 1. Setup
        let (setup_actions, setup_state) = interpret_actions(FuzzState::default(), setup_abstracts, "s-");

        let mut doc_a = init_doc().expect("Init failed");
        for action in setup_actions {
            dispatch_and_validate(&mut doc_a, action, "merge fuzz setup");
        }

        // 2. Fork
        let mut doc_b = doc_a.fork();

        // 3. Interpret concurrent streams using the setup state as base
        // Note: We clone setup_state for each branch independently so they diverge from the same snapshot.
        let (conc_a, _) = interpret_actions(setup_state.clone(), concurrent_a_abstracts, "a-");
        let (conc_b, _) = interpret_actions(setup_state, concurrent_b_abstracts, "b-");

        // 4. Mutate concurrently
        for action in conc_a {
            dispatch_and_validate(&mut doc_a, action, "merge fuzz concurrent_a");
        }
        for action in conc_b {
            dispatch_and_validate(&mut doc_b, action, "merge fuzz concurrent_b");
        }

        // 5. Merge
        doc_a.merge(&mut doc_b).expect("Merge failed");

        // 6. Assert: Invariants held
        if let Err(msg) = check_invariants(&doc_a, HydrationStrategy::Heal) {
            panic!("Invariant Failure!\n{}", msg);
        }
    }
}

# Implementation Plan - Golden File Reconciliation Test

## Phase 1: Test Scaffold & Data Loading [checkpoint: cc2081c]
- [x] Task: Create Test File [acfec3e]
    - [x] Create `crates/tasklens-store/tests/golden_reconciliation.rs`
    - [x] Add basic test entry point `test_golden_reconciliation`
- [x] Task: Implement Data Loading [4d422b5]
    - [x] Read `crates/tasklens-store/tests/data/golden.automerge`
    - [x] Load into `automerge::AutoCommit` (Source Doc)
    - [x] Assert load success
- [x] Task: Conductor - User Manual Verification 'Test Scaffold & Data Loading' (Protocol in workflow.md)

## Phase 2: Hydration & Reconciliation Logic [checkpoint: 0a0d8e6]
- [x] Task: Implement Hydration [b766027]
    - [x] Use `autosurgeon::hydrate` to load `TunnelState` from Source Doc
    - [x] Verify hydration success (basic check, e.g., task count > 0)
- [x] Task: Implement Reconciliation [144f296]
    - [x] Fork Source Doc to create Target Doc (`doc.fork()` or `clone`)
    - [x] Run `autosurgeon::reconcile` on Target Doc with hydrated state
- [x] Task: Conductor - User Manual Verification 'Hydration & Reconciliation Logic' (Protocol in workflow.md)

## Phase 3: Recursive Diff Implementation
- [x] Task: Implement Diff Logic [f2a091d]
    - [x] Implement a recursive comparison function `assert_docs_equal(doc_a, doc_b)`
    - [x] Walk the document tree (maps, lists, values)
    - [x] Collect differences into a structured log/error message
- [x] Task: Connect to Test [f2a091d]
    - [x] Call comparison function in main test body
    - [x] Fail test with panic/assert if differences found
- [~] Task: Conductor - User Manual Verification 'Recursive Diff Implementation' (Protocol in workflow.md)

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

## Phase 2: Hydration & Reconciliation Logic
- [x] Task: Implement Hydration [b766027]
    - [x] Use `autosurgeon::hydrate` to load `TunnelState` from Source Doc
    - [x] Verify hydration success (basic check, e.g., task count > 0)
- [x] Task: Implement Reconciliation [144f296]
    - [x] Fork Source Doc to create Target Doc (`doc.fork()` or `clone`)
    - [x] Run `autosurgeon::reconcile` on Target Doc with hydrated state
- [~] Task: Conductor - User Manual Verification 'Hydration & Reconciliation Logic' (Protocol in workflow.md)

## Phase 3: Recursive Diff Implementation
- [ ] Task: Implement Diff Logic
    - [ ] Implement a recursive comparison function `assert_docs_equal(doc_a, doc_b)`
    - [ ] Walk the document tree (maps, lists, values)
    - [ ] Collect differences into a structured log/error message
- [ ] Task: Connect to Test
    - [ ] Call comparison function in main test body
    - [ ] Fail test with panic/assert if differences found
- [ ] Task: Conductor - User Manual Verification 'Recursive Diff Implementation' (Protocol in workflow.md)

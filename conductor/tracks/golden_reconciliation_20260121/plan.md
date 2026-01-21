# Implementation Plan - Golden File Reconciliation Test

## Phase 1: Test Scaffold & Data Loading
- [x] Task: Create Test File [acfec3e]
    - [x] Create `crates/tasklens-store/tests/golden_reconciliation.rs`
    - [x] Add basic test entry point `test_golden_reconciliation`
- [ ] Task: Implement Data Loading
    - [ ] Read `crates/tasklens-store/tests/data/golden.automerge`
    - [ ] Load into `automerge::AutoCommit` (Source Doc)
    - [ ] Assert load success
- [ ] Task: Conductor - User Manual Verification 'Test Scaffold & Data Loading' (Protocol in workflow.md)

## Phase 2: Hydration & Reconciliation Logic
- [ ] Task: Implement Hydration
    - [ ] Use `autosurgeon::hydrate` to load `TunnelState` from Source Doc
    - [ ] Verify hydration success (basic check, e.g., task count > 0)
- [ ] Task: Implement Reconciliation
    - [ ] Fork Source Doc to create Target Doc (`doc.fork()` or `clone`)
    - [ ] Run `autosurgeon::reconcile` on Target Doc with hydrated state
- [ ] Task: Conductor - User Manual Verification 'Hydration & Reconciliation Logic' (Protocol in workflow.md)

## Phase 3: Recursive Diff Implementation
- [ ] Task: Implement Diff Logic
    - [ ] Implement a recursive comparison function `assert_docs_equal(doc_a, doc_b)`
    - [ ] Walk the document tree (maps, lists, values)
    - [ ] Collect differences into a structured log/error message
- [ ] Task: Connect to Test
    - [ ] Call comparison function in main test body
    - [ ] Fail test with panic/assert if differences found
- [ ] Task: Conductor - User Manual Verification 'Recursive Diff Implementation' (Protocol in workflow.md)

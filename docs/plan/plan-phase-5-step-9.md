# Step 9: Deferred E2E Verification

_Catch up on E2E tests that were deferred from Phase 4._

**PRD Reference**: [§4.3 The Plan View](../design/prd.md)

## Tasks

### 1. E2E: Edit Task Properties

- [x] **Goal**: Verify that task properties can be edited and persisted via the Plan View and Task Editor.
- [x] **Scenario**:
  - Click task title in Plan View to open Task Editor modal.
  - Modify Title, Importance, Effort.
  - Save changes.
  - Verify changes persisted (reload page, check values).

### 2. E2E: Delete Task with Cascade

- [x] **Goal**: Verify that deleting a parent task removes it and all its descendants.
- [x] **Scenario**:
  - Create or identify a task with children (Hierarchy).
  - Open Task Editor for parent task.
  - Click "Delete".
  - Verify confirmation dialog shows descendant count.
  - Confirm deletion.
  - Verify parent AND all children are removed from the tree.

### 3. E2E: Page Reload Persistence

- [x] **Goal**: Verify that data persists across page reloads (simulating app restart).
- [x] **Scenario**:
  - Fix seed parameter issue preventing this test (if any).
  - Create a task.
  - Reload the page.
  - Verify task is still present.

## Verification

### Automated

- [x] `plan.spec.ts`: Add `test("Edit Task Properties")`
- [x] `plan.spec.ts`: Add `test("Delete Task with Cascade")`
- [x] `plan.spec.ts`: Add `test("Page Reload Persistence")`

## Quality Gates

- [x] `pnpm fix && pnpm pre-commit` -> Pass
- [x] **EVIDENCE**: Show terminal output of passing tests.

## Completion

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix && pnpm pre-commit`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

# Implementation Plan: Phase 4 (Task Details & Editing)

**Goal**: Full property editing via modal. Add Sibling/Child creation. Delete with confirmation.

**PRD References**: [§4.5 Task Editing](../design/prd.md), [§3.6 Deletion Logic](../design/prd.md), [§4.2 Mobile FAB](../design/prd.md)

---

## Step 1: `useTaskDetails` Projection Hook

_Fetch task details for the modal._

- [x] Create `apps/client/src/viewmodel/projections/useTaskDetails.ts`:
  - Returns task object, parent title, descendant count, isLoading
- [x] Write unit tests

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (Client tests: 26 passed)
- [x] `pnpm lint` -> Pass
- [x] ✅ Update this doc
- [x] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 **STOP FOR HUMAN APPROVAL** before commit
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response is NOT sufficient.

**Evidence:**

```sh
# pnpm lint (Passed)
Checked 83 files in 16ms. No fixes applied.

# pnpm build (Passed)
client:build: ✓ built in 3.70s
Tasks: 2 successful, 2 total

# pnpm test --filter client (26 passing)
✓ src/viewmodel/projections/useTaskDetails.test.tsx (4 tests)
✓ src/viewmodel/projections/UsePriorityList.test.tsx (4 tests)
✓ src/viewmodel/intents/UseSystemIntents.test.tsx (1 test)
✓ src/components/primitives/QuickAddInput.test.tsx (6 tests)
✓ src/components/primitives/TaskRow.test.tsx (5 tests)
✓ src/viewmodel/intents/UseTaskIntents.test.tsx (4 tests)
✓ src/viewmodel/UseDocument.test.tsx (2 tests)
✓ src/tests/setup.test.ts (1 test)

# pnpm test:e2e (6 passed, 1 skipped)
6 passed (8.1s)
1 skipped (persistence test deferred to Phase 5)
```

---

## Step 2: `deleteTask` with Cascade

_Implement cascade deletion per PRD §3.6._

- [x] Extend `useTaskIntents` with `deleteTask(id)`
- [x] Implement cascade in `ops.deleteTask`:
  - Set `status = 'Deleted'` on target task
  - Recursively set `status = 'Deleted'` on all descendants
  - Remove from parent's `childTaskIds` and/or `rootTaskIds`
- [x] Write unit tests for cascade behavior

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass
- [x] `pnpm test:e2e` -> 6 Passed (1 Skipped - Known Issue)
- [x] `pnpm lint` -> Pass
- [ ] ✅ Update this doc
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 **STOP FOR HUMAN APPROVAL** before commit
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response is NOT sufficient.

---

## Step 3: TaskEditorModal UI

_Build the modal per PRD §4.5._

- [ ] Create `TaskEditorModal` component:
  - Title input
  - Parent (read-only) + "Move..." button _(defer to Phase 5)_ + "Find in Plan" _(defer)_
  - Importance slider (0.0 - 1.0)
  - Effort picker (1 | 3 | 5) → maps to `creditIncrement`
  - Due Date picker, Lead Time input, Repeat selector
  - Place dropdown
  - Notes textarea
  - Footer: Add Sibling, Add Child, Delete
- [ ] Create `DeleteConfirmModal` (shows descendant count)
- [ ] Wire modal open on task tap
- [ ] Implement Mobile FAB for quick Inbox add (PRD §4.2)

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass
- [ ] `pnpm test:e2e` -> Pass
- [ ] ✅ Update this doc
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 **STOP FOR HUMAN APPROVAL** before commit
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response is NOT sufficient.

---

## Step 4: E2E Verification

- [ ] E2E: Open modal on task tap → verify fields populated
- [ ] E2E: Edit title/importance → save → verify persistence
- [ ] E2E: Delete task with children → confirm dialog shows count → verify cascade

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass
- [ ] `pnpm test:e2e` -> Pass
- [ ] ✅ Update this doc
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 **STOP FOR HUMAN APPROVAL** before commit
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response is NOT sufficient.

---

## Deferred to Phase 5

- "Move..." picker modal (reparenting)
- "Find in Plan" navigation

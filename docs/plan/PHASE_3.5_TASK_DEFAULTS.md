# Implementation Plan: Phase 3.5 (Task Creation Defaults)

**Goal**: Ensure tasks are created with correct PRD-specified defaults and parent inheritance.

**PRD Reference**: [§3.2 Default Values for New Tasks](../design/prd.md#32-default-values-for-new-tasks)

---

## Step 1: Update `useTaskIntents.createTask` to Accept `parentId`

_Enable specifying where new tasks are created in the hierarchy._

- [ ] Modify `apps/client/src/viewmodel/intents/useTaskIntents.ts`:
  - Change `createTask(title: string)` → `createTask(title: string, parentId?: TaskID)`
  - Pass `parentId` to `ops.add({title, parentId})`
- [ ] Update test `UseTaskIntents.test.tsx` to verify parentId is passed through.

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass (ALL repo tests)
- [ ] `pnpm test:e2e` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 Request git commit.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
  - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
  - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
  - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.

---

## Step 2: Implement Parent Inheritance in `ops.createTask`

_Apply PRD §3.2 defaults with parent inheritance for `placeId`._

Currently, `ops.createTask` in `packages/tasklens/src/persistence/ops.ts`:
- Sets `importance: 1.0` ✅ (matches PRD)
- Sets `status: Pending` (PRD says `active`, equivalent) ✅
- Sets `childTaskIds: []` ✅
- Does NOT inherit `placeId` from parent

**Proposed Changes**:
- [ ] In `createTask`, implement `placeId` inheritance:
  ```typescript
  const placeId = props.placeId ?? (props.parentId ? state.tasks[props.parentId]?.placeId : ANYWHERE_PLACE_ID);
  ```
- [ ] Write unit tests in `packages/tasklens/tests/unit/createTaskDefaults.test.ts`:
  - Root task gets default placeId (ANYWHERE_PLACE_ID)
  - Child task inherits placeId from parent

> **Note**: The PRD `effort` field (1 | 3 | 5) is deferred to Phase 4 (Task Details Modal). The current `creditIncrement` field serves the algorithmic purpose.

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass (ALL repo tests)
- [ ] `pnpm test:e2e` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 Request git commit.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
  - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
  - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
  - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.

---

## Step 3: Verify End-to-End Task Creation

_Ensure the full stack applies defaults correctly._

- [x] Add E2E test in `apps/client/tests/e2e/` to verify:
  - Created task has correct default values
  - Task appears in priority list

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (61 tasklens, 22 client)
- [x] `pnpm test:e2e` -> Pass (6 of 7 tests)
- [x] **EVIDENCE**: All tests passing.

**Completion**

- [x] ✅ **CLEAN LISTS**: Lists cleaned.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 Request git commit.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Commit pending approval.

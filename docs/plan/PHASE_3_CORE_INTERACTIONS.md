# Implementation Plan: Phase 3 (Core Interactions)

**Goal**: Make the list interactive by implementing completion toggling and task creation.

## Step 1: Implement `useTaskIntents` Hook

_Create the intent hooks that project user actions into Automerge operations._

- [x] Create `apps/client/src/viewmodel/intents/useTaskIntents.ts`.
- [x] Implement `toggleTaskCompletion(id)`: atomic update of status.
- [x] Implement `createTask(text)`: atomic append to task list.
- [x] Write unit test: `tests/useTaskIntents.test.tsx`.

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: [lint/build/test pass](file:///Users/matt/src/mydoo/docs/plan/PHASE_3_CORE_INTERACTIONS.md#L100)

**Completion**

- [x] ✅ **CLEAN LISTS**: Lists are clean.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 Request git commit.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Pending.

## Step 2: Wire UI Interactions

_Connect the `TaskRow` checkbox to the intents._

- [x] Update `apps/client/src/components/primitives/TaskRow.tsx` to accept `onToggle`.
- [x] Update `apps/client/src/components/composites/PriorityTaskList.tsx` to pass `onToggle`.
- [x] Update `apps/client/src/viewmodel/containers/DoViewContainer.tsx` to use `useTaskIntents`.
- [x] Verify checkbox click triggers state change.

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: [lint/build/test pass](file:///Users/matt/src/mydoo/docs/plan/PHASE_3_CORE_INTERACTIONS.md#L100)

**Completion**

- [x] ✅ **CLEAN LISTS**: Lists are clean.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 Request git commit.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Pending.

## Step 3: Implement Quick Add

_Add the ability to create new tasks from the view._

- [x] Create `apps/client/src/components/primitives/QuickAddInput.tsx`.
- [x] Add to `DoViewContainer`.
- [x] Wire to `createTask` intent.

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: [lint/build/test/e2e pass](file:///Users/matt/src/mydoo/docs/plan/PHASE_3_CORE_INTERACTIONS.md#L100)

**Completion**

- [x] ✅ **CLEAN LISTS**: Lists are clean.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 Request git commit.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Pending.

## Step 4: End-to-End Verification

_Verify the full interaction loop._

- [ ] Write E2E test: `tests/e2e/interactions.spec.ts`.
- [ ] Test: Load app -> Create Task -> Verify it appears.
- [ ] Test: Click Checkbox -> Verify it remains but shows as Done (since Do view shows pending).
- [ ] Test: Click "update" button -> Verify done tasks are gone.

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

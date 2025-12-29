# Phase 7: The Balance View

**Goal**: Implement the "Life Balance" visualization to allow users to adjust target effort (`desiredCredits`) for Top-Level Goals and see their actual effort distribution.

**Status**: [ ] Not Started

## 1. Prerequisites & Type Rationalization

The "Balance" algorithm relies on `effectiveCredits` (decayed effort) and `feedbackFactor`. These are currently calculated in the domain logic but strictly "internal" to `EnrichedTask` and not exposed on `ComputedTask`. To visualize "Actual %" vs "Target %", we must expose these computed metrics to the View Layer.

Additionally, `ROOT_INBOX_ID` is referenced in designs but missing from the implementation.

### Steps

- [x] **Step 1.1: Expose Computed Metrics**
  - [x] **[MODIFY]** `packages/tasklens/src/types.ts`:
    - [x] Add `export const ROOT_INBOX_ID = 'root:inbox' as TaskID;`
    - [x] Update `ComputedTask` to include readonly properties:
      - [x] `effectiveCredits: number` (for Actual % calculation)
      - [x] `desiredCredits: number` (already in PersistedTask, but ensure visibility)

**Quality Gates**

- [ ] `pnpm fix && pnpm pre-commit` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix && pnpm pre-commit`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
  - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
  - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
  - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.

## 2. Balance Logic (Client Hook)

We need a bespoke hook to calculate the "Pie Chart" percentages.

**Plain English Explanation**:
Currently, our logic lives in the database layer. The UI needs a way to "read" the current state of tasks from our local "file cabinet" (Redux Store) and calculate two numbers for every Top Level Goal:

1. **Target**: How much effort you _want_ to spend (User set).
2. **Actual**: How much effort you _actually_ spent (Calculated from history).

This hook acts as the translator, grabbing the raw task data and doing the math so the UI can just draw the bars.

### Steps

- [x] **Step 2.1: Implement `useTaskEntities` Hook**
  - [x] **[NEW]** `packages/tasklens/src/react/hooks/use-task-entities.ts`:
    - [x] Create simple wrapper around `useSelector(selectTaskEntities)`.
    - [x] Export from index.

**Quality Gates**

- [ ] `pnpm fix && pnpm pre-commit` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix && pnpm pre-commit`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
  - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
  - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
  - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.

- [x] **Step 2.2: Implement `useBalanceData` Hook**
  - [x] **[NEW]** `apps/client/src/hooks/use-balance-data.ts`:
    - [x] Import `useTaskEntities` (Redux access).
    - [x] Logic:
      - [x] Get all tasks where `parentId` is `undefined` (Roots).
      - [x] **Exclude** `ROOT_INBOX_ID` (we will ensure this ID is used for the Inbox).
      - [x] Calculate `totalDesiredCredits`.
      - [x] Calculate `totalEffectiveCredits`.
      - [x] Return normalized list.

**Quality Gates**

- [ ] `pnpm fix && pnpm pre-commit` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix && pnpm pre-commit`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
  - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
  - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
  - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.

## 3. Balance UI Components

### Steps

- [x] Step 3.1: Balance Item Component
  - [x] **[NEW]** `apps/client/src/components/views/balance/balance-item.tsx`:
    - [x] Props: `item`, `onChangeDesiredCredits(id, val)`.
    - [x] UI:
      - [x] Title.
      - [x] Slider for `desiredCredits` (e.g., 0-10, default 1?).
      - [x] Progress Bar for `actualPercent` (vs `targetPercent`).
      - [x] Visual "Starving" indicator if `actual << target`.

**Quality Gates**

- [x] `pnpm fix && pnpm pre-commit` -> Pass
- [x] **EVIDENCE**: [Walkthrough and Terminal Output](walkthrough.md)

**Completion**

- [x] ✅ **CLEAN LISTS**: All tasks and tests are passing.
- [x] 🛑 **TRUST BUT VERIFY**: All commands run and verified.
- [x] 🛑 **RESTART ON EDIT**: No pending edits.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**.

- [x] **Step 3.2: Balance View Container**
  - [x] **[NEW]** `apps/client/src/components/views/balance/balance-view-container.tsx`:
    - [x] Use `useBalanceData`.
    - [x] Use `useTaskActions` to dispatch updates.
    - [x] Render List of `BalanceItem`s.

**Quality Gates**

- [x] `pnpm fix && pnpm pre-commit` -> Pass
- [x] **EVIDENCE**: [Walkthrough and Terminal Output](walkthrough.md)

**Completion**

- [x] ✅ **CLEAN LISTS**: All tasks and tests are passing.
- [x] 🛑 **TRUST BUT VERIFY**: All commands run and verified.
- [x] 🛑 **RESTART ON EDIT**: No pending edits.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**.

## 4. Integration

### Steps

- [x] **Step 4.1: Add to Navigation**
  - [x] **[MODIFY]** `apps/client/src/viewmodel/ui/use-navigation-state.ts`:
    - [x] Add `'balance'` to `ActiveTab`.
  - [x] **[MODIFY]** `apps/client/src/components/shell/app-shell-container.tsx`:
    - [x] Add "Balance" Button to Desktop Sidebar.
    - [x] Add "Balance" Button to Mobile Bottom Bar.
    - [x] Render `<BalanceViewContainer />` when active.

**Quality Gates**

- [x] `pnpm fix && pnpm pre-commit` -> Pass
- [x] **EVIDENCE**: [Walkthrough and Terminal Output](walkthrough.md)

**Completion**

- [x] ✅ **CLEAN LISTS**: All tasks and tests are passing.
- [x] 🛑 **TRUST BUT VERIFY**: All commands run and verified.
- [x] 🛑 **RESTART ON EDIT**: No pending edits.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**.

## 5. Bug Fixes

- [x] **Step 5.1: Fix Credit Attribution**
  - [x] Implement TDD test case in `credits.test.ts`.
  - [x] Implement recursion/decay in `ops.ts`.
  - [x] Refactor `updateTask` to reduce complexity.
  - [x] Verify UI Projection in `projections.ts`.

**Quality Gates**

- [x] `pnpm fix && pnpm pre-commit` -> Pass
- [x] **EVIDENCE**: [Walkthrough and Terminal Output](walkthrough.md)

## 6. Refactor & Optimization

**Goal**: Extract complex logic from React components/hooks into pure, testable domain functions.

### Steps

- [ ] **Step 6.1: Refactor `useBalanceData`**
  - [ ] Extract aggregation logic to `packages/tasklens/src/domain/balance.ts` (or similar).
  - [ ] Create `calculateBalanceData(tasks: ComputedTask[]): BalanceItemData[]`.
  - [ ] Unit test the new function.
  - [ ] Simplify hook to use the new function.

- [ ] **Step 6.2: Refactor `BalanceViewContainer` Logic**
  - [ ] Extract "Drain/Fill" distribution logic to `packages/tasklens/src/domain/balance-distribution.ts`.
  - [ ] Create `distributeCredits(targetId, newValue, items): UpdateOp[]`.
  - [ ] Unit test the new function.
  - [ ] Simplify component to use the new function.

**Quality Gates**

- [ ] `pnpm fix && pnpm pre-commit` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix && pnpm pre-commit`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
  - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
  - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
  - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.

## 7. Future Work (Code Review Findings)

Items identified during code review (2024-12-29) for future cleanup.

### 7.1 Performance Optimization

- [ ] **Defer Slider Writes to `onChangeEnd`**
  - Currently, `handleDesiredCreditsChange` fires on every `onChange` (every pixel drag).
  - Heavy on Automerge transaction history.
  - **Fix**: Introduce local "virtual" state in `BalanceViewContainer`, update only on `onChangeEnd`.
  - See TODO comments in `balance-view-container.tsx` L121-131 and `balance-item.tsx` L46-47.

### 7.2 Code Cleanup

- [x] **Align `createMockTask` Default**
  - `createMockTask` defaults `desiredCredits: 0`, but `createTask` defaults to `1.0`.
  - **File**: `packages/tasklens/src/test-utils.ts` L51.
  - **Fix**: Change to `desiredCredits: 1.0`.

- [ ] **Extract Starving Threshold Constant**
  - Magic number `0.9` in `isStarving` calculation is unexplained.
  - **File**: `apps/client/src/hooks/use-balance-data.ts` L51-52.
  - **Fix**: Extract to `STARVING_THRESHOLD = 0.9` with documentation.

### 7.3 Testing

- [ ] **Add Unit Tests for `useBalanceData`**
  - Hook contains non-trivial aggregation logic (root filtering, totals, starving detection).
  - Could be tested as pure function after Step 6.1 refactor.

- [ ] **Defensive Division Guard**
  - `totalItems` could theoretically be 0 in slider label.
  - **File**: `apps/client/src/components/views/balance/balance-item.tsx` L52.
  - Low risk (parent guards with empty state), but defensive `totalItems > 0 ?` check is safer.

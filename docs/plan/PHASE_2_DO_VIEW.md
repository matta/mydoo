# Implementation Plan: Phase 2 (The "Do" View)

**Goal**: Implement the read-only priority list ("Do" View) to prove the data pipeline and basic UI rendering works.

## Step 1: Implement `usePriorityList` Hook

_Create the projection hook that transforms raw Automerge data into a sorted, flat list of tasks._

- [x] Create `apps/client/src/viewmodel/projections/usePriorityList.ts`.
- [x] Implement logic to:
  - Flatten the task tree (if needed) or iterate tasks.
  - Filter by `status: Pending`.
  - Sort by `priority` (descending).
- [x] Write unit test: `tests/usePriorityList.test.tsx`.

**Validation**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass (Regression check)
- [x] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [x] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [x] 🛑 STOP and prompt for user review.
- [x] 💾 Request git commit.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: Did the commit succeed? If not, fix it immediately.

## Step 2: Implement UI Primitives

_Create the basic visualization components._

- [x] Create `apps/client/src/components/primitives/TaskRow.tsx`.
- [x] Create `apps/client/src/components/composites/PriorityTaskList.tsx`.
- [x] Implement Storybook/Preview (or just use basic dev references).

**Validation**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [x] 🛑 STOP and prompt for user review.
- [x] 💾 Request git commit.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: Did the commit succeed? If not, fix it immediately.

## Step 3: Implement Do View Container

_Wire everything together in the main view._

- [x] Create `apps/client/src/viewmodel/containers/DoViewContainer.tsx`.
- [x] Use `usePriorityList` and `PriorityTaskList`.
- [x] Update `App.tsx` (or Router) to show `DoViewContainer` at `/`.

**Validation**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [x] 🛑 STOP and prompt for user review.
- [x] 💾 Request git commit.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: Did the commit succeed? If not, fix it immediately.

## Step 4: End-to-End Verification

_Verify with real data._

- [x] Create a "Seed Data" mechanism (button/script) to populate Automerge with test tasks.
- [x] Verify sorting matches expectations visually.
- [x] Write E2E test verifying list rendering.

**Validation**

- [x] `pnpm test:e2e` -> New test passes.

**Completion**

- [x] 🛑 STOP and prompt for user review.
- [x] 💾 Request git commit.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: Did the commit succeed? If not, fix it immediately.

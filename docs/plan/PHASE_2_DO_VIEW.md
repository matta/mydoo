# Implementation Plan: Phase 2 (The "Do" View)

**Goal**: Implement the read-only priority list ("Do" View) to prove the data pipeline and basic UI rendering works.

## Step 1: Implement `usePriorityList` Hook

_Create the projection hook that transforms raw Automerge data into a sorted, flat list of tasks._

- [ ] Create `apps/client/src/viewmodel/projections/usePriorityList.ts`.
- [ ] Implement logic to:
  - Flatten the task tree (if needed) or iterate tasks.
  - Filter by `status: Pending`.
  - Sort by `priority` (descending).
- [ ] Write unit test: `tests/usePriorityList.test.tsx`.

**Validation**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm test` -> Pass (New tests passing)

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 2: Implement UI Primitives

_Create the basic visualization components._

- [ ] Create `apps/client/src/components/primitives/TaskRow.tsx`.
- [ ] Create `apps/client/src/components/composites/PriorityTaskList.tsx`.
- [ ] Implement Storybook/Preview (or just use basic dev references).

**Validation**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 3: Implement Do View Container

_Wire everything together in the main view._

- [ ] Create `apps/client/src/viewmodel/containers/DoViewContainer.tsx`.
- [ ] Use `usePriorityList` and `PriorityTaskList`.
- [ ] Update `App.tsx` (or Router) to show `DoViewContainer` at `/`.

**Validation**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm dev` -> App loads and displays "No tasks" or placeholder state.
- [ ] `pnpm test:e2e` -> Smoke test still passes.

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 4: End-to-End Verification

_Verify with real data._

- [ ] Create a "Seed Data" mechanism (button/script) to populate Automerge with test tasks.
- [ ] Verify sorting matches expectations visually.
- [ ] Write E2E test verifying list rendering.

**Validation**

- [ ] `pnpm test:e2e` -> New test passes.

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

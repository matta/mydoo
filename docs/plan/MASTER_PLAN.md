# View Layer Implementation: Master Plan

This document tracks the rollout of the `mydoo` View Layer. The implementation is broken down into **Vertical Slices** to allow for "Testing As You Go" and avoiding a "big bang" integration.

> **Status Tracking**: Mark items as `[x]` when completed and committed.

## Workflow Rules (Strict Enforcement)

1.  **Phase Documentation**: Before starting any phase, you **MUST** create a detailed `docs/plan/PHASE_<N>_<NAME>.md` document. This document must strictly adhere to the following structure:

    - **Clean Git State**: Before starting any step, you **MUST** verify that `git status` is clean (no uncommitted changes).
    - **Discrete Steps**: The Phase Doc must break the work down into discrete, atomic steps (e.g., "Implement one hook").
    - **Test Coverage**: All logic changes must be covered by tests as specified in [`docs/design/test-suite.md`](../design/test-suite.md).
    - **Quality Gates and Stop Protocol (MANDATORY)**:
      For **EVERY SINGLE STEP** in your Phase Document, you **MUST** copy-paste the following block **VERBATIM**. You are **NOT ALLOWED** to summarize or shorten it. It must appear exactly as below at the end of every step:

      ```markdown
      **Quality Gates**

      - [ ] `pnpm fix && pnpm check` -> Pass
      - [ ] **EVIDENCE**: Show terminal output of passing tests.

      **Completion**

      - [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
      - [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
      - [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix && pnpm check`). They must all pass in sequence against the same repository state.
      - [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
      - [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
      - [ ] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
        - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
        - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
        - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.
      ```

    - **Final Review**: After the entire phase is complete, you must follow the same "Stop & Wait" protocol.
      - Return to the MASTER_PLAN.md to continue working on the next phase.

---

## Completed Phases

- **Phase 1: Foundation & Infrastructure** [Complete]
- **Phase 2: The "Do" View** [Complete]
- **Phase 3: Core Interactions** [Complete]
- **Phase 3.5: Task Creation Defaults** [Complete]
- **Phase 4: Task Details & Editing** [Complete]
- **Phase 5: The "Plan" View (Tree)** [Complete]

---

## Phase 6: TaskLens Type Rationalization
**Rationale**: Clarifies the boundary between Persisted State (DB) and Computed View State. Prevents accidental mutation of CRDT history with transient scores. Makes the data flow explicit: `DB -> Algo -> View`.

- [x] **Define Exact Types**:
  - [x] `PersistedTask` (Schema properties only): `id`, `title`, `status`, `importance`, `schedule`, `isAcknowledged`, etc.
  - [x] `ComputedTask` (Immutable Veneer): `Readonly<PersistedTask & ComputedProps>`
    - `ComputedProps`: **REQUIRED** `priority`, `visibility`, `effectiveCredits`, `normalizedImportance`, `feedbackFactor`.
    - **Goal**: View layer receives guaranteed immutable objects. No accidental invalidation of Automerge proxies.
- [x] **Refactor Algorithm**:
  - [x] Input: `Readonly<PersistedTask>[]`
  - [x] Output: `ComputedTask[]` (New objects, not modified proxies)
  - [x] `recalculatePriorities` becomes a pure function returning the veneer types.
- [x] **Strict Enforcement**:
  - [x] Update `TunnelState` to strictly use `PersistedTask` record.
  - [x] Fix all TS errors resulting from the split.

## Phase 7: The Balance View
**Goal**: Implement the "Life Balance" algorithm visualization.

**PRD References**: [§4.4 Balance View](../design/prd.md)

- [ ] **Step 1: Balance Logic**
  - [ ] Implement `useBalanceData` (Compute Target % vs Actual %)
  - [ ] Implement `desiredCredits` updates
- [ ] **Step 2: Balance UI**
  - [ ] Create `BalanceItem` row (Slider for Target, Progress Bar for Actual)
  - [ ] Implement `BalanceViewContainer` (List Root Goals)
  - [ ] **Verification**: E2E for adjusting Desired Credits.

## Phase 8: Contexts & Places
**Goal**: Manage spatial and temporal contexts to filter the "Do" list.

**PRD References**: [§4.1 Global Nav](../design/prd.md), [§4.2 Do View (Filter)](../design/prd.md), [ALGORITHM.md §3.2 Places](../design/algorithm.md)

- [ ] **Step 1: Place Logic & State**
  - [ ] Implement `usePlaceIntents` (CRUD: Create, Update, Delete w/ Orphan logic)
  - [ ] Implement `usePlaces` (List provider)
  - [ ] Implement `deletePlace` op (Reassigns tasks to `ANYWHERE`)
  - [ ] **Verification**: Unit tests for Place CRUD and Deletion logic.
- [ ] **Step 2: Place Management UI**
  - [ ] Create `PlaceList` component (Context View)
  - [ ] Create `PlaceEditorModal`:
    - Name input
    - Schedule Editor (Hours: 24/7 vs Custom)
    - Inclusion Picker ("Inside of...")
  - [ ] Implement `ContextViewContainer`
  - [ ] **Verification**: E2E for creating and editing places.
- [ ] **Step 3: Integration**
  - [ ] Add `PlacePicker` to `TaskEditorModal` (Assign places)
  - [ ] Add `ContextFilter` dropdown to `DoViewContainer`
  - [ ] Update `useNavigationState` to handle `placeFilter`
  - [ ] **Verification**: E2E: Create Place -> Assign Task -> Filter View -> Verify Visibility.

## Phase 9: Accessibility & Final Polish
**Goal**: Ensure the app is accessible and production-ready.

- [ ] Audit semantic HTML and ARIA labels
- [ ] Verify keyboard navigation
- [ ] Verify screen reader announcements
- [ ] Mobile PWA Manifest & Icon verification

**Nits (Low Priority)**:
- [ ] Improve E2E test seeding reliability (currently relies on Dev menu visibility)
- [ ] Consider memoization for large OutlineTree lists
- [ ] Refactor `TaskOutlineItem` props: 12 props is approaching threshold; consider grouping callbacks (e.g., `actions: TaskActions`) or using context


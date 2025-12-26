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
      ```

    - **Final Review**: After the entire phase is complete, you must follow the same "Stop & Wait" protocol.
      - Return to the MASTER_PLAN.md to continue working on the next phase.

---

## Phase 1: Foundation & Infrastructure

_Goal: Setup tooling, dependencies, and core data wiring._

- [x] **Infrastructure Setup**
  - [x] Install UI Library dependencies (Mantine v7)
  - [x] Configure `vitest` and `playwright` for `apps/client`
  - [x] Create directory structure (`viewmodel/`, `components/`, `tests/`)
  - [x] Create `AppShell` layout skeleton
- [x] **Data Wiring**
  - [x] Implement `useDocument` (doc handle provider)
  - [x] Verify Automerge connection in React (smoke test)

## Phase 2: The "Do" View (Vertical Slice 1)

_Goal: Read-only rendering of the priority list. Prove the data pipeline works._

- [x] **Logic (TDD)**
  - [x] Test & Implement `usePriorityList` hook
  - [ ] Test & Implement `useSystemIntents` (Healer/Refresh) **[MISSED - SEE REMEDIATION]**
- [x] **UI Components**
  - [x] Create `TaskRow` primitive
  - [x] Create `PriorityTaskList` composite
  - [x] Implement `DoViewContainer`
- [x] **Verification**
  - [x] Unit Test: `usePriorityList` sorting
  - [x] E2E: App loads and displays tasks from IDB

## Phase 3: Core Interactions (Vertical Slice 2)

_Goal: Make the list interactive. Complete, Uncomplete, Quick Add._

- [x] **Logic (TDD)**
  - [x] Test & Implement `useTaskIntents` (Complete, Create)
- [x] **UI Integration**
  - [x] Wire `TaskRow` checkbox to `completeTask`
  - [x] Add "Quick Entry" input to `DoViewContainer`
- [x] **Verification**

  - [x] Integration Test: Click checkbox -> Store updates
  - [x] E2E: Create task -> Realtime update

- [x] **Deferred / Known Gaps** _(Migrated to Phases 3.5, 4, 5)_
  - [x] ~~Task Creation Inheritance (PRD §3.2)~~ → **Phase 3.5**
  - [x] ~~Task Details Modal (PRD §4.5)~~ → **Phase 4**
  - [x] ~~Mobile FAB (PRD §4.2)~~ → **Phase 4** (with Task Details)

## Phase 2/3 Remediation Plan (Complete)

_Goal: Fix regressions and missing specs identified during Phase 3 verification._

- [x] **Fix: Immediate Disappearance**
  - [x] Update `usePriorityList` to include `isAcknowledged=false` tasks (even if status=Done)
  - [x] Verify strikethrough styling for Done tasks
- [x] **Fix: Missing System Intents**
  - [x] Implement `useSystemIntents` (refresh/acknowledge logic)
  - [x] Add "Update/Refresh" button to `DoViewContainer`
- [x] **Fix: Task Creation Logic** _(Migrated to Phase 3.5)_
  - [x] ~~Enforce PRD defaults (Section 3.2) in `createTask`~~ → **Phase 3.5**
  - [x] ~~Enforce hierarchy rules (Inbox vs TLI)~~ → **Phase 3.5**
  - [x] ~~Implement Task Details Modal (PRD §4.5)~~ → **Phase 4**
  - [x] ~~Implement FAB for Mobile Task Creation (PRD §4.2)~~ → **Phase 4**
- [x] **Verification**
  - [x] E2E: Complete task -> Stays visible -> Click Refresh -> Disappears

---

## Phase 3.5: Task Creation Defaults (Complete)

_Goal: Ensure tasks are created with correct PRD-specified defaults and hierarchy rules._

**PRD References**: [§3.2 Default Values for New Tasks](../design/prd.md)

- [x] **Logic**
  - [x] Update `createTask` in `useTaskIntents` to accept optional `parentId`
  - [x] Implement `placeId` inheritance in `ops.createTask`:
    - Root tasks default to `ANYWHERE_PLACE_ID`
    - Child tasks inherit `placeId` from parent
- [x] **Verification**
  - [x] Unit tests for default value application
  - [x] Unit tests for inheritance from parent
  - [x] E2E tests for task creation
- [x] **Documentation**
  - [x] Updated PRD §3.2 to clarify field mappings (effort→creditIncrement, leadTimeMs, status Pending=active)

**Deferred:**

- `lastReviewTimestamp` initialization → Phase 6+ (Staleness/Autofocus)

**Commits**: `0dcd64b`, `1f9d3dc`, `263af85`

## Phase 4: Task Details & Editing
_Goal: Full property editing via modal. Add Sibling/Child creation. Delete with confirmation._

**PRD References**: [§4.5 Task Editing (Details Modal)](../design/prd.md), [§3.6 Deletion Logic](../design/prd.md)

- [x] **Logic**
  - [x] Implement `useTaskDetails` (fetch task + parent title + descendant count)
  - [x] Extend `useTaskIntents` with `deleteTask` (cascade to descendants)
- [x] **UI Components**
  - [x] Create `TaskEditorModal` (PRD §4.5):
    - Title input
    - Parent (read-only) + "Move..." button + "Find in Plan" button
    - Importance slider (0.0 - 1.0)
    - Effort picker (1 | 3 | 5)
    - Due Date picker, Lead Time input, Repeat selector
    - Place dropdown
    - Notes textarea
    - Footer: Add Sibling, Add Child, Delete
  - [x] Create `DeleteConfirmModal` (PRD §3.6: show descendant count)
  - [ ] Implement Mobile FAB for quick Inbox add (PRD §4.2)
- [ ] **Verification**
  - [x] Unit tests for `useTaskDetails`
  - [x] Integration tests for modal interactions
  - [ ] E2E: Edit task -> Save -> Verify changes _(Deferred to Phase 5)_
  - [ ] E2E: Delete task with children -> cascade _(Deferred to Phase 5)_

## Phase 5: The "Plan" View (Tree)
_Goal: Hierarchical tree navigation with expand/collapse._

**PRD References**: [§4.3 The Plan View](../design/prd.md)

> **🔴 Current Priority**: Implement Critical Create Flow Fixes (Highlight & Reveal).

- [x] **Step 1-3: Logic & UI (Complete)**
  - [x] Implement `useTaskTree` (Recursive projection)
  - [x] Implement `useBreadcrumbs`
  - [x] Implement `useNavigationState` (Expansion state, viewPath)
  - [x] Create `OutlineTree` component (Recursive, with chevrons)
  - [x] Create `TaskOutlineItem` primitive
  - [x] Implement `PlanViewContainer`
  - [x] Unit tests for tree projection, breadcrumbs, keyboard navigation
- [x] **Step 4: Make Outline Visible (Complete)**
  - [x] Verify route/tab integration loads `PlanViewContainer`
  - [x] Visual verification: confirm tree renders in running app
  - [x] E2E baseline for outline view
- [x] **Step 5: Mobile Drill-Down Navigation (Complete)**
  - [x] Implement strict viewport modes (Tree vs Drill-Down)
  - [x] Implement scrollable breadcrumb trail for mobile
  - [x] Tap row/title opens Task Editor; tap arrow drills down
  - [x] E2E tests for mobile navigation
- [ ] **Step 6: Critical Create Flow Fixes (Highlight & Reveal)**
  - [ ] Add "More" menu to TaskOutlineItem (Add Child, Add Sibling)
  - [ ] Implement "Highlight & Reveal" (Auto-expand, Auto-scroll, Flash)
- [ ] **Remaining Work**
  - [ ] Create `MovePickerModal` (select new parent + position)
  - [ ] E2E: Move task -> Verify tree updates correctly
  - [ ] E2E: Task Editor verification _(Deferred from Phase 4)_
  - [ ] E2E: Task Deletion cascade _(Deferred from Phase 4)_
  - [ ] Enable skipped E2E test: `should persist task after page reload`
  - [ ] Replace browser `confirm()` with Mantine modal in `TaskEditorContainer` _(Polish)_

## Phase 6: TaskLens Type Rationalization
**Rationale**: Clarifies the boundary between Persisted State (DB) and Computed View State. Prevents accidental mutation of CRDT history with transient scores. Makes the data flow explicit: `DB -> Algo -> View`.

- [ ] **Define Exact Types**:
  - `PersistedTask` (Schema properties only): `id`, `title`, `status`, `importance`, `schedule`, `isAcknowledged`, etc.
  - `ComputedTask` (Immutable Veneer): `Readonly<PersistedTask & ComputedProps>`
    - `ComputedProps`: **REQUIRED** `priority`, `visibility`, `effectiveCredits`, `normalizedImportance`, `feedbackFactor`.
    - **Goal**: View layer receives guaranteed immutable objects. No accidental invalidation of Automerge proxies.
- [ ] **Refactor Algorithm**:
  - Input: `Readonly<PersistedTask>[]`
  - Output: `ComputedTask[]` (New objects, not modified proxies)
  - `recalculatePriorities` becomes a pure function returning the veneer types.
- [ ] **Strict Enforcement**:
  - Update `TunnelState` to strictly use `PersistedTask` record.
  - Fix all TS errors resulting from the split.

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


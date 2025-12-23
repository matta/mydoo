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

- [ ] **Deferred / Known Gaps**
  - [-] Task Creation Inheritance (PRD §3.2)
  - [-] Task Details Modal (PRD §4.5)
  - [-] Mobile FAB (PRD §4.2)

## Phase 2/3 Remediation Plan (Active)

_Goal: Fix regressions and missing specs identified during Phase 3 verification._

- [x] **Fix: Immediate Disappearance**
  - [x] Update `usePriorityList` to include `isAcknowledged=false` tasks (even if status=Done)
  - [x] Verify strikethrough styling for Done tasks
- [x] **Fix: Missing System Intents**
  - [x] Implement `useSystemIntents` (refresh/acknowledge logic)
  - [x] Add "Update/Refresh" button to `DoViewContainer`
- [ ] **Fix: Task Creation Logic** [DEFERRED]
  - [-] Enforce PRD defaults (Section 3.2) in `createTask` (currently minimal)
  - [-] Enforce hierarchy rules (Inbox vs TLI)
  - [-] Implement Task Details Modal (PRD §4.5)
  - [-] Implement FAB for Mobile Task Creation (PRD §4.2)

- [x] **Verification**
  - [x] E2E: Complete task -> Stays visible -> Click Refresh -> Disappears

## Phase 4: The "Plan" View (Tree)

_Goal: Hierarchical navigation._

- [ ] **Logic**
  - [ ] Implement `useTaskTree` (Recursive projection)
  - [ ] Implement `useBreadcrumbs`
  - [ ] Implement `useNavigationState` (Expansion state)
- [ ] **UI Components**
  - [ ] Create `OutlineTree` component (Recursive)
  - [ ] Implement `PlanViewContainer`

## Phase 5: Task Details & Editing

_Goal: Full property editing._

- [ ] **Logic**
  - [ ] Implement `useTaskDetails` (Deep data fetch)
  - [ ] Implement `usePlaceIntents`
- [ ] **UI Components**
  - [ ] Create `TaskEditorModal` (The "Mega-Component")
  - [ ] Wire form handling (Validation, Dirty checking)
  - [ ] Create `MovePickerModal`

## Phase 6: Polish & Balance

_Goal: Complete the feature set._

- [ ] Implement `BalanceViewContainer`
- [ ] Implement `ContextViewContainer`
- [ ] Final Accessibility Audit

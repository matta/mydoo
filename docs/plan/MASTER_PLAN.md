# View Layer Implementation: Master Plan

This document tracks the rollout of the `mydoo` View Layer. The implementation is broken down into **Vertical Slices** to allow for "Testing As You Go" and avoiding a "big bang" integration.

> **Status Tracking**: Mark items as `[x]` when completed and committed.

## Workflow Rules (Strict Enforcement)

1.  **Phase Documentation**: Before starting any phase, you **MUST** create a detailed `docs/plan/PHASE_<N>_<NAME>.md` document. This document must strictly adhere to the following structure:
    - **Clean Git State**: Before starting any step, you **MUST** verify that `git status` is clean (no uncommitted changes).
    - **Discrete Steps**: The Phase Doc must break the work down into discrete, atomic steps (e.g., "Implement one hook").
    - **Test Coverage**: All logic changes must be covered by tests as specified in [`docs/design/test-suite.md`](../design/test-suite.md).
    - **Quality Gates (MANDATORY FOR EVERY STEP)**:
      The following **must all pass** before you prompt the user for manual review. You are NOT ALLOWED to skip any of these, even for "logic only" changes:
      - ✅ `pnpm fix` (1/4)
      - ✅ `pnpm build` (2/4)
      - ✅ `pnpm test` (3/4) - **MUST RUN ON ENTIRE REPO**
      - ✅ `pnpm test:e2e` (4/4)
      - ✅ **EVIDENCE**: You must present the terminal output/screenshot of passing tests to the user.
      - ✅ **CLEAN LISTS**: **MUST** update `task.md` and `PHASE_X.md` to mark completed steps as `[x]` before asking for review. Don't leave stale TODOs.
    - **Stop & Wait**: At the end of **every step**, you must:
      - 🛑 **STOP** execution.
      - 🗣️ **Prompt** the user for manual review with the EVIDENCE.
      - 💾 **Ask** to commit changes to git.
    - **Final Review**: After the entire phase is complete, you must:
      - 🗣️ **Quote** STOP AND WAIT
      - 💾 **Ask** to commit changes to git, if any.
      - 🛑 **VERIFY COMMIT SUCCESS**: Ensure the commit command succeeded (Exit Code 0). If it failed (e.g., lint error), you **MUST** fix it and retry before proceeding. Do NOT continue until `git status` is clean.
      - STOP AND WAIT for the user's response and act on it.
      - Return to the MASTER_PLAN.md to continue working on the next phase.

---

## Phase 1: Foundation & Infrastructure

_Goal: Setup tooling, dependencies, and core data wiring._

- [ ] **Infrastructure Setup**
  - [ ] Install UI Library dependencies (Mantine v7)
  - [ ] Configure `vitest` and `playwright` for `apps/client`
  - [ ] Create directory structure (`viewmodel/`, `components/`, `tests/`)
  - [ ] Create `AppShell` layout skeleton
- [ ] **Data Wiring**
  - [ ] Implement `useDocument` (doc handle provider)
  - [ ] Verify Automerge connection in React (smoke test)

## Phase 2: The "Do" View (Vertical Slice 1)

_Goal: Read-only rendering of the priority list. Prove the data pipeline works._

- [ ] **Logic (TDD)**
  - [ ] Test & Implement `usePriorityList` hook
  - [ ] Test & Implement `useSystemIntents` (Healer/Refresh)
- [ ] **UI Components**
  - [ ] Create `TaskRow` primitive
  - [ ] Create `PriorityTaskList` composite
  - [ ] Implement `DoViewContainer`
- [ ] **Verification**
  - [ ] Unit Test: `usePriorityList` sorting
  - [ ] E2E: App loads and displays tasks from IDB

## Phase 3: Core Interactions (Vertical Slice 2)

_Goal: Make the list interactive. Complete, Uncomplete, Quick Add._

- [ ] **Logic (TDD)**
  - [ ] Test & Implement `useTaskIntents` (Complete, Create)
- [ ] **UI Integration**
  - [ ] Wire `TaskRow` checkbox to `completeTask`
  - [ ] Add "Quick Entry" input to `DoViewContainer`
- [ ] **Verification**
  - [ ] Integration Test: Click checkbox -> Store updates
  - [ ] E2E: Create task -> Realtime update

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

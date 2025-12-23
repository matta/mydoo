# Implementation Plan: Phase 5 (The Plan View)

**Goal**: Hierarchical tree navigation with expand/collapse and breadcrumbs.

**PRD References**: [§4.3 The Plan View](../design/prd.md)

---

> [!IMPORTANT]
> **Critical Path**: The outline view must be visible before any navigation or reparenting features can be meaningfully tested or developed.

---

## Step 1: Logic & State ✅

_Implemented the core tree projection and navigation state._

- [x] Implement `useNavigationState` (manages `expandedIds` set, `viewPath` stack)
- [x] Implement `useTaskTree` projection (recursive tree builder, sorting, filtering)
- [x] Unit tests for tree projection

**Committed**: All quality gates passed.

---

## Step 2: Tree UI Components ✅

- [x] Create `TaskOutlineItem` component (row renderer)
- [x] Create `OutlineTree` component (recursive list)
- [x] Update `PlanViewContainer` to render the tree

**Committed**: All quality gates passed.

---

## Step 3: Interaction & Breadcrumbs ✅

- [x] Implement `useBreadcrumbs` (path from root to current view)
- [x] Connect `TaskOutlineItem` actions (Drill down, Toggle completion)
- [x] Add "Breadcrumbs" for navigation history in `PlanViewContainer`
- [x] Keyboard navigation tests

**Committed**: All quality gates passed.

---

## Step 4: Make Outline Visible 🔴 CURRENT PRIORITY

> [!CAUTION]
> **Blocker**: The Plan View outline is not currently accessible in the running app. This must be resolved before any further Phase 5 work.

### Tasks

- [x] **Verify Route/Tab Integration**
  - [x] Confirm the "Plan" tab in the `AppShellContainer` / navigation shell loads `PlanViewContainer`
  - [x] Check that the mobile tab bar includes the Plan tab
  - [x] Verify desktop split-pane layout renders `PlanViewContainer` in the sidebar

- [x] **Implement Dev Tools / Seeding UI**
  - [x] Add visual controls (e.g., in a Dev/Debug menu or bottom bar) to populate sample data
  - [x] Sample data must include hierarchical structures (multi-level nesting) to test tree visualization
  - [x] Validation: Seeding should function correctly even if the document is empty or partially populated

- [x] **Visual Verification**
  - [x] Load the app in development mode (`pnpm dev`)
  - [x] Navigate to the Plan view on both mobile and desktop viewport sizes
  - [x] Confirm the tree renders with correct indentation and chevrons
  - [x] Verify breadcrumb bar appears and functions

- [x] **E2E Baseline**
  - [x] Ensure existing `plan.spec.ts` E2E tests exercise the outline view
  - [x] Verify navigation, expand/collapse, and basic interactions work
  - [x] **New**: Verify that the seeding UI populates the tree structure correctly in E2E tests

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
  - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
  - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
  - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.

---

## Step 5: Mobile Drill-Down Navigation

_Handle deep navigation on small screens with breadcrumb trail._

**PRD Reference**: [§4.3 The "Plan" View (Outline)](../design/prd.md) — "Mobile: Drill-down navigation. Tapping a parent 'zooms in' to show only its children. A breadcrumb trail at the top shows the current path and allows navigation back up."

### Tasks

- [ ] **Implement Drill-Down Interaction**
  - On mobile viewport, tapping the "Drill Down" arrow icon pushes that task's ID onto the `viewPath` stack
  - `PlanViewContainer` should render only the children of the current `viewPath` head (not the full tree)
  - **New Requirement**: Tapping the task title (or rest of row) opens the Task Editor modal

- [ ] **Breadcrumb Trail for Mobile**
  - Mobile breadcrumbs should be horizontally scrollable if path is long
  - Each breadcrumb segment is tappable to navigate directly to that level
  - "Root" or home icon at the start to return to top-level view
  - Current location should be visually distinct (not clickable)

- [ ] **Navigation Actions**
  - `navigateTo(path: TaskID[])` — Set the entire path stack
  - `navigateUp()` — Pop the last item from the stack (go up one level)
  - Hardware/gesture back should trigger `navigateUp()` on mobile

- [ ] **Responsive Behavior**
  - Desktop: Full tree visible with expand/collapse (no drill-down)
  - Mobile: Drill-down with breadcrumbs
  - Switching between viewports should handle gracefully (e.g., if drilled down on mobile and resize to desktop, show expanded tree at that position)

### Verification

- [ ] Unit tests for `useNavigationState` path manipulation (`navigateTo`, `navigateUp`)
- [ ] Component tests for mobile breadcrumb rendering and tap interactions
- [ ] E2E test: On mobile viewport, tap parent → verify only children visible → tap breadcrumb → verify navigation back
- [ ] **Deferred from Step 4**: E2E test for 4-level deep navigation (Deep Work Project → Module A → Component X → Unit Test)

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass (ALL repo tests)
- [ ] `pnpm test:e2e` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

---

## Step 6: MovePickerModal & Reparenting

_Allow users to reorganize tasks by changing parent and position._

**PRD Reference**: [§4.5 Task Editing](../design/prd.md) — "Move... Button: Opens a picker modal allowing the user to: 1) Select a new parent (reparenting), and/or 2) Choose position among siblings (reordering within the same or new parent). This is the only mechanism for reorganizing tasks—no drag-and-drop is required for MVP."

### Tasks

- [ ] **Create `MovePickerModal` Component** ([view-layer.md §5.6](../design/view-layer.md))
  - Modal displays a tree/list of potential parent tasks
  - **Exclusion filter**: Must exclude the task being moved AND all its descendants (prevents circular references)
  - For each potential parent, show option to choose position among its children:
    - "At the beginning"
    - "After [sibling name]" dropdown
  - Visual distinction for current parent (if staying in same location)
  - Loading state while calculating valid targets

- [ ] **Implement `useValidParentTargets` Hook**
  - Input: `docUrl`, `taskIdBeingMoved`
  - Output: List of valid parent tasks (all tasks minus self and descendants)
  - Memoized computation to avoid recalculating on every render

- [ ] **Implement `moveTask` Operation**
  - Signature: `moveTask(taskId: TaskID, newParentId: TaskID | undefined, afterSiblingId: TaskID | undefined)`
  - `newParentId = undefined` means move to root level
  - `afterSiblingId = undefined` means insert at the beginning
  - Updates:
    1. Remove task ID from old parent's `childTaskIds` (or `rootTaskIds` if was root)
    2. Update task's `parentId` field
    3. Insert task ID into new parent's `childTaskIds` (or `rootTaskIds`) at correct position

- [ ] **Wire into Task Editor Modal**
  - "Move..." button in `TaskEditorModal` opens `MovePickerModal`
  - After successful move, close both modals and refresh tree view

- [ ] **Navigation State Integration**
  - `openMovePicker(taskId)` and `closeMovePicker()` actions in `useNavigationState`
  - `MovePickerContainer` renders when `nav.movePickerTaskId` is set

### Verification

- [ ] Unit tests for `useValidParentTargets` (excludes self and descendants)
- [ ] Unit tests for `moveTask` operation (correct array manipulation)
- [ ] Component tests for `MovePickerModal` (renders targets, handles selection)
- [ ] E2E test: Open task editor → Click "Move..." → Select new parent → Verify tree updates correctly

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass (ALL repo tests)
- [ ] `pnpm test:e2e` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

---

## Step 7: Deferred E2E Verification (from Phase 4)

_Catch up on E2E tests that were deferred from Phase 4._

### Tasks

- [ ] **E2E: Edit Task Properties**
  - Click task title in Plan View to open Task Editor modal
  - Modify title, importance, effort
  - Save changes
  - Verify changes persisted (reload page, check values)

- [ ] **E2E: Delete Task with Cascade**
  - Create or identify a task with children
  - Open Task Editor for parent task
  - Click "Delete"
  - Verify confirmation dialog shows descendant count (per PRD §3.6)
  - Confirm deletion
  - Verify parent AND all children are removed from the tree

- [ ] **E2E: Page Reload Persistence**
  - Fix seed parameter issue preventing this test
  - Create a task
  - Reload the page
  - Verify task is still present

### Verification

- [ ] All three E2E scenarios pass in both Chromium and WebKit
- [ ] Enable any previously skipped tests (`should persist task after page reload`)

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass (ALL repo tests)
- [ ] `pnpm test:e2e` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

**Completion**

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

---

## Phase 5 Completion Criteria

Phase 5 is complete when ALL of the following are true:

| Criterion | Status |
|-----------|--------|
| Plan view is visible and navigable in running app | ⬜ |
| Breadcrumbs work for navigation history | ✅ |
| Mobile drill-down navigation works | ⬜ |
| MovePickerModal allows reparenting and reordering | ⬜ |
| All deferred E2E tests pass | ⬜ |
| All quality gates pass for each step | ⬜ |
| All commits verified successful | ⬜ |

---

## Files to Create/Modify

| File | Action | Step |
|------|--------|------|
| `components/modals/MovePickerModal.tsx` | CREATE | Step 6 |
| `viewmodel/containers/MovePickerContainer.tsx` | CREATE | Step 6 |
| `viewmodel/projections/useValidParentTargets.ts` | CREATE | Step 6 |
| `viewmodel/useNavigationState.ts` | MODIFY | Steps 5, 6 |
| `components/views/plan/PlanViewContainer.tsx` | MODIFY | Steps 4, 5 |
| `e2e/plan.spec.ts` | MODIFY | Steps 5, 6, 7 |
| `e2e/task-editor.spec.ts` | MODIFY | Step 7 |

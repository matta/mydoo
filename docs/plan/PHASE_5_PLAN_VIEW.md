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

## Step 4: Make Outline Visible ✅

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

- [x] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [x] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [x] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [x] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: The repo has strict presubmit hooks (lint-staged, commitlint).
  - **ACTION**: You **MUST** check the terminal output and exit code of `git commit` immediately after running it.
  - **IF FAILED**: You **MUST** read the error message, fix the issue (e.g., shorten commit message, fix linting), and **RETRY** the commit until it succeeds.
  - **CRITICAL**: Do NOT mark the step as done or proceed to the "STOP AND WAIT" step until `git log` confirms the new commit exists and `git status` is clean.

---

## Step 5: Mobile Drill-Down Navigation 🔴 CURRENT PRIORITY

_Handle deep navigation on small screens with breadcrumb trail._

**PRD Reference**: [§4.3 The "Plan" View (Outline)](../design/prd.md) — "Mobile: Drill-down navigation. Tapping a parent 'zooms in' to show only its children. A breadcrumb trail at the top shows the current path and allows navigation back up."

### Tasks

- [x] **Implement Drill-Down Interaction**
  - **Interaction Model (Codified)**:
    - **Tap "Drill Down" Arrow**: Pushes task ID to `viewPath` (Drill Down)
    - **Tap Row / Title**: Opens Task Editor modal
    - **New Requirement**: Tapping the task title (or rest of row) opens the Task Editor modal
  - **Task Editor Implementation**:
    - Create `viewmodel/containers/TaskEditorContainer.tsx` (connects storage to `TaskEditorModal`)
    - Mount `<TaskEditorContainer />` in `AppShellContainer` (conditionally rendered when `editingTaskId` is set)
    - Ensure `useNavigationState` properly tracks `editingTaskId`
    - **Long Press**: No action (reserved)

- [x] **Breadcrumb Trail for Mobile**
  - Mobile breadcrumbs should be horizontally scrollable if path is long
  - Each breadcrumb segment is tappable to navigate directly to that level
  - "Root" or home icon at the start to return to top-level view
  - Current location should be visually distinct (not clickable)

- [x] **Navigation Actions**
  - `navigateTo(path: TaskID[])` — Set the entire path stack
  - `navigateUp()` — Pop the last item from the stack (go up one level)
  - Hardware/gesture back should trigger `navigateUp()` on mobile

- [x] **Strict Viewport Modes**
  - **Mobile (< 768px)**:
    - **Drill-Down Mode**: Show content of `viewPath` head only.
    - **Icons**: Show "Drill Down" arrow (Right). **Hide** Expand Chevron.
    - **Breadcrumbs**: **Visible** (scrollable).
  - **Desktop (>= 768px)**:
    - **Tree Mode**: Show full expandable tree.
    - **Icons**: Show "Expand" Chevron (Left). **Hide** Drill-Down Arrow.
    - **Breadcrumbs**: **Hidden** (Tree structure provides context).
  - **Switching behavior**:
    - Mobile -> Desktop: Reset `viewPath` to empty (show full tree), try to preserve `expandedIds`.
    - Desktop -> Mobile: If `expandedIds` has deep items, maybe auto-drill? (Optional: Start at root for simplicity).

### Verification

- [x] Unit tests for `useNavigationState` path manipulation (`navigateTo`, `navigateUp`)
- [x] Component tests for mobile breadcrumb rendering and tap interactions
- [x] E2E test: On mobile viewport, tap parent → verify only children visible → tap breadcrumb → verify navigation back
- [x] **Deferred from Step 4**: E2E test for 4-level deep navigation (Deep Work Project → Module A → Component X → Unit Test)

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
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

---

## Step 6: Critical Create Flow Fixes (Workflowy Polish) 🔴 CURRENT PRIORITY

_Implementing "Workflowy-like" interactions for seamless structure management._

### Sub-Step 6A: Modal Create Mode (Prerequisite)

**Goal**: Enable the Task Editor modal to create new tasks, not just edit existing ones.

> [!IMPORTANT]
> This is a prerequisite for Sub-Steps 6B and 6C. Both desktop and mobile create flows depend on "Create Mode."

> [!WARNING]
> **API Gap**: Current `createTask(state, props)` always inserts at **top** (`unshift`). The spec requires:
> - "Add Sibling" → insert **immediately after** reference task
> - "Add Child" → insert at **end** of children
> 
> The `createTask` API must be extended with an optional `afterTaskId` parameter (similar to `moveTask`).

- [ ] **API**: Extend `ops.createTask` to support flexible positioning
  - `createTask(state, props, options?)` where `options` is one of:
    - `{position: 'start'}` — prepend to beginning of children (for Bottom Bar `[+]`)
    - `{position: 'end'}` — append to end of children (for Append Row, Add Child)
    - `{position: 'after', afterTaskId: TaskID}` — insert after specified task (for Add Sibling)
  - Defaults to `position: 'end'` if not specified.
  - Update `@mydoo/tasklens` with this change.

- [ ] **State**: Add to `useNavigationState`
  - `createTaskParentId: TaskID | null` — when set, modal opens in **Create Mode**.
  - `createTaskAfterTaskId: TaskID | undefined` — for "Add Sibling", the reference task to insert after.
  - `editingTaskId` for Edit Mode, `createTaskParentId` for Create Mode — mutually exclusive.
  - `openCreateModal(parentId: TaskID | null, afterTaskId?: TaskID)` — opens Create Mode.
  
- [ ] **Component**: `TaskEditorModal` — Support Create Mode
  - **When in Create Mode**:
    - Form fields are empty (default values).
    - "Parent" display shows the target parent's title (or "Root").
    - **Hierarchy Controls** (Indent/Outdent/Move) are **HIDDEN**.
    - Save button creates new task via `createTask(title, parentId, afterTaskId)`.
  - **When in Edit Mode**: Existing behavior (loads task, saves updates).

- [ ] **Container**: `TaskEditorContainer` — Handle both modes
  - Determine mode from navigation state.
  - Provide appropriate save handler based on mode.

### Sub-Step 6B: Desktop Interaction (Hover Menu)

**Goal**: Controls appear on the left. The "Menu/Bullet" is the anchor.

> [!NOTE]
> This layout change applies to **desktop (tree mode) only**. Mobile retains the existing `[Checkbox] [Title] [Drill Arrow]` layout from Step 5.

- [ ] **Component**: `TaskOutlineItem` (Desktop Variant)
  - **Layout**: `[Indent Spacer] [Menu Trigger (•••)] [Chevron] [Checkbox] [Title]`
  - **Behavior**:
    - `Menu Trigger`: Acts as the "Bullet".
      - Default State: Looks like a Bullet (•) or faint icon.
      - Hover State: Becomes `•••` (Horizontal Dots).
    - **Expansion Logic**:
      - The Chevron occupies fixed space (e.g., 16px) between the Menu and the Checkbox.
      - If the task has no children, the space remains (or contains a faint dot) to maintain alignment.
      - **Final Layout**: `[Menu] [ChevronContainer (16px)] [Checkbox] [Title]`
  - **Actions**:
    - Menu Click: Opens dropdown (Add Sibling, Add Child, Delete).
      - **Note**: Delete in the quick menu is intentional for faster workflows. Confirmation dialog still applies for tasks with children.
    - Chevron Click: Toggles expansion.

### Sub-Step 6C: Mobile Interaction (Bottom Bar & Drill-Down)

**Goal**: Persistent navigation and creation controls within the current zoom context.

> [!NOTE]
> The **Row Layout** behavior below is already implemented in Step 5. It is documented here for context only.
> New work in this step: **Bottom Bar** (entirely new component) and **Append Row**.

> [!IMPORTANT]
> **"Current Zoom Level" Definition**: Tasks created via `[+]` or Append Row use the `viewPath` head as their parent.
> - At root (`viewPath = []`): New tasks are **root-level** (`parentId = null`), NOT Inbox children.
> - Drilled into "Groceries" (`viewPath = ["groceries-id"]`): New tasks become children of "Groceries".

- [ ] **NEW — Component**: `PlanViewContainer` (Mobile Layout)
  - **Bottom Bar**: Fixed at bottom of the **Plan View** (not the modal). Entirely new UI — does not exist yet.
    - `[Hamburger]` (Left) - Menu/Settings. **Visible but disabled** (placeholder for future feature).
    - `[<]` (Center-Left) - "Up Level" / Pop View. Wires to existing `navigateUp()`. Disabled at root.
    - `[+]` (Center) - Adds Item to **Top** of current zoom level (uses `position: 'start'`).
    - **(Right slot reserved/empty)**.
  - **Behavior**:
    - When user taps `[+]`, it opens the **Task Editor Modal** in Create Mode with `position: 'start'`.

- **EXISTING (Step 5)** — Row Layout: `[Checkbox] [Title] ... [Drill Arrow >]`
  - Expansion chevrons hidden.
  - Drill-down arrow on far right.
  - Tapping the row/title opens the **Task Editor Modal** (already wired in `TaskOutlineItem`).

- [ ] **NEW — Append Row**: `[ + ]` (Icon)
  - Located at the very bottom of the list. **Visible on both Desktop and Mobile.**
  - **Scrolls with content** (not floating).
  - Tap -> Opens **Task Editor Modal** to create new item at **Bottom** of current zoom level (uses `position: 'end'`).

### Sub-Step 6D: Modal Hierarchy Controls

**Goal**: Structural editing within the modal experience (replaces keyboard ribbon).

> [!NOTE]
> The original "keyboard ribbon" pattern was designed for inline outline editing (à la Workflowy).
> Since this app uses a modal for task editing, the ribbon is replaced with inline modal content.

- [ ] **Component**: `TaskEditorModal` — Add "Hierarchy" section
  - **Layout** (within modal content, **Edit Mode Only**):
    ```
    ── Hierarchy ──────────────
    Parent: "Groceries"
    [← Outdent]  [Indent →]
    [Move...]
    ```
  - **Elements**:
    - **Parent Display**: Read-only label showing current parent name (or "Root" if top-level). Label updates immediately to reflect the new parent name when moved.
    - **Outdent Button** (`← Outdent`): Moves task up one level (becomes sibling of current parent). Disabled if already at root.
      - **Edge Case (Mobile)**: If task is a direct child of the current zoom context, outdenting would move it out of view. In this case, the system **MUST auto-navigate up one level** (`navigateUp()`) so the moved task remains visible.
    - **Indent Button** (`Indent →`): Moves task to become child of previous sibling. Disabled if no previous sibling exists.
    - **Move Button**: Opens `MovePickerModal` for precise reparenting (existing Step 7 feature).
  - **Logic**: Wire up Indent/Outdent to `useTaskIntents`. **Immediate Action**: Moves are applied immediately to the database (not deferred to Save).

### Sub-Step 6E: "Highlight & Reveal" (Navigation Feedback)

**Goal**: Prevent "Where did my task go?" confusion.

- [ ] **State**: Track `lastCreatedTaskId`.
- [ ] **Logic**:
  - **Auto-Expand**: Parent of new task must expand (if collapsed).
  - **Auto-Scroll**: Scroll Plan View to new item after modal close. **Always** scroll to the new task, whether inserted at top or bottom.
  - **Visual Cue**: Yellow fade-flash on the new row (brief animation, ~500ms).

### User Flows (Storyboard)

In all flows, the user is in Plan mode (the outline).

**1. Desktop: Add Sibling**
- **User**: Hovers over "Buy Milk".
- **UI**: Bullet becomes `•••`.
- **User**: Clicks `•••` -> Selects "Add Sibling".
- **UI**: Opens Modal (Create Mode). Parent = "Groceries" (same as "Buy Milk").
- **User**: Types "Buy Eggs" -> Returns.
- **UI**: Modal closes. "Buy Eggs" appears **immediately after** "Buy Milk" (not at end of list). Row flashes yellow.

**2. Desktop: Add Child**
- **User**: Hovers over "Smart Fats".
- **UI**: Bullet becomes `•••`.
- **User**: Clicks `•••` -> Selects "Add Child".
- **UI**: Opens Modal (Create Mode). Parent = "Smart Fats".
- **User**: Types "Avocado" -> Returns.
- **UI**: Modal closes. "Smart Fats" expands (if closed). "Avocado" appears **at the end** of children. Row flashes yellow.

**3. Mobile: Add to Bottom (Append Row)**
- **User**: Scrolls to bottom of list.
- **User**: Taps `[ + ]` row.
- **UI**: Opens Modal (Create Mode). Parent = Current Zoom View.
- **System**: Keyboard slides up.
- **User**: Types "Buy Eggs" -> Save.
- **UI**: Modal closes. "Buy Eggs" appears at the **bottom** of the list (above the `[ + ]` row). Flashes yellow.

**4. Mobile: Add to Top (Bottom Toolbar)**
- **User**: Taps `+` on the Bottom Toolbar.
- **UI**: Opens Modal (Create Mode). Parent = Current Zoom View.
- **System**: Keyboard slides up.
- **User**: Types "Buy Milk" -> Save.
- **UI**: Modal closes. "Buy Milk" appears at the **top** of the list. Flashes yellow. 

**5. Mobile: Edit & Indent**
- **User**: Taps "Buy Eggs" title.
- **UI**: Opens Modal (Edit Mode). Hierarchy section visible.
- **User**: Taps `[Indent →]` in Hierarchy section.
- **UI**: Task becomes child of previous sibling ("Buy Milk").
- **User**: Taps "Save".
- **UI**: Modal closes. View updates to show new structure.

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

## Step 7: MovePickerModal & Reparenting

_Allow users to reorganize tasks by changing parent and position._

**PRD Reference**: [§4.5 Task Editing](../design/prd.md) — "Move... Button: Opens a picker modal allowing the user to: Select a new parent (reparenting). **NOTE**: Detailed reordering within siblings is deferred for this step (User Decision: Option A)."

### Tasks

- [ ] **Create `MovePickerModal` Component**
  - **Does not exist yet** — Create in `components/modals`
  - Modal displays a tree/list of potential parent tasks
  - **Exclusion filter**: Must exclude the task being moved AND all its descendants (prevents circular references)
  - **Interaction**: Selecting a parent moves the task to the **end** of that parent's child list
  - **Deferred**: Precise reordering among siblings is deferred to a future update
  - Loading state while calculating valid targets

- [ ] **Implement `useValidParentTargets` Hook**
  - Input: `docUrl`, `taskIdBeingMoved`
  - Output: List of valid parent tasks (all tasks minus self and descendants)
  - Memoized computation to avoid recalculating on every render

- [ ] **Utilize Existing `ops.moveTask`**
  - Logic already exists in `packages/tasklens/src/persistence/ops.ts`
  - Supports `newParentId` (undefined = root) and `afterTaskId`
  - Includes cycle detection and depth validation
  - **Task**: Expose this via `useTaskIntents` (add `moveTask` to the interface)

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

## Step 8: Task Editor Polish (Missing Features)

_Implement features erroneously marked as complete in Phase 4._

### Tasks

- [ ] **"Find in Plan" Button**
  - **Logic**: Closes modal, resets `viewPath` to the task's location, expands necessary parents.
  - **UI**: Added to "Navigation & Hierarchy" section of modal.
  - **Interaction**: `onFindInPlan` callback prop.

- [ ] **Repetition Selector**
  - **Logic**: Maps to `task.repeatConfig`.
  - **UI**: Dropdown or Segmented Control (None | Routinely | Calendar).
  - **Validation**: Ensure `dueDate` is set if repetition is enabled.

- [ ] **Notes Field**
  - **Logic**: Wire up to `task.notes`.
  - **UI**: Enable the currently disabled Textarea.

### Verification

- [ ] E2E: Create repeating task -> Verify config persisted.
- [ ] E2E: "Find in Plan" -> Navigates to correct tree location.
- [ ] Unit tests for recurrence logic edge cases.

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

## Step 9: Deferred E2E Verification (from Phase 4)

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
| `viewmodel/ui/useNavigationState.ts` | MODIFY | Step 6 (Create Mode state) |
| `components/modals/TaskEditorModal.tsx` | MODIFY | Step 6 (Create Mode support) |
| `viewmodel/containers/TaskEditorContainer.tsx` | MODIFY | Step 6 (Create Mode handling) |
| `components/views/plan/PlanViewContainer.tsx` | MODIFY | Step 6 (Bottom Bar, Append Row) |
| `components/primitives/TaskOutlineItem.tsx` | MODIFY | Step 6 (Hover Menu) |
| `components/modals/MovePickerModal.tsx` | CREATE | Step 7 |
| `viewmodel/containers/MovePickerContainer.tsx` | CREATE | Step 7 |
| `viewmodel/projections/useValidParentTargets.ts` | CREATE | Step 7 |
| `e2e/plan.spec.ts` | MODIFY | Steps 6, 7, 9 |


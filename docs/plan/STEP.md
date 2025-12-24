# Step 6: Critical Create Flow Fixes (Workflowy Polish) 🔴 CURRENT PRIORITY

_Implementing "Workflowy-like" interactions for seamless structure management._

## 🐛 Bugs (Next To Complete)

- [ ] **BUG: Title Input Not Focused**: When Task Editor Modal opens (Create or Edit mode), the Title input should be auto-focused, at least on desktop. Currently requires manual click.
- [ ] **BUG: Child Not Visible After Creation**: When adding a child via "Add Child" (hover menu or modal), the parent is not auto-expanded, so the child is hidden. User must manually expand. **Cross-ref**: This is addressed by **Sub-Step 6E: "Highlight & Reveal"** (Auto-Expand logic).
- [x] **BUG: CSS Specificity (Hover Menu)**: Fixed. The selector was incorrectly applying `opacity: 1` universally. Changed to `.task-row:hover .task-menu-trigger`.
- [ ] **PERF: Memoize TaskOutlineItem**: `OutlineTree` creates arrow functions in every render. Apply `React.memo` to `TaskOutlineItem` to avoid unnecessary re-renders.

## 🧪 Missing Tests

- [ ] **TEST: Mobile E2E Gap**: Step 5 was marked Done, but the E2E test for Mobile Drill-Down Journey is missing. Add verification for this.

### Sub-Step 6A: Modal Create Mode (Prerequisite)

**Goal**: Enable the Task Editor modal to create new tasks, not just edit existing ones.

> [!IMPORTANT]
> This is a prerequisite for Sub-Steps 6B and 6C. Both desktop and mobile create flows depend on "Create Mode."

> [!WARNING] > **API Gap**: Current `createTask(state, props)` always inserts at **top** (`unshift`). The spec requires:
>
> - "Add Sibling" → insert **immediately after** reference task
> - "Add Child" → insert at **end** of children
>
> The `createTask` API must be extended with an optional `afterTaskId` parameter (similar to `moveTask`).

- [x] **API**: Extend `ops.createTask` to support flexible positioning
  - `createTask(state, props, options?)` where `options` is one of:
    - `{position: 'start'}` — prepend to beginning of children (for Bottom Bar `[+]`)
    - `{position: 'end'}` — append to end of children (for Append Row, Add Child)
    - `{position: 'after', afterTaskId: TaskID}` — insert after specified task (for Add Sibling)
  - Defaults to `position: 'end'` if not specified.
  - Update `@mydoo/tasklens` with this change.

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: Full quality gate sequence passed in background command `dcd1c1c5-979a-4ee9-9279-8775e7ebd64a`. Unit tests for `createTask` positioning added and passing in `packages/tasklens/tests/unit/createTaskDefaults.test.ts`. Existing client tests fixed to handle new default positioning.

- [x] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [x] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run.
- [x] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`).
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [x] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

- [x] **State**: Add to `useNavigationState`
  - `createTaskParentId: TaskID | null` (Refactored to `modal: ModalState`)
  - `createTaskAfterTaskId: TaskID | undefined` (Refactored to `modal: ModalState`)
  - `editingTaskId` (Refactored to `modal: ModalState`)
  - Refactored all modal state into a single `modal` object.
  - Consistently use `undefined` for optional IDs (`parentId`, `afterTaskId`) per preference.

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass (ALL repo tests)
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: Full quality gate sequence passed in background command `8fb6be2e-b503-40ad-bfcd-db995e2e5617`. Refactored `useNavigationState` and its usages to use a unified `ModalState` object with `undefined` for optional properties.

**Completion**

- [x] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [x] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [x] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [x] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

- [x] **Component**: `TaskEditorModal` — Support Create Mode
  - [x] Form fields are empty (default values).
  - [x] "Parent" display shows target parent title.
  - [x] Hide hierarchy/footer actions in Create Mode.
- [x] **Container**: `TaskEditorContainer` — Handle both modes
- [x] **Tests**: Verified with full quality gate pass.

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: Full quality gate pass in `b504f8eb-71e9-4cb2-bc83-5e35df56ebc6`. Create Mode UI, Container logic, and intent signature updates verified.

**Completion**

- [x] ✅ **CLEAN LISTS**: All 6A items marked as complete.
- [x] 🛑 **TRUST BUT VERIFY**: Verified with full build/test pass.
- [x] 🛑 **RESTART ON EDIT**: Completed final pass after all fixes.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [x] 💾 **COMMIT GATE**: Ready for final commit of component changes.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output.

### Sub-Step 6B: Desktop Interaction (Hover Menu)

**Goal**: Controls appear on the left. The "Menu/Bullet" is the anchor.

> [!NOTE]
> This layout change applies to **desktop (tree mode) only**. Mobile retains the existing `[Checkbox] [Title] [Drill Arrow]` layout from Step 5.

- [x] **Component**: `TaskOutlineItem` (Desktop Variant)
  - [x] Layout: `[Menu] [ChevronContainer] [Checkbox] [Title]`
  - [x] Interaction: Hover reveals menu trigger (`ActionIcon` with `IconDots`).
  - [x] Actions: Menu includes Add Sibling, Add Child, Delete.
  - [x] Integration: Props wired through `PlanViewContainer` -> `OutlineTree`.

**Quality Gates**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass
- [x] `pnpm test:e2e` -> Pass
- [x] **EVIDENCE**: Full quality gate pass in `c9cbe829-227c-4ac2-ad9f-1d8e0ad3f5f2`. Components updated smoothly, tests passed.

**Completion**

- [x] ✅ **CLEAN LISTS**: Checked.
- [x] 🛑 **TRUST BUT VERIFY**: Verified.
- [x] 🛑 **RESTART ON EDIT**: No edits after final verification.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [x] 💾 **COMMIT GATE**: Ready for commit.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Pending.

- [ ] **Tests**:
  - [ ] Component: `TaskOutlineItem` - Verify Hover Menu appears on hover (desktop).
  - [ ] Component: `TaskOutlineItem` - Verify Menu clicks trigger correct events.

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

### Sub-Step 6C: Mobile Interaction (Bottom Bar & Drill-Down)

**Goal**: Persistent navigation and creation controls within the current zoom context.

> [!NOTE]
> The **Row Layout** behavior below is already implemented in Step 5. It is documented here for context only.
> New work in this step: **Bottom Bar** (entirely new component) and **Append Row**.

> [!IMPORTANT] > **"Current Zoom Level" Definition**: Tasks created via `[+]` or Append Row use the `viewPath` head as their parent.
>
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

- **EXISTING (Step 5)** — Row Layout: `[Checkbox] [Title] ... [Drill Arrow >]`

  - Expansion chevrons hidden.
  - Drill-down arrow on far right.
  - Tapping the row/title opens the **Task Editor Modal** (already wired in `TaskOutlineItem`).

- [ ] **NEW — Append Row**: `[ + ]` (Icon)

  - Located at the very bottom of the list. **Visible on both Desktop and Mobile.**
  - **Scrolls with content** (not floating).
  - Tap -> Opens **Task Editor Modal** to create new item at **Bottom** of current zoom level (uses `position: 'end'`).

- [ ] **Tests**:
  - [ ] Component: `PlanViewContainer` - Verify Bottom Bar renders on mobile.
  - [ ] Component: `PlanViewContainer` - Verify Append Row renders at bottom.
  - [ ] E2E: Mobile - Tap `[+]` (Bottom Bar) opens modal -> Save -> Task at top.
  - [ ] E2E: Mobile - Tap `[+]` (Append Row) opens modal -> Save -> Task at bottom.

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

- [ ] **Tests**:
  - [ ] Component: `TaskEditorModal` - Verify Hierarchy section appears in Edit Mode.
  - [ ] Unit: `useTaskIntents` - Indent/Outdent logic (parent/sibling resolution).
  - [ ] Unit: Single-child Outdent Edge Case (Mobile) -> Triggers `navigateUp()`.

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

### Sub-Step 6E: "Highlight & Reveal" (Navigation Feedback)

**Goal**: Prevent "Where did my task go?" confusion.

- [ ] **State**: Track `lastCreatedTaskId`.
- [ ] **Logic**:

  - **Auto-Expand**: Parent of new task must expand (if collapsed).
  - **Auto-Scroll**: Scroll Plan View to new item after modal close. **Always** scroll to the new task, whether inserted at top or bottom.
  - **Visual Cue**: Yellow fade-flash on the new row (brief animation, ~500ms).

- [ ] **Tests**:
  - [ ] E2E: Create Sibling -> Verify parent expands + auto-scroll to new task.
  - [ ] E2E: Verify yellow flash class is applied to new item.

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

### Sub-Step 6F: Regression Coverage (Full Journey)

**Goal**: Verify the complete user flow from an empty state to a populated hierarchy on both Desktop and Mobile.

- [ ] **Scenario**: Desktop Full Journey
  - Start Empty -> "Add First Task" button appears.
  - Create Root Task -> Verify visible.
  - Rename Root Task -> Verify title update persistence.
  - Add Sibling (Hover Menu) -> Verify position.
  - Add Child (Hover Menu) -> Verify hierarchy.

- [ ] **Scenario**: Mobile Full Journey
  - Start Empty -> "Add First Task" button appears.
  - Create Root Task -> Verify visible.
  - Rename Root Task (Tap Title -> Modal) -> Verify persistence.
  - Add Sibling (Footer Action) -> Verify position.
  - Add Child (Footer Action) -> Verify Drill-Down visibility (Child hidden at root, visible after drill).

**Quality Gates**

- [ ] `pnpm test:e2e` -> Pass (`full-journey.spec.ts`)

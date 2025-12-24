# Step 6: Critical Create Flow Fixes (Workflowy Polish) 🔴 CURRENT PRIORITY

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


- [ ] **State**: Add to `useNavigationState`
  - `createTaskParentId: TaskID | null` — when set, modal opens in **Create Mode**.
  - `createTaskAfterTaskId: TaskID | undefined` — for "Add Sibling", the reference task to insert after.
  - `editingTaskId` for Edit Mode, `createTaskParentId` for Create Mode — mutually exclusive.
  - `openCreateModal(parentId: TaskID | null, afterTaskId?: TaskID)` — opens Create Mode.

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


- [ ] **Component**: `TaskEditorModal` — Support Create Mode
  - **When in Create Mode**:
    - Form fields are empty (default values).
    - "Parent" display shows the target parent's title (or "Root").
    - **Hierarchy Controls** (Indent/Outdent/Move) are **HIDDEN**.
    - Save button creates new task via `createTask(title, parentId, afterTaskId)`.
  - **When in Edit Mode**: Existing behavior (loads task, saves updates).

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


- [ ] **Container**: `TaskEditorContainer` — Handle both modes
  - Determine mode from navigation state.
  - Provide appropriate save handler based on mode.

- [ ] **Tests**:
  - [ ] Unit: `ops.createTask` - Verify `position: 'start'`, `'end'`, and `'after'` (logic & persistence).
  - [ ] Unit: `useNavigationState` - Verify `openCreateModal` sets correct state.
  - [ ] Component: `TaskEditorModal` - Verify "Create Mode" renders correctly (empty form, no hierarchy controls).


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

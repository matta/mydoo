# Step 7: MovePickerModal & Reparenting

_Allow users to reorganize tasks by changing parent and position._

**PRD Reference**: [§4.5 Task Editing](../design/prd.md) — "Move... Button: Opens a picker modal allowing the user to: Select a new parent (reparenting)."

> [!NOTE]
> Detailed reordering within siblings is deferred. This step focuses on selecting a new parent (which appends to the end of that parent's children).

## Spec Draft (Stop & Plan)

> [!IMPORTANT] > **Logic Rule**: When moving a task, you cannot move it _inside itself_ or _inside any of its descendants_. This would create a cycle or detach the subtree.

### Test Spec Draft (TypeScript)

```typescript
test("MovePicker Flow", async ({ plan }) => {
  await test.step("Setup Hierarchy", async () => {
    // A -> B -> C
    await plan.createTask("Task A");
    await plan.selectTask("Task A");
    await plan.addChild("Task B");
    await plan.selectTask("Task B");
    await plan.addChild("Task C");
  });

  await test.step("Open Move Picker", async () => {
    // Open editor for 'Task A'
    await plan.openTaskEditor("Task A");
    await plan.clickMoveButton();
    // Expect Move Picker to be visible
    await expect(page.getByRole("dialog", { name: "Move Task" })).toBeVisible();
  });

  await test.step("Verify Invalid Targets Excluded", async () => {
    // Should verify that 'Task A' (self) and 'Task B' (child) are NOT selectable
    // This might be via visual check or ensuring they don't appear in the picker list
    const picker = page.getByRole("dialog", { name: "Move Task" });
    await expect(picker.getByText("Task A")).toBeDisabled(); // or NotVisible
    await expect(picker.getByText("Task B")).toBeDisabled();
  });

  await test.step("Move to new Root", async () => {
    // If we had another root task, we could pick it.
    // For now, let's create a sibling 'Task D' and move 'Task B' to it.
  });
});
```

## Tasks

### 1. State & Logic

- [x] **State**: Update `useNavigationState`

  - Extend `ModalState` to support `{ type: 'move', taskId: TaskID }`.
  - Add `openMoveModal(taskId: TaskID)` and handling.

- [x] **Logic**: Create `viewmodel/projections/useValidParentTargets.ts`
  - **Input**: `doc`, `taskIdBeingMoved`
  - **Output**: List of potential parent tasks (and "Root").
  - **Filter**:
    - Exclude `taskIdBeingMoved`.
    - Exclude all descendants of `taskIdBeingMoved`.
  - **Structure**: Tree-style structure for picker UI.

### 2. UI Components

- [x] **Component**: `MovePickerModal`

  - **Props**: `taskId`, `onClose`.
  - **UI**:
    - Header: "Move [Task Title]"
    - Content: Scrollable list of valid parents in tree style.
    - **Action**: Clicking a target calls `ops.moveTask`.
    - **Post-Action**: Close modals and refresh view.

- [x] **Integration**: `TaskEditorContainer` / `TaskEditorModal`
  - Add "Move..." button to Hierarchy section.
  - `NavigationState` handles modal transitions.

### 3. Container

- [x] **Container**: `MovePickerContainer`
  - Connects `MovePickerModal` to data.
  - Uses `useValidParentTargets`.
  - Handles `onSelect` -> `transact(ops.moveTask)`.

## Quality Gates

- [x] `pnpm fix && pnpm pre-commit` -> Pass
- [x] **EVIDENCE**: Verified via terminal output and git commit hooks.

## Status

### ✅ Complete

- Core implementation (all components, hooks, containers)
- Integration (button, mounting, wiring)
- Code quality (kebab-case, linting, type-checking)
- All unit tests passing (71/71)
- All E2E tests passing (21/21)
- Static analysis passing (`fix`, `lint`, `lint:filenames`, `typecheck`)
- Resolved modal race condition and fixture stability issues.

### ⚠️ Incomplete

- (None)

## Completion

- [x] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [x] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [x] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix && pnpm pre-commit`). They must all pass in sequence against the same repository state.
- [x] 🛑 STOP and prompt for user review with the EVIDENCE.
- [x] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [x] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

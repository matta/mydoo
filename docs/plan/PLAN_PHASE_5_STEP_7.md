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

- [ ] **State**: Update `useNavigationState`

  - Extend `ModalState` to support `{ type: 'move', taskId: TaskID }`.
  - Add `openMoveModal(taskId: TaskID)` and handling.

- [ ] **Logic**: Create `viewmodel/projections/useValidParentTargets.ts`
  - **Input**: `doc`, `taskIdBeingMoved`
  - **Output**: List of potential parent tasks (and "Root").
  - **Filter**:
    - Exclude `taskIdBeingMoved`.
    - Exclude all descendants of `taskIdBeingMoved`.
    - (Optionally verify against `validateNoCycle` in ops, but UI should filter proactively).
  - **Structure**: Should probably return a flat list or a tree? A tree is better for the picker UI.
    - Maybe `getTaskTree` but filtered?

### 2. UI Components

- [ ] **Component**: `MovePickerModal`

  - **Props**: `taskId`, `onClose`.
  - **UI**:
    - Header: "Move [Task Title]"
    - Content: Scrollable list of valid parents.
      - "Root" (top level).
      - Other tasks (indented or breadcrumb style? Tree style is best).
    - **Action**: Clicking a target calls `ops.moveTask(taskId, newParentId, undefined)`.
    - **Post-Action**: Close MoveModal, Close EditorModal (or return to it?), Refresh View.

- [ ] **Integration**: `TaskEditorContainer` / `TaskEditorModal`
  - Add "Move..." button to Hierarchy section.
  - Clicking it closes Editor and opens MovePicker (or stacks them? Stacking modals is tricky).
  - Better: `NavigationState` handles switching `modal` state from `'edit'` to `'move'`.

### 3. Container

- [ ] **Container**: `MovePickerContainer`
  - Connects `MovePickerModal` to data.
  - Uses `useValidParentTargets`.
  - Handles `onSelect` -> `transact(ops.moveTask)`.

## Quality Gates

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass (ALL repo tests)
- [ ] `pnpm test:e2e` -> Pass
- [ ] **EVIDENCE**: Show terminal output of passing tests.

## Status

### ✅ Complete

- Core implementation (all components, hooks, containers)
- Integration (button, mounting, wiring)
- Code quality (kebab-case, linting, type-checking)
- All unit tests passing (71/71)
- All existing E2E tests passing (19/19)
- Static analysis passing (`fix`, `lint`, `lint:filenames`, `typecheck`)

### ⚠️ Incomplete

- **E2E Test Coverage for Move Picker Flow**
  - Test scaffolded in `move-picker.spec.ts` but currently **skipped**
  - Fixtures created/extended in `fixtures.ts`
  - **Current**: Desktop tree mode only
  - **Recommendation**: Split into two test cases
    1. **Mobile test**: Drill-down navigation (unblocked - can implement now)
    2. **Desktop test**: Tree expansion navigation (blocked on expansion state fix)
  - **Desktop blocker**: Tree expansion state doesn't persist across modal interactions
  - **Impact**: Cannot verify end-to-end move flow automatically for either platform yet
  - **See**: `move-picker.spec.ts` TODO and `ROLLING_CONTEXT.md`
  - **Options for desktop**: Fix UI expansion persistence (preferred) or update fixtures (workaround)

## Completion

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

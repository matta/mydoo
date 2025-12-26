# Step 8: Task Editor: Find in Plan

_Allow users to jump to the task's location in the tree from the modal._

**PRD Reference**: [§4.5 Task Editing](../design/prd.md)

## Tasks

### 1. "Find in Plan" Button

- [x] **Goal**: Allow users to jump to the task's location in the tree from the modal.
- [x] **Logic**:
  - Closes the modal.
  - Resets `viewPath` to the task's ancestors (expanding as needed).
  - Scrolls to the task.
- [x] **UI**:
  - Add button to "Navigation & Hierarchy" section (near Indent/Outdent/Move).
  - Label: "Find in Plan" or icon (e.g., crosshairs/target).
- [x] **Integration**:
  - Use `useNavigationState` to manipulate `viewPath` and `expandedIds`.

## Verification

### Automated

- [x] **E2E**: "Find in Plan" -> Navigates to correct tree location.

### Manual

- [x] Verify "Find in Plan" ensures visibility (expands parents).

## Quality Gates

- [x] `pnpm fix && pnpm pre-commit` -> Pass
- [x] **EVIDENCE**: Show terminal output of passing tests.

## Completion

- [ ] ✅ **CLEAN LISTS**: **MUST** clean up all TODO lists and plans before stopping and asking for human review.
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix && pnpm pre-commit`). They must all pass in sequence against the same repository state.
- [ ] 🛑 STOP and prompt for user review with the EVIDENCE.
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response (e.g., "yes", "lgtm", "go ahead") is NOT sufficient.
- [ ] 🛑 **VERIFY COMMIT SUCCESS**: Check terminal output and exit code of `git commit`.

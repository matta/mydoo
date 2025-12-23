# Implementation Plan: Phase 5 (The Plan View)

**Goal**: Hierarchical tree navigation with expand/collapse and breadcrumbs.

**PRD References**: [§4.3 The Plan View](../design/prd.md)

---

## Step 1: Logic & State

_Implement the core tree projection and navigation state._

- [x] Implement `useNavigationState` (manages `expandedIds` set, `viewPath` stack)
- [x] Implement `useTaskTree` projection:
  - Recursive tree builder
  - Sorting (Priority desc, then DFS index)
  - Filtering based on `expandedIds` (for flat lists) or providing nested structure
- [x] Unit tests for tree projection

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass
- [x] `pnpm test:e2e` -> Pass
- [x] ✅ Update this doc
- [x] 🛑 **TRUST BUT VERIFY**: Tested locally:
  - `pnpm test`: 41 passed (reflects removal of 2 deleted-task tests)
  - `pnpm test:e2e`: 6 passed, 1 skipped
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [x] 🛑 **STOP FOR HUMAN APPROVAL** before commit
- [x] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response is NOT sufficient.

---

### Step 2: Tree UI Components
- [x] Create `TaskOutlineItem` component (row renderer).
- [x] Create `OutlineTree` component (recursive list).
- [x] Update `PlanViewContainer` to render the tree.
- [x] **Verification**:
  - [x] `pnpm fix` passed.
  - [x] `pnpm build` passed.
  - [x] `pnpm test` passed (new component tests + `tasklens` filtering logic).
  - [x] `pnpm test:e2e` passed (regression check).
  - [x] Removed bogus `nextTaskId`/`nextPlaceId` counters (using UUIDs).

### Step 3: Interaction & Layout
- [ ] Implement drag-and-drop (if using `dnd-kit` or similar) OR basic move actions.
  - *Decision*: For MVP, stick to "Move Up/Down" buttons or basic keyboard shortcuts?
  - *Refinement*: Let's ensure basic "Indent/Outdent" via keyboard (Tab/Shift+Tab) works.
- [ ] Connect `TaskOutlineItem` actions (Drill down, Toggle completion).
- [ ] Add "Breadcrumbs" for navigation history in `PlanViewContainer`.
- [ ] **Verification**:
  - [ ] Keyboard navigation tests.
  - [ ] E2E test for tree manipulation.

**Quality Gates (Step 2)**
- [x] `pnpm fix`
- [x] `pnpm build`
- [x] `pnpm test`
- [x] `pnpm test:e2e`
- [x] `git commit` (User approval required). They must all pass in sequence against the same repository state.
- [x] 🛑 **STOP FOR HUMAN APPROVAL** before commit
- [x] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response is NOT sufficient.

---

## Step 3: Navigation & Mobile Support

_Handle deep navigation and breadcrumbs._

- [ ] Implement `useBreadcrumbs` (path from root to current view)
- [ ] Implement Mobile drill-down interaction (Tap parent -> push to stack)
- [ ] UI: Breadcrumb bar integration

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass
- [ ] `pnpm test:e2e` -> Pass
- [ ] ✅ Update this doc
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 **STOP FOR HUMAN APPROVAL** before commit
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response is NOT sufficient.

---

## Step 4: Reparenting & Validation

_Move tasks and verify everything (including deferred items)._

- [ ] Create `MovePickerModal` (Select new parent)
- [ ] Implement `moveTask` op (update `parentId`)
- [ ] **Verification Catch-up**:
  - [ ] E2E: Edit task properties (Title/Importance) _(Deferred from Phase 4)_
  - [ ] E2E: Delete task + Cascade _(Deferred from Phase 4)_
  - [ ] E2E: Drag/Move task -> verify tree update

**Quality Gates**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm build` -> Pass
- [ ] `pnpm test` -> Pass
- [ ] `pnpm test:e2e` -> Pass
- [ ] ✅ Update this doc
- [ ] 🛑 **TRUST BUT VERIFY**: You **MUST NOT** check any of the above boxes until the corresponding command has actually been run. **CRITICAL**: Do not assume success of one command based on the success of another (e.g., a passing `test` run does NOT guarantee a clean `lint` check).
- [ ] 🛑 **RESTART ON EDIT**: If you make ANY code changes to fix a failure in any quality gate, you **MUST** uncheck ALL boxes and restart verification from the very first gate (`pnpm fix`). They must all pass in sequence against the same repository state.
- [ ] 🛑 **STOP FOR HUMAN APPROVAL** before commit
- [ ] 💾 **COMMIT GATE**: You **MUST NOT** run `git commit` until the user responds with the single word **"commit"**. Any other response is NOT sufficient.

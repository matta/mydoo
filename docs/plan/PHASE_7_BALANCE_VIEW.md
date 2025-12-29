# Phase 7: The Balance View

**Goal**: Implement the "Life Balance" visualization to allow users to adjust target effort (`desiredCredits`) for Top-Level Goals and see their actual effort distribution.

**Status**: [ ] Not Started

## 1. Prerequisites & Type Rationalization

The "Balance" algorithm relies on `effectiveCredits` (decayed effort) and `feedbackFactor`. These are currently calculated in the domain logic but strictly "internal" to `EnrichedTask` and not exposed on `ComputedTask`. To visualize "Actual %" vs "Target %", we must expose these computed metrics to the View Layer.

Additionally, `ROOT_INBOX_ID` is referenced in designs but missing from the implementation.

### Steps

- [ ] **Step 1.1: Expose Computed Metrics**
  - [ ] **[MODIFY]** `packages/tasklens/src/types.ts`:
    - [ ] Add `export const ROOT_INBOX_ID = 'root:inbox' as TaskID;`
    - [ ] Update `ComputedTask` to include readonly properties:
      - `effectiveCredits: number` (for Actual % calculation)
      - `desiredCredits: number` (already in PersistedTask, but ensure visibility)
  - [ ] **Quality Gates**:
    - [ ] `pnpm fix && pnpm pre-commit` -> Pass
    - [ ] **EVIDENCE**: passing tests.
    - [ ] **STOP & WAIT** for "commit".

## 2. Balance Logic (Client Hook)

We need a bespoke hook to calculate the "Pie Chart" percentages.

**Plain English Explanation**:
Currently, our logic lives in the database layer. The UI needs a way to "read" the current state of tasks from our local "file cabinet" (Redux Store) and calculate two numbers for every Top Level Goal:

1. **Target**: How much effort you _want_ to spend (User set).
2. **Actual**: How much effort you _actually_ spent (Calculated from history).

This hook acts as the translator, grabbing the raw task data and doing the math so the UI can just draw the bars.

### Steps

- [ ] **Step 2.1: Implement `useTaskEntities` Hook**

  - [ ] **[NEW]** `packages/tasklens/src/react/hooks/use-task-entities.ts`:
    - [ ] Create simple wrapper around `useSelector(selectTaskEntities)`.
    - [ ] Export from index.

- [ ] **Step 2.2: Implement `useBalanceData` Hook**
  - [ ] **[NEW]** `apps/client/src/hooks/use-balance-data.ts`:
    - [ ] Import `useTaskEntities` (Redux access).
    - [ ] Logic:
      - Get all tasks where `parentId` is `undefined` (Roots).
      - **Exclude** `ROOT_INBOX_ID` (we will ensure this ID is used for the Inbox).
      - Calculate `totalDesiredCredits`.
      - Calculate `totalEffectiveCredits`.
      - Return normalized list.
  - [ ] **Quality Gates**:
    - [ ] `pnpm fix && pnpm pre-commit` -> Pass
    - [ ] **EVIDENCE**: passing tests.
    - [ ] **STOP & WAIT** for "commit".

## 3. Balance UI Components

### Steps

- [ ] **Step 3.1: Balance Item Component**

  - [ ] **[NEW]** `apps/client/src/components/views/balance/balance-item.tsx`:
    - [ ] Props: `item`, `onChangeDesiredCredits(id, val)`.
    - [ ] UI:
      - Title.
      - Slider for `desiredCredits` (e.g., 0-10, default 1?).
      - Progress Bar for `actualPercent` (vs `targetPercent`).
      - Visual "Starving" indicator if `actual << target`.
  - [ ] **Quality Gates**:
    - [ ] `pnpm fix && pnpm pre-commit` -> Pass
    - [ ] **EVIDENCE**: passing tests.
    - [ ] **STOP & WAIT** for "commit".

- [ ] **Step 3.2: Balance View Container**
  - [ ] **[NEW]** `apps/client/src/components/views/balance/balance-view-container.tsx`:
    - [ ] Use `useBalanceData`.
    - [ ] Use `useTaskActions` to dispatch updates.
    - [ ] Render List of `BalanceItem`s.
  - [ ] **Quality Gates**:
    - [ ] `pnpm fix && pnpm pre-commit` -> Pass
    - [ ] **EVIDENCE**: passing tests.
    - [ ] **STOP & WAIT** for "commit".

## 4. Integration

### Steps

- [ ] **Step 4.1: Add to Navigation**
  - [ ] **[MODIFY]** `apps/client/src/viewmodel/ui/use-navigation-state.ts`:
    - [ ] Add `'balance'` to `ActiveTab`.
  - [ ] **[MODIFY]** `apps/client/src/components/shell/app-shell-container.tsx`:
    - [ ] Add "Balance" Button to Desktop Sidebar.
    - [ ] Add "Balance" Button to Mobile Bottom Bar.
    - [ ] Render `<BalanceViewContainer />` when active.
  - [ ] **Quality Gates**:
    - [ ] `pnpm fix && pnpm pre-commit` -> Pass
    - [ ] **EVIDENCE**: passing tests and **Screenshot/Recording** of the new view.
    - [ ] **STOP & WAIT** for "commit".

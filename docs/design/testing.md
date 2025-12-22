# Testing Strategy

This document defines the testing strategy for `mydoo`, ensuring robust progression and safe AI-led iteration.

## The Testing Pyramid

We adhere to a standard 3-layer testing pyramid tailored for our Local-First PWA architecture:

| Layer              | Tool             | Scope                                | Speed         | Responsibility                                                                |
| ------------------ | ---------------- | ------------------------------------ | ------------- | ----------------------------------------------------------------------------- |
| **1. Unit Logic**  | `vitest`         | Domain functions, pure algorithms    | ⚡️ Fast (<1s) | Validate priority scoring, healer logic, invariant enforcement.               |
| **2. Integration** | `vitest` + `RTL` | React Hooks, View Models, Containers | 🚀 Medium     | Verify UI logic _without_ a browser. Mock Automerge storage if needed.        |
| **3. End-to-End**  | `Playwright`     | Full App in Browser                  | 🐢 Slow       | Verify critical user flows (Create, Sync, Offline) in a real browser context. |

---

## 1. Unit Logic Tests (`@mydoo/tasklens`)

These tests already exist and should be expanded as domain logic grows. They test the "Brain" of the application.

**Location:** `packages/tasklens/tests/**`
**Runner:** `vitest`

### What to Test:

- **Priority Algorithm**: Does the scoring math work? (Fixture-based tests)
- **Healer**: Does it correctly deduplicate arrays?
- **Schemas**: Does Zod validate/reject data correctly?

**How to Run:**

```bash
pnpm --filter @mydoo/tasklens test
```

---

## 2. Integration & Component Tests (`apps/client`)

These tests verify the "View Projection" layer—React hooks and Container/Presentation components.

**Location:** `apps/client/src/**/*.test.tsx`
**Runner:** `vitest` environment `jsdom`

### What to Test:

- **Hooks (`viewmodel/`)**:
  - Test `useTaskInternal` verifies correct `TaskIntents` calls.
  - Test `usePriorityList` sorts correctly given a mock Automerge doc.
- **Containers**:
  - Verify `DoViewContainer` renders the list and handles interactions.
  - Mock `useTaskIntents` to verify it calls the right methods when buttons are clicked.
- **Presentational Components**:
  - Verify `TaskRow` renders "Overdue" visual cues correctly.

**Setup Required:** (Plan Phase 1)

- Install: `vitest`, `jsdom`, `@testing-library/react`, `@testing-library/user-event`
- Config: `apps/client/vitest.config.ts`

**How to Run:**

```bash
pnpm --filter client test
```

---

## 3. End-to-End (E2E) Tests (`apps/client`)

These tests verify the full application stack in a real browser, including IndexedDB persistence and Service Workers.

**Location:** `apps/client/e2e/**`
**Runner:** `Playwright`

### Critical User Flows (The "Safety Net"):

1.  **First Run**: Load app, see "Inbox", create a task.
2.  **Persistence**: Reload page, verify task is still there (IndexedDB check).
3.  **Offline Mode**: Go offline (Service Worker), complete task, go online.
4.  **Priority View**: Create tasks with different due dates, verify ordering in "Do" list.
5.  **Multi-Tab Sync**: Open app in two tabs, change title in one, verify update in other.

**Setup Required:** (Plan Phase 1)

- Install: `@playwright/test`
- Config: `playwright.config.ts`

**How to Run:**

```bash
pnpm --filter client test:e2e
```

---

## "Testing As You Go" (Workflow for AI)

To ensure "AI can iterate without breaking stuff," follow this workflow for every new feature:

1.  **Write the Unit Test First (TDD)**:
    - If adding a new scoring factor, write a test case in `packages/tasklens/tests` first.
    - Run `pnpm test` -> Red -> Implement -> Green.

2.  **Write the Hook Test**:
    - If adding `useBalanceData`, write a test that mocks `useTunnel` return data and assertions on the hook's output.

3.  **Manual Verification (Interactive)**:
    - Use `pnpm dev` to verify the UI looks right.

4.  **Add/Update E2E Test**:
    - If adding a major feature (e.g., "Move Picker"), add a Playwright test that actually opens the modal and moves a task.

---

## CI/CD Enforcement

Pull Requests should be blocked unless:

- `pnpm lint` passes
- `pnpm build` passes
- `pnpm test` (Unit + Integration) passes

(E2E tests may be run on merge or nightly due to valid speed concerns, but for this project, running them on PR is recommended as the suite is small.)

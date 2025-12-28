# Testing Strategy

This document defines the testing strategy for `mydoo`, ensuring robust progression and safe AI-led iteration.

## The Testing Pyramid

We adhere to a 3-tier testing approach tailored for our Local-First PWA architecture:

| Tier               | Tool                      | Scope                                | Speed        |
| ------------------ | ------------------------- | ------------------------------------ | ------------ |
| **1. Unit Logic**  | `vitest`                  | Domain functions, pure algorithms    | ⚡️ Fast      |
| **2. Integration** | `vitest` + Browser Mode   | React Hooks, Views, Redux store      | 🚀 Medium    |
| **3. End-to-End**  | `Playwright`              | Full App in Browser                  | 🐢 Slow      |

---

## 1. Unit Logic Tests (`@mydoo/tasklens`)

These tests verify the domain "brain" of the application—pure logic with no React dependencies.

**Location:** `packages/tasklens/tests/**`
**Runner:** `vitest`

### What to Test:
- **Priority Algorithm**: Scoring math and filter logic.
- **Schemas**: Zod validation.
- **Domain helpers**: `readiness.ts`, `projections.ts`.

**How to Run:**
```bash
pnpm --filter @mydoo/tasklens test
```

---

## 2. Integration & Component Tests (`apps/client`)

These tests verify React hooks, view models, and component interactions using **Vitest Browser Mode** (Chromium).

**Location:** `apps/client/src/**/*.test.tsx`
**Runner:** `vitest` with `@vitest/browser` (Chromium via Playwright)

> **Note:** We use Vitest Browser Mode instead of jsdom for higher fidelity. Tests run in a real Chromium instance.

### What to Test:
- **Hooks (`viewmodel/`)**:
  - `usePriorityList` returns correct tasks from Redux store.
  - `useTaskIntents` dispatches correct Automerge mutations.
- **Containers**:
  - `DoViewContainer` renders prioritized tasks and handles interactions.
- **Redux Integration**:
  - Tests dispatch `syncDoc` directly to populate the Redux store before rendering hooks.

**Test Setup:**
- `createTestWrapper(repo, store)` provides `RepoContext`, Redux `Provider`, and `MantineProvider`.
- For Redux-consuming hooks, dispatch `syncDoc` with the current doc state before assertions.

**How to Run:**
```bash
pnpm --filter client test
```

---

## 3. End-to-End (E2E) Tests (`apps/client`)

These tests verify the full application stack in a real browser, including IndexedDB persistence.

**Location:** `apps/client/e2e/**`
**Runner:** `Playwright`

### Critical User Flows:
1. **First Run**: Load app, see empty "Do" list, create a task via Quick Add.
2. **Persistence**: Reload page, verify task is still there.
3. **Task Defaults**: Create task and verify default values (status, notes, schedule).
4. **Priority View**: Create tasks with different priorities, verify ordering.
5. **Move Picker**: Open hierarchical move modal, reparent a task.

**How to Run:**
```bash
pnpm --filter client test:e2e
```

---

## Pre-Commit Quality Gates

The pre-commit hook runs via Turbo:
- `pnpm fix` (lint + format)
- `pnpm typecheck`
- `pnpm test` (Unit + Integration)
- `pnpm test:e2e` (E2E)

All must pass before a commit is accepted.

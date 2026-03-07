# Testing Strategy

This document defines the testing strategy for `mydoo`, ensuring robust
progression and safe AI-led iteration.

## The Testing Pyramid

Use `just` as the canonical entrypoint for all test execution. The `justfile`
wraps pnpm/cargo commands, applies required build steps, and supports argument
pass-through for targeted runs. Avoid invoking `pnpm test` or
`pnpm exec playwright test` directly.

We adhere to a 3-tier testing approach tailored for our Local-First PWA
architecture:

| Tier               | Tool            | Scope                             | Speed     |
| ------------------ | --------------- | --------------------------------- | --------- |
| **1. Unit Logic**  | `just test`     | Rust Crate Logic, Algorithm Tests | ⚡️ Fast   |
| **2. Integration** | `just test`     | Dioxus Component Integration      | 🚀 Medium |
| **3. End-to-End**  | `just test-e2e` | Full App in Browser               | 🐢 Slow   |

---

## 1. Unit Logic Tests

These tests verify the core Rust logic—pure algorithms and state management with no
UI dependencies.

**Location:** `crates/tasklens-core/src/**`, `crates/tasklens-store/src/**` **Runner:** `just test` (for Rust crates)

### What to Test:

- **Priority Algorithm**: Scoring math and filter logic.
- **Schemas**: Zod validation.
- **Domain helpers**: `readiness.ts`, `projections.ts`.

**How to Run:**

```bash
just test
```

---

## 2. Integration & Component Tests

These tests verify Dioxus components and their integration with the store.

**Location:** `crates/tasklens-ui/src/**/*.rs` **Runner:** `just test`

> **Note:** We use Vitest Browser Mode instead of jsdom for higher fidelity.
> Tests run in a real Chromium instance.

### What to Test:

- **Hooks (`viewmodel/`)**:
  - `usePriorityList` returns correct tasks from Redux store.
  - `useTaskIntents` dispatches correct Automerge mutations.
- **Containers**:
  - `DoViewContainer` renders prioritized tasks and handles interactions.
- **Redux Integration**:
  - Tests dispatch `syncDoc` directly to populate the Redux store before
    rendering hooks.

**Test Setup:**

- `createTestWrapper(repo, store)` provides `RepoContext`, Redux `Provider`, and
  `MantineProvider`.
- For Redux-consuming hooks, dispatch `syncDoc` with the current doc state
  before assertions.

**How to Run:**

```bash
just test
```

---

## 3. End-to-End (E2E) Tests (`apps/client`)

These tests verify the full application stack in a real browser, including
IndexedDB persistence.

**Location:** `apps/client/e2e/**` **Runner:** `Playwright`

### Critical User Flows:

1. **First Run**: Load app, see empty "Do" list, create a task via Quick Add.
2. **Persistence**: Reload page, verify task is still there.
3. **Task Defaults**: Create task and verify default values (status, notes,
   schedule).
4. **Priority View**: Create tasks with different priorities, verify ordering.
5. **Move Picker**: Open hierarchical move modal, reparent a task.

**How to Run:**

```bash
just test-e2e
```

### Running Individual Tests

For faster iteration, you can run specific tests or patterns:

1.  **By Pattern (Grep)**:
    ```bash
    just test-e2e -- -g "some scenario name"
    ```
2.  **Specific Spec File**:
    - **Native Specs**:
      `just test-e2e -- tests/e2e/specs/due-dates.spec.ts`
    - **BDD Features**: First run `pnpm run generate`, then:
      `just test-e2e -- tests/e2e/.features-gen/desktop/tests/e2e/features/due-dates.feature.spec.js`
3.  **UI Mode**:
    ```bash
    just test-e2e -- --ui
    ```

---

## Pre-Commit Quality Gates

**General:**

- `just fix` (lint + format)
- `just check-types`
- `just test` (Unit + Integration)
- `just test-e2e` (E2E)

All must pass before a commit is accepted.

### Docs-Only Exception

For documentation-only changes (no runtime, source, or test behavior changes):

- Do not run `just verify` by default.
- Run document-format checks (for example, `pnpm prettier --check` on changed
  docs) and rely on commit hooks.
- If a docs change accompanies code changes, treat it as a code change and run
  the normal quality gates.

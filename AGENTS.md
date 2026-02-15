# AGENTS.md

## 1. Project Context

**mydoo** is a local-first, synchronization-agnostic task management system. It eliminates "list rot" by dynamically promoting tasks based on a **"Life Balance" algorithm** (Target vs. Actual effort) and **"Autofocus" principles** (surfacing neglected tasks). The device is the source of truth.

### Key Technologies

- **Frontend:** Dioxus (Rust) compiled to WASM.
- **State Management:** `automerge` (CRDTs) wrapped in `tasklens-store`.
- **Persistence:** IndexedDB via `rexie` (Dioxus).
- **Backend/Sync:** WebSocket via custom Rust sync implementation.
- **Core Logic:** Rust/WASM (shared via `tasklens-core`).

## 2. Key Documentation

Use these documents to understand the system architecture and requirements:

- **[Product Requirements (PRD)](docs/design/prd.md):** The source of truth for features and **Ubiquitous Language**.
- **[System Architecture](docs/design/architecture.md):** Layering strategy (UI -> ViewModel -> Domain -> Store).
- **[Code Review Guidance](docs/guidance/code-review.md):** Strict TypeScript rules and best practices.
- **[Scoring Algorithm](docs/design/algorithm.md):** The core logic for task prioritization.
- **[Testing Strategy](docs/design/testing.md):** The 3-tier testing pyramid (Unit, Integration, E2E).
- **[Automerge Schema](docs/design/automerge-schema.md):** The exact structure of the CRDT document.
- **[Dioxus Guide](AGENTS_DIOXUS.md):** Syntax cheat sheet for Dioxus 0.7 components.

## 3. Development Guidelines

### Package Management

- Use `pnpm`, not `npm`.
- Avoid `pnpm dlx`. Favor deterministic alternatives (install + exec) for reproducibility.
- Use `cargo` for Rust.
- Use `just` for running commands.
- **Dioxus Components:** Use `cargo xtask dx-components vendor` (never run `dx components add` directly).

### Environment Initialization

- **Install:** `pnpm install`.

### Git Workflow

- **Clean Tree Rule:** Ensure clean working tree before starting new work.
- **Feature Branch Isolation:** Each task gets its own branch (`codex/<task-slug>`).
- **PR Isolation:** One concern per PR.
- **Commit Rule:** Commit autonomously when quality gates pass (`just verify`).
  - **Do NOT commit if:** Tests/lints fail or changes are experimental.
  - **Protocol:** Summarize changes, confirm gates passed, state commit message.
- **Verification:** ALWAYS verify `Exit Code` is `0` before claiming success.

### Coding Guidelines

- **TypeScript Strictness:** No `any`, no unsafe casting (`as`/`!`), exhaust all unions.
- **Documentation:** Document all new code and non-obvious logic.
- **Testing:** All new code must have tests.

## 4. Testing Requirements

**Verification Strategy:**

- **Full Verification:** `just verify` (Complex logic, refactors).
- **Standard Testing:** `just test` or `just test-e2e` (Routine logic).
- **Presubmit:** `git push` hooks (Documentation, formatting).

### Test Commands

```bash
just test          # All unit tests
just test-scripts  # Scripts package unit tests
just verify        # Full build and test suite
just check-rust    # Full Rust validation

# E2E Tests
# NOTE: these commands automatically rebuild the Dioxus app before running tests.
just test-e2e      # All E2E tests
just test-e2e -- --ui  # Open Playwright UI mode
just test-e2e -- -g "my test name" # Run specific test by name
just test-e2e -- crates/tasklens-ui/tests/e2e/specs/my.spec.ts # Run specific file
```

### AI Agent Instructions

- **Avoid Fake Events:** Do not use synthetic events. Test through realistic user interactions (clicks, keyboard input) using Playwright locators.
- **Ubiquitous Language:** ALWAYS use domain terms (`Inbox`, `Plan`, `Do`, `Balance`, `Context`) in tests.
- **Code-First Gherkin:** Scenarios are written in TypeScript using strictly typed actor fixtures.

## 5. Technical Field Notes

### Playwright & E2E Strategies

- **Semantic Selectors:** Use `data-testid` or `data-urgency`, avoid CSS styles.
- **Timezone Pitfalls:** `page.clock.setFixedTime()` sets system time (UTC), but `new Date()` uses browser timezone.
- **WASM Init Race:** Use `page.waitForFunction` to ensure custom WASM APIs are attached.
- **Focus Traps:** Avoid `expect(locator).toBeFocused()` in dialogs; interact with inputs directly.
- **Dialog Stacking:** Ensure dialogs close/unmount to avoid occlusion.
- **Dioxus Toast "Parking":** Toasts are parked off-screen (`right: -1000px`), appearing "visible" to Playwright.
- **Worker Collision:** Disable `fullyParallel` if using stateful `IndexedDB`.
- **Reload Race:** `repo.import()` then `location.reload()` needs a settle delay.

### Automerge & Autosurgeon Patterns

- **Asymmetric Serialization:** Use `Hydrate` (broad) and `Reconcile` (strict) asymmetrically.
- **`hydrate_prop` vs `MaybeMissing`:** Use `MaybeMissing<T>` for optional keys.
- **Numeric Strictness:** Treat `ScalarValue::Int` and `F64` as equal if values match.
- **Realized Text:** Use `am_text` helper to assert on `Text` objects.

### Rust & Dioxus Development

- **Workspace:** Isolate Rust crates in `crates/`, shared deps in `[workspace.dependencies]`.
- **WASM Gating:** Use `[target.'cfg(target_arch = "wasm32")'.dependencies]` for web-only crates.
- **Tree-Walking:** Use `FlattenContext` to avoid deep recursion warnings.
- **Hydration Race:** Initialize store with default/empty state synchronously.
- **Async Locking:** Never hold a `Signal` or `RefCell` write lock across an `.await`.
- **WASM Debugging:** Use `console_error_panic_hook` and `expect("message")` over `unwrap()`.
- **Numeric Interop:** Use `f64` for ALL numeric fields crossing FFI/Automerge.
- **CSS Modules:** Use `#[css_module(...)]` struct Styles for scoped CSS.

## 6. Session Completion

**MANDATORY WORKFLOW:**

1. **File issues** for follow-up work.
2. **Run quality gates** (tests, linters).
3. **PUSH TO REMOTE** (`git push`).
   - `git fetch origin`
   - `git merge origin/main`
   - `git push`
   - `git status` (Must show "up to date")
4. **Verify** all changes are pushed.

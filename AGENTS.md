# Project Memories

## Immutable Design Decisions

- **Track Generated CSS:** The file `crates/tasklens-ui/assets/tailwind.css`
  MUST be tracked in Git. It is NOT to be gitignored.
- **Formatting Authority:** To resolve formatting conflicts for
  `assets/tailwind.css`, it is added to `.prettierignore` so that the Dioxus
  build process remains the primary authority for its formatting, while it
  remains tracked in the repository.

# Behavior Guidelines

- Always address the user with the salutation "Howdy".

# Development Guidelines

## Package Management

- Use `pnpm` for all package management and scripts.

## Environment Initialization

- **Dependency Installation:** Run `pnpm install`.
- **Clean Install:** If you need to clean the environment, use
  `scripts/aggressive-git-clean.sh` followed by `pnpm install`.

## Git Workflow

- **Clean Tree Rule:** Before starting unrelated work or a new development
  phase, run `git status`. If the working tree is not clean, STOP and notify the
  user.
- **Git Commit Rule:** NEVER commit a git change without an explicit command
  from the user beginning with `git commit`. If the user asks to commit (e.g.,
  "commit this") without the explicit command, STOP and ask for confirmation.
  - **Enforcement Protocol:** When work is complete and ready to commit, the
    Agent MUST explicitly state "Waiting for git commit command" and STOP. The
    Agent must NOT infer or assume permission to commit from context, prior
    commits, or phrases like "continue" or "finish this."
- **No Auto-Staging Rule**: The Agent MUST NOT stage its own manual code edits
  using `git add`. The user relies on unstaged changes to review the Agent's
  work via the diff.
  - **The Content-Stage Heuristic**: If the Agent touch the content of a file
    (logic or documentation), the Agent MUST NOT touch the stage for that file.
    Meta-documentation updates (like `AGENTS.md`) are NOT exceptions.
  - **Exception**: Staging IS permitted and encouraged for results produced by
    vetted automated tools (e.g., `./scripts/format`, `cargo fmt`, `pnpm fix`)
    or when explicitly instructed by prompts, commands, workflows, or skills.
  - **Future Tense Prohibition:** Phrases like "we'll commit", "undo and redo",
    or "fix and commit" act as plans, NOT commands. You must execute the work
    (e.g., the undo/fix) and then **STOP** to request a fresh commit command.
  - **Verification Check:** Before executing any `git commit`, the Agent must
    verify: "Did the user's most recent message start with 'git commit'?" If NO,
    STOP immediately.
- **Git Push Rule:** NEVER run `git push` unless the user explicitly commands it
  (e.g., "push", "sync"). "Commit" does NOT imply "Push". You must stop after
  committing.
- **Ambiguity Protocol:** If a user's instruction combines a correction ("undo
  this") with a future action ("we'll commit"), treating the future action as
  authorized is a **Violation**. You must strictly separate the immediate
  correction from the subsequent authorization.
- **Git Presubmit Rule:** NEVER use `--no-verify`. On presubmit failure: fix
  trivial issues and retry; otherwise STOP AND WAIT.
- **Foreground Commit Rule:** ALWAYS run `git commit` in the foreground
  (synchronously). Presubmit hooks often fail or warn; immediate feedback is
  required to retry or fix issues promptly.
- **Verification Protocol:** The Agent must NEVER report a shell command as
  successful based on log output alone. The Agent MUST verify that the process
  `Exit Code` is `0` (via `command_status`) before claiming success.
- **Secret Verification:** The Agent must set the `AGENT_GIT_COMMIT_SECRET`
  environment variable to the correct value ("I follow directions.") when
  running `git commit`. Do not export it into the environent; simply do:

  ```sh
  AGENT_GIT_COMMIT_SECRET="..." git commit [args]
  ```

## Coding Guidelines

# TypeScript Strictness & Type Safety Rules

1.  **Zero-Tolerance for `any`:** Never use the `any` type. If a type is
    difficult to express, you must define an interface, use a generic, or
    utilize utility types (`Pick`, `Omit`, etc.) to construct it correctly.
2.  **Prohibition on Casting:** Do not use `as` casting (e.g.,
    `variable as Type`) or non-null assertions (`!`) to silence type errors.
    Casting is only acceptable when bridging boundaries (e.g., parsing raw JSON)
    and must be accompanied by runtime validation (like Zod) or a user-defined
    type guard.
3.  **Type Errors are Logical Defects:** Treat a compilation error as a
    structural defect in the code's logic or data flow, not a hurdle to be
    bypassed. If types do not match, change the implementation or the data
    structures to align—do not relax the type definition to satisfy the
    compiler.
4.  **No `unknown` Lazy-Loading:** Do not type variables as `unknown` to defer
    typing decisions. Only use `unknown` if the value is truly dynamic at
    runtime, and immediately narrow it using control flow analysis.
5.  **Exhaustiveness:** When handling unions (especially in `switch`
    statements), ensure all cases are handled. Use a `assertUnreachable` utility
    if necessary to guarantee exhaustiveness.
6.  **The Protocol of Strictness:** If you cannot express a type correctly
    without using `any` or `as` casting, **STOP**. A complex type puzzle you
    cannot solve is a signal to pause and ask the user for guidance, not a
    license to bypass the type system. Lowering the quality bar to achieve
    autonomy is **unacceptable**.

## Documentation

- All new code must have documentation comments. Explain all non-obvious logic.
- Do not remove comments from existing code unless asked to do so by the user.
- Keep comments up to date.
- **Code Review Guidance:** See
  [docs/guidance/code-review.md](docs/guidance/code-review.md) for best
  practices on TypeScript, React, Redux, and testing conventions.
- **Testing Strategy:** See [docs/design/testing.md](docs/design/testing.md) for
  the authoritative guide on unit, integration, and E2E testing execution.
- **Markdown Style:** Use markdown bold and italics rarely.

## Testing Requirements

- We use Vitest for testing.
- All new code must have tests.
- **Strict Verification:** ALWAYS run `pnpm verify` before certifying a change.
  You MAY use targeted `turbo` commands during development, but you MUST run the
  full `pnpm verify` sequence before asking the user to commit.

### Test Commands

```bash
# All unit tests (monorepo-wide)
pnpm test

# All E2E tests (monorepo-wide)
pnpm test:e2e

# All unit tests in a specific package
pnpm exec turbo run test --filter <package>
# e.g. pnpm exec turbo run test --filter @mydoo/client

# Specific test file within a package
pnpm exec turbo run test --filter <package> -- <RelativePathToTestFile>
# e.g. pnpm exec turbo run test --filter @mydoo/client -- src/test/utils/date-formatter.test.ts

# Specific E2E feature or test with Playwright/BDD
# Note: Always run bddgen first if modifying .feature files
pnpm exec turbo run test-e2e --filter <package> -- --project=<project> -g <pattern>
# e.g. pnpm exec turbo run test-e2e --filter @mydoo/client -- --project='bdd-desktop' -g 'Due Dates'

# Fully build everything and re-run all tests including e2e (monorepo-wide), ignoring cache
TURBO_FORCE=true pnpm exec turbo run check-agent
```

### Rust Validation

The `pnpm verify` command includes Rust checks via `scripts/check-rust.sh`. You
can also run Rust checks independently:

```bash
# Full Rust validation (fmt, clippy, WASM build, dx build, tests)
pnpm check-rust

# Individual checks during development
cargo fmt --check                                    # Formatting
cargo clippy --all-targets -- -D warnings           # Lints
cargo build --target wasm32-unknown-unknown -p tasklens-store  # WASM build
dx build -p tasklens-ui                             # Dioxus build
cargo test                                          # Tests
```

## Context Convention: `ROLLING_CONTEXT.md`

For efforts spanning multiple sessions or commits, we maintain a root-level
`ROLLING_CONTEXT.md`.

- **Purpose:** Persistent, user-editable "working memory" for the project's
  current focus.
- **Workflow:**
  - **User:** Updates this file to set high-level goals, shift direction, or
    clarify requirements.
  - **Agent:** Reads at new phase start (per Clean Tree Rule). Must keep rolling
    task lists up to date. May update other agent-designated sections
    autonomously.
- **Contrast with System Artifacts:**
  - `task.md` / `implementation_plan.md`: The agent's _ephemeral, internal_
    checklist for the immediate next step.
  - `ROLLING_CONTEXT.md`: The _persistent, shared_ narrative of the broader
    effort. It is not tracked in git.

## Testing Strategy

For a detailed implementation guide, see
[docs/design/testing.md](docs/design/testing.md).

### Core Philosophy: Fidelity First

We prioritize fidelity. Because this is a Local-First application relying on
**Automerge** (WASM, Binary CRDTs) and specific browser technologies
(`IndexedDB`, `TextEncoder`, `Crypto`), our strategic goal is to **avoid
simulated environments like JSDOM**.

### 1. The 3-Tier Architecture (Goal State)

| Tier       | Scope      | Target Infrastructure   | Rationale                                          |
| :--------- | :--------- | :---------------------- | :------------------------------------------------- |
| **Tier 1** | Pure Logic | **Node.js**             | Native WASM performance; no DOM pollution.         |
| **Tier 2** | Components | **Vitest Browser Mode** | Real `IndexedDB` and `TextEncoder` implementation. |
| **Tier 3** | Journeys   | **Playwright**          | Full multi-tab sync and offline/online simulation. |

### 2. Current Implementation Status

> **Note:** We are largely in **JSDOM** today. Use the rules below for daily
> development.

**Tier 1 (Packages/Logic):**

- **Status:** Runs in **Node**.
- **Command:** `pnpm test` (via Turbo).

**Tier 2 (Client Components):**

- **Status:** Runs in **JSDOM**.
- **Command:** `pnpm test` (via Turbo).
- **Aspiration:** We will migrate this to Browser Mode (`--project=browser`).

**Tier 3 (E2E):**

- **Status:** Runs in **Chromium**.
- **Command:** `pnpm test:e2e`.

### 3. AI Agent Instructions

1.  **New Test Mandate:** All **NEW** tests must strictly follow the
    **Executable Specs** style (Section 4). **Do not mimic legacy patterns**
    found in existing files.
2.  **Generate Compatible Tests:** Write component tests using
    `@testing-library/react` that pass in JSDOM.
3.  **Respect the Goal:** Avoid relying on JSDOM-specific APIs (`jest-dom`
    extensions are okay, but don't access `window.` internals directly if a
    standard API exists).
4.  **Mocking Strategy:**
    - **Tier 1:** No mocks. Test logic directly.
    - **Tier 2:** **Mock AutomergeRepo**. JSDOM struggles with the Repo's
      binary/WASM networking. Mock the handle to ensure component tests are
      stable in the simulated environment.
5.  **Prefer `userEvent` over `fireEvent`**: Always use
    `@testing-library/user-event`. Use of `fireEvent` from
    `@testing-library/react` is strictly prohibited as it doesn't simulate real
    browser interactions and often leads to flaky tests in async environments.

### 4. Executable Specs & Style Guide

Tests are **Executable Specifications**. They should read as high-level
narratives using the project's **Ubiquitous Language**.

**Ubiquitous Language Rule:** ALWAYS use domain terms (`Inbox`, `Plan`, `Do`,
`Balance`, `Context`). See `docs/design/prd.md` for the dictionary. Reject
implementation-level terms in test narratives.

- ✅ `plan.createTask("Buy Milk")`
- ❌ `button.click()`, `journal.entry()`

**Fluent Architecture (Playwright E2E):**

- Use `test.step` for high-level narrative structure.
- **NO** direct `page`, `locator`, or CSS selectors in the spec layer—delegate
  to Domain Helpers or Page Objects.
- Use **Inline Gherkin** comments (`// Given`, `// When`, `// Then`) to define
  intent.

**Example:**

```typescript
test("User can organize tasks", async ({ plan }) => {
  await test.step("Create task", async () => {
    // Given the user is on the Plan view
    // When they create a new task
    await plan.createTask("Buy Milk");
    // Then the task appears in the list
  });
});
```

### 5. BDD Support (Playwright)

We use `playwright-bdd` to generate Playwright tests from Gherkin `.feature`
files.

1.  **Generation Step:** BDD tests MUST be generated manually before execution.
    Tests are NOT automatically generated by `pnpm test`.

    ```bash
    # Generate BDD tests for the client
    pnpm --filter @mydoo/client exec bddgen

    # Or manually from apps/client
    pnpm exec bddgen
    ```

2.  **Output Location:** Generated tests are stored in `tests/e2e/.features-gen`
    (split by profile, e.g., `/desktop`, `/mobile`). These files are ignored by
    git and Prettier.
3.  **Stale Files:** If you delete or rename a `.feature` file, you MUST run
    `bddgen` to remove the corresponding stale `.spec.js` file. Stale files will
    cause `test-e2e` failures.

## Testing Workflows

### Running Specific Tests

To run only the algorithm BDD tests:

```bash
pnpm test tests/unit/algorithm.test.ts
```

To run a specific feature within the algorithm tests, use the `FEATURE_FILTER`
environment variable:

```bash
FEATURE_FILTER="Inheritance" pnpm test tests/unit/algorithm.test.ts
```

Or use the standard Vitest `-t` flag (which runs all but skips non-matching):

```bash
pnpm test tests/unit/algorithm.test.ts -t "Inheritance"
```

# Learnings

## Strict Redux & TypeScript Strategies

- **Prefer Inference over Casting:** When configuring a Redux store, rely on
  `configureStore`'s automatic type inference. Avoid explicitly typing the
  generic parameters if it leads to complex intersection types that require
  casting.
- **Middleware Composition:** Use `.concat()` for adding middleware (e.g.,
  `getDefaultMiddleware().concat(myMiddleware)`). This preserves the specific
  types of the middleware (like Thunk capabilities) better than `new Tuple()`,
  allowing `AppDispatch` to correctly infer `ThunkDispatch` without manual
  intervention.
- **`combineReducers` for Safety:** Even if you only have one reducer, using
  `combineReducers` can help satisfy TypeScript's `ReducersMapObject`
  requirements more naturally than a raw object literal, avoiding the need for
  `as Reducer` casts.
- **Forbidden Casts:** `as any` and `as unknown` are strictly forbidden. If you
  are tempted to use them, the architecture or the type definition is likely
  wrong. Simplify the approach (e.g., switch to inference) rather than forcing
  the type.
- **File Corruption:** comprehensive file overwrite tools should never include
  markdown code block delimiters (```) inside the replacement content unless
  they are part of the string literal being written.

## E2E & BDD Strategy (Playwright)

- **Feature File Location:** `playwright-bdd` is strict about file location. For
  `@mydoo/client`, feature files MUST be placed in
  `apps/client/tests/e2e/features/`. Placing them in other folders (like the
  root `tests/`) will cause them to be ignored by the generator.
- **Generation Workflow:** Any change to a `.feature` file requires running the
  generator: `pnpm exec turbo run generate --filter @mydoo/client`. Tests are
  not auto-generated at runtime. Stale specs cause misleading failures.
- **Semantic Selectors:** Avoid checking CSS styles (e.g. colors) or fragile
  label text. Instead, modify components to emit stable data attributes (e.g.,
  `data-urgency="Overdue"` or `data-testid="my-element"`) and assert on those.
- **Timezone Pitfalls:** `page.clock.setFixedTime(...)` sets the system time
  (often UTC in CI), but `new Date()` in the browser uses the browser's local
  timezone. When testing date boundaries (e.g., "due today"), ensure the test
  environment and browser timezone align, or use ISO strings that force specific
  handling.
- **Declarative Steps:** Prefer high-level domain actions ("Given I have a clean
  workspace") over implementation details ("Given I click the settings button").
- **Debugging "Element Not Found":**
  1.  **Inject Console Relays:** In the Page Object constructor, adding
      `page.on("console", msg => console.log(msg.text()))` renders browser logs
      in the Node process. This is the only way to see what's happening inside
      the app during a headless run.
  2.  **Verify Component Existence:** If expected logs from a child component
      (e.g., `[DEBUG] Rendering Badge`) do not appear, the issue is likely that
      the **parent is not rendering the child at all**, not that the child logic
      is broken. Check the parent's `render` method immediately.
- **Migration Tagging (`@migration-pending`)**: When porting test suites, import
  all features but tag unimplemented ones with `@migration-pending` (configured
  with `grepInvert`). This validates the harness immediately while maintaining a
  "Green CI" baseline during incremental implementation.
- **Framework-Specific Selector Purge**: When porting E2E tests, immediately
  replace legacy framework selectors (e.g., `.mantine-Badge-root`) with semantic
  `data-testid` attributes. Do not attempt to emulate legacy class names in the
  new implementation; decouple the test instead.
- **Focus Traps Break Focus Assertions**: When testing components that use modal
  dialogs with focus traps (like `Dialog`), avoid
  `expect(locator).toBeFocused()` assertions. The focus trap may steal focus
  from the expected element.
  - _Pattern:_ Instead of asserting focus, directly click and fill the input:
    ```typescript
    // ❌ Brittle: await expect(titleInput).toBeFocused();
    // ✅ Robust:
    await modal.getByLabel("Title").fill(title);
    ```

## Rust & Dioxus Workspace Strategies (Migration)

- **Root Workspace Pattern:** For hybrid JS/Rust monorepos, use a root-level
  `Cargo.toml` with a `[workspace]` definition. This allows `cargo` commands to
  be run from the root.
- **Member Isolation (`crates/` vs `apps/`):** During a migration, isolate ALL
  new Rust crates (logic, stores, and UI) inside a top-level `crates/`
  directory. This prevents confusion with legacy Node.js-based `apps/` and
  `packages/`.
- **Dependency Centralization:** Define all shared dependencies in the root
  `Cargo.toml` under `[workspace.dependencies]`. In individual crates, use
  `dependency_name.workspace = true`. This ensures version parity across the
  migration effort.
- **Dioxus Config Isolation:** Each Dioxus app (UI crate) needs its own
  `Dioxus.toml` within its crate directory.
- **WASM Feature Flags:** When targeting WASM (Dioxus), ensure dependencies
  supporting browser environments (e.g., `getrandom`, `uuid`, `chrono`) have the
  `js` or `wasmbind` features enabled in the root workspace dependencies to
  prevent compilation errors in the WASM target.
- **Gitignore Hygiene:** Add `target/` and binary output directories to the root
  `.gitignore` early to prevent accidental inclusion of heavy Rust build
  artifacts.
- **Dioxus Workspace Platform Detection:** `dx build` and `dx serve` may fail to
  detect the target platform in a workspace if running from the root.
  - _Fix:_ Explicitly add `platform = "web"` to the `[application]` section of
    the crate's `Dioxus.toml`.
  - _Fix:_ Ensure the `dioxus` dependency in `Cargo.toml` has the `web` feature
    enabled (e.g., `features = ["web"]`).
- **Schema Source of Truth (Migration Trap):** When migrating TypeScript types
  to Rust, be wary of mismatched schemas.
  - _Trap:_ `feature.schema.json` (BDD) often uses **recursive** structures for
    test convenience.
  - _Truth:_ The actual persistence schema (e.g., `schemas.ts` / Zod) often uses
    **flat** relational structures for database efficiency.
  - _Action:_ Always port types from the **Persistence/Zod** layer, NOT the
    BDD/Test layer, to ensure the data model satisfies the storage engine
    (Automerge).
- **Autosurgeon Bridge Strategy:**
  - _Context:_ We use `autosurgeon` to bridge Rust structs and Automerge
    documents.
  - _Mechanism:_ `autosurgeon` relies on `serde::Serialize` and
    `serde::Deserialize` traits.
  - _Verification:_ Validating that a Rust struct serializes to the exact same
    JSON as the TypeScript `TaskSchema` (Zod) is a valid proxy for verifying
    Automerge compatibility, as both `autosurgeon` and `JSON.stringify` follow
    the Serde data model.
- **Migration Governance:**
  - _Plan Authority:_ `docs/plan/rust_migration.md` is the authoritative,
    persistent implementation plan for the migration.
  - _Workflow:_ Do not create ephemeral `implementation_plan.md` artifacts for
    migration tasks. Update `docs/plan/rust_migration.md` with concrete file
    changes (`[NEW]`, `[MODIFY]`) and task lists.
  - _Context:_ This ensures a single source of truth for the multi-epoch
    migration effort.
- **Dioxus Router URL Parameter Types**:
  - _Requirement:_ Custom types used as URL parameters in Dioxus routes (e.g.,
    `Route::PlanPage { focus_task: Option<TaskID> }`) must implement
    `std::fmt::Display` and `std::str::FromStr` traits.
  - _Symptom:_ Navigation fails silently or produces confusing serialization
    errors.
  - _Fix:_ Implement both traits for ID wrapper types:
    ```rust
    impl std::fmt::Display for TaskID {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::str::FromStr for TaskID {
        type Err = std::convert::Infallible;
        fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Self(s.to_string())) }
    }
    ```

## Rust Async Testing & Framework Gotchas

- **Async Testing with `LocalPool`**: When using `futures::executor::LocalPool`
  in unit tests:
  - _Trap:_ `pollster::block_on` only blocks on the provided future and **does
    not poll background tasks** spawned on the `Spawner`. This can lead to hangs
    or `SpawnError("shutdown")` if the pool is dropped while background tasks
    are still pending.
  - _Fix:_ Always use `pool.run_until(async { ... })` and ensure the `pool`
    variable is kept alive (named `pool`, not `_pool`, as per project
    preference) throughout the test execution.
- **Autosurgeon Hydration Errors (`Unexpected String`)**:
  - _Symptom:_ `hydrate` fails with `Some(Unexpected(String))` or
    `unexpected string`.
  - _Cause:_ This often indicates a mismatch between the Automerge document
    structure and the Rust type. It frequently occurs with `transparent`
    wrappers (like `TaskID(String)`) or nested fields where `autosurgeon`
    expects a specific container type but finds a scalar.
  - _Action:_ Verify that `autosurgeon` attributes (like `#[autosurgeon(key)]`
    or `#[autosurgeon(rename = "...")]`) align with the data actually present in
    the Automerge document.
- **WASM Dependency Gating in Cargo**:
  - _Pattern:_ Use `[target.'cfg(target_arch = "wasm32")'.dependencies]` to
    isolate web-specific crates (`rexie`, `wasm-bindgen`) from native builds.
    This avoids compilation errors on native targets that don't satisfy
    web-specific feature requirements.

## Rust-JavaScript Interoperability

- **Numeric Type Requirement for Automerge/JavaScript Interop**:
  - _Rule:_ ALL numeric fields in Rust types that serialize to/from Automerge
    documents MUST use 64-bit double-precision IEEE 754 floating-point values
    (`f64`). This is a critical requirement to guarantee interoperability with
    JavaScript implementations.
  - _Rationale:_ JavaScript's `Number` type is exclusively IEEE 754 double
    precision. Using `u64`, `i64`, `u32`, or other integer types in Rust will
    cause type mismatches and data corruption when round-tripping through
    JavaScript/TypeScript code.
  - _Scope:_ This applies to ALL numeric fields including:
    - Timestamps (milliseconds since epoch)
    - Counters and IDs (e.g., `nextTaskId`, `nextPlaceId`)
    - Durations and intervals (e.g., `leadTime`, `interval`)
    - Scores, priorities, and other computed values
    - Credit/importance values
  - _Exception:_ Internal Rust-only types that never cross the FFI boundary may
    use native integer types, but this should be rare in a hybrid codebase.
- **Fixture Testing with Immutable JSON**:
  - _Pattern:_ When testing against immutable JSON fixtures (e.g., production
    data snapshots), use a normalization function to convert all JSON numbers to
    `f64` representation before comparison.
  - _Implementation:_ Create a recursive `normalize_json()` helper that converts
    `serde_json::Value::Number` to consistent `f64` values, preserving booleans
    and other types.
  - _Rationale:_ This allows Rust's `f64`-based serialization (which produces
    `1.0`, `604800000.0`) to be compared against legacy JSON fixtures with
    integer notation (`1`, `604800000`) without modifying the immutable fixture.
- **Type Conversion Cascade**:
  - _Pattern:_ When converting a codebase from integer types to `f64`, the
    change cascades through multiple layers:
    1. Core domain types (structs with `#[derive(Serialize, Deserialize)]`)
    2. Function signatures in domain logic
    3. Test fixtures and mock data (all integer literals → `f64` literals)
    4. Compliance/BDD test infrastructure (date parsing, time arithmetic)
    5. Constants and default values
  - _Verification:_ Run `cargo check`, `cargo clippy`, and all test suites
    (`cargo test`) to catch every location requiring updates. The compiler will
    identify all type mismatches.

## Dioxus & WASM Debugging

- **Server-Side Runtime Requirement**: Unlike static file hosting, Dioxus
  applications (especially when using `dx serve`) require the development server
  to be actively running to serve the WASM/JS bundle and handle HMR.
  - _Symptom:_ "Connection Refused" or failed checks on `localhost:8080` when
    investigating runtime issues.
  - _Fix:_ Always ensure `dx serve` is running in a background process (like
    `npm run dev`) before attempting to debug runtime behavior or browse the
    app.
- **Verify Build Status Before Browsing**: When using `dx serve`, always verify
  that the build has succeeded (by checking terminal output or a quick page load
  check) before launching a long-running browser agent task. Accessing the app
  while the build is failing or still compiling will waste time and tokens.
  Dioxus displays build errors directly in the browser window; if you see these,
  bail immediately.
- **Ambiguity Analysis & Debug Symbols**:
  - **Interpretation**: If you see raw `wasm-function[...]` offsets in a stack
    trace, you are likely missing debug symbols (common in release builds).
  - **Action**: Enable debug symbols in `Cargo.toml`
    (`[profile.release] debug = true`) to resolve function names. Note this
    increases binary size significantly.
  - **Tooling**: Use `console_error_panic_hook` to translate Rust panics into
    readable JS/console errors.
  - **Time Panic**: The error `time not implemented on this platform` on
    `wasm32-unknown-unknown` almost always means `std::time` is being used
    directly or by a dependency without WASM support features enabled. Fix by
    using `web-time`, `instant`, or enabling `wasm-bindgen`/`js` features on
    dependencies (e.g., `chrono`). Include context from @AGENTS.md
- **Dioxus Hydration & Store Initialization**:
  - _Symptom:_ `Hydration failed: unexpected None` when loading views that
    depend on `use_context`.
  - _Root Cause:_ The `use_context_provider` call must happen _before_ any child
    components attempt `use_context`. If state initialization is async or
    conditional, children may render before context is available.
  - _Fix:_ Initialize the store with default/empty state synchronously in the
    root component's `use_context_provider`, then hydrate from persistence
    asynchronously. Never let the context be `None` during initial render.
  - _Pattern:_ Prefer `use_context_provider(|| AppStore::default())` over
    `use_context_provider(|| load_from_db().await)`.
- **Dioxus Toast "Parking" Pattern**:
  - _Symptom:_ The Dioxus hot-reload toast (`#__dx-toast-text`) appears in the
    DOM even when not visible, causing E2E tests to potentially interact with it
    or wait for it incorrectly.
  - _Root Cause:_ Dioxus "parks" the toast off-screen using `right: -1000px`
    rather than `display: none`. It remains in the DOM tree.
  - _Fix:_ When creating DOM introspection utilities (e.g., synthetic DOM
    serializers for debugging), use geometric visibility checks
    (`getBoundingClientRect()` vs viewport) not just `display`/`visibility`
    styles.

## E2E & BDD Strategy (Playwright)

- **Dioxus Child Task Visibility Requires Parent Expansion**:
  - _Symptom:_ Child tasks created via "Add Child" are not found by Playwright
    locators even though the action succeeded.
  - _Root Cause:_ In the Dioxus Plan view, child elements are only rendered into
    the DOM when their parent task is expanded. Collapsed parents do not render
    their children.
  - _Fix (WORKAROUND):_ After creating a child task, explicitly call
    `toggleExpand(parent, true)` in the fixture before asserting child
    visibility. Do not assume the UI auto-expands.
  - _Intent:_ The UI _should_ automatically expand the parent when a child is
    added. This workaround exists because that feature is TBD. The test logic
    should be removed once implemented.
- **IndexedDB.deleteDatabase() is Async (Fire-and-Forget Trap)**:
  - _Trap:_ `indexedDB.deleteDatabase("name")` returns an `IDBOpenDBRequest`,
    not a Promise. Calling it without awaiting `onsuccess` means the database
    may not be deleted before the test proceeds.
  - _Fix:_ Wrap in a Promise:
    ```typescript
    await page.evaluate(
      () =>
        new Promise<void>((resolve, reject) => {
          const req = indexedDB.deleteDatabase("tasklens_db");
          req.onsuccess = () => resolve();
          req.onerror = () => reject(req.error);
        }),
    );
    ```
- **Feature File Location:** `playwright-bdd` is strict about file location. For
- **Platform-Specific Import Guards**:
  - _Symptom:_ Unused import warnings for `std::time::{SystemTime, UNIX_EPOCH}`
    when building for WASM, but the types are actually used in a
    `#[cfg(not(target_arch = "wasm32"))]` block.
  - _Fix:_ Apply `#[cfg(not(target_arch = "wasm32"))]` to the `use` statement
    itself, not just the code block that uses it:
    ```rust
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::{SystemTime, UNIX_EPOCH};
    ```
- **Prefer `expect()` over `unwrap()` in WASM**: In WASM builds, panic messages
  are harder to trace. Always use `expect("descriptive message")` instead of
  `unwrap()` to provide context in crash logs.
  - _Example:_ `draft().expect("draft should be initialized")` vs
    `draft().unwrap()`

## Project Management

- **Milestone Granularity - The "View" Trap**: "Views" are often deceptively
  large milestones because they implicitly require implementing all mutation and
  component infrastructure to be testable.
  - _Hard-won lesson:_ Milestone 3.3 (Do View) absorbed Milestone 3.4 (Task
    Editor) because we couldn't easily verify the view without creating/editing
    data. This resulted in a massive, hard-to-review change set.
  - _Mitigation:_ Break milestones by **Capability** (e.g. "Data Rendering",
    "Status Toggling", "Creation Flow"), not by **View**. If a milestone
    requires a complex new component (like a DatePicker), that component is a
    milestone of its own.

# Validation

To validate changes in this project, run:
`cargo build --target wasm32-unknown-unknown` in addition to `cargo check` and
`cargo clippy`.

# Coding Standards

- **Dead Code Handling**: Prefer `#[expect(dead_code)]` over
  `#[allow(dead_code)]` for code that is intentionally left in place but
  currently unused (e.g., library variants).
- Do not use "conventional commit" prefixes (e.g. `feat:`, `fix:`) in commit
  messages.
- Do not use markdown quoting (backticks) in commit messages.
- Always run "./scripts/format --write" to format files before committing, and
  be sure to stage the changes.
- Always check the codebase for existing functions and features and do not
  duplicate anything.
- Strictly follow DRY, KISS and YAGNI conceps.

# Tailwind CSS Generation

This project uses an implicit Dioxus CLI behavior for Tailwind CSS:

- The `dx` CLI looks for `tailwind.css` in the **project root**.
- If found, it compiles it using the internal Tailwind v4+ compiler.
- The output is saved to `assets/tailwind.css`.
- **DO NOT EDIT `assets/tailwind.css`**. Edit the root `tailwind.css` instead.
  You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant.
  Dioxus 0.7 changes every api in dioxus. Only use this up to date
  documentation. `cx`, `Scope`, and `use_state` are gone

Provide concise code examples with detailed descriptions

# Dioxus Dependency

You can add Dioxus to your `Cargo.toml` like this:

```toml
[dependencies]
dioxus = { version = "0.7.1" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

# Launching your application

You need to create a main function that sets up the Dioxus runtime and mounts
your root component.

```rust
use dioxus::prelude::*;

fn main() {
	dioxus::launch(App);
}

#[component]
fn App() -> Element {
	rsx! { "Hello, Dioxus!" }
}
```

Then serve with `dx serve`:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

# UI with RSX

```rust
rsx! {
	div {
		class: "container", // Attribute
		color: "red", // Inline styles
		width: if condition { "100%" }, // Conditional attributes
		"Hello, Dioxus!"
	}
	// Prefer loops over iterators
	for i in 0..5 {
		div { "{i}" } // use elements or components directly in loops
	}
	if condition {
		div { "Condition is true!" } // use elements or components directly in conditionals
	}

	{children} // Expressions are wrapped in brace
	{(0..5).map(|i| rsx! { span { "Item {i}" } })} // Iterators must be wrapped in braces
}
```

# Assets

The asset macro can be used to link to local files to use in your project. All
links start with `/` and are relative to the root of your project.

```rust
rsx! {
	img {
		src: asset!("/assets/image.png"),
		alt: "An image",
	}
}
```

## Styles

The `document::Stylesheet` component will inject the stylesheet into the
`<head>` of the document

```rust
rsx! {
	document::Stylesheet {
		href: asset!("/assets/styles.css"),
	}
}
```

> [!WARNING] When using Tailwind CSS, `assets/tailwind.css` is an
> **auto-generated file** created by the `dx serve` or `tailwind` CLI command.
> Do NOT edit it directly. Instead, edit `input.css` (or your configured input
> source) and let the CLI handle the rebuild.
>
> In many Dioxus projects, `assets/tailwind.css` is treated as a generated
> artifact. Always verify before editing.

# Components

Components are the building blocks of apps

- Component are functions annotated with the `#[component]` macro.
- The function name must start with a capital letter or contain an underscore.
- A component re-renders only under two conditions:
  1.  Its props change (as determined by `PartialEq`).
  2.  An internal reactive state it depends on is updated.

```rust
#[component]
fn Input(mut value: Signal<String>) -> Element {
	rsx! {
		input {
            value,
			oninput: move |e| {
				*value.write() = e.value();
			},
			onkeydown: move |e| {
				if e.key() == Key::Enter {
					value.write().clear();
				}
			},
		}
	}
}
```

Each component accepts function arguments (props)

- Props must be owned values, not references. Use `String` and `Vec<T>` instead
  of `&str` or `&[T]`.
- Props must implement `PartialEq` and `Clone`.
- To make props reactive and copy, you can wrap the type in `ReadOnlySignal`.
  Any reactive state like memos and resources that read `ReadOnlySignal` props
  will automatically re-run when the prop changes.

# State

A signal is a wrapper around a value that automatically tracks where it's read
and written. Changing a signal's value causes code that relies on the signal to
rerun.

## Local State

The `use_signal` hook creates state that is local to a single component. You can
call the signal like a function (e.g. `my_signal()`) to clone the value, or use
`.read()` to get a reference. `.write()` gets a mutable reference to the value.

Use `use_memo` to create a memoized value that recalculates when its
dependencies change. Memos are useful for expensive calculations that you don't
want to repeat unnecessarily.

```rust
#[component]
fn Counter() -> Element {
	let mut count = use_signal(|| 0);
	let mut doubled = use_memo(move || count() * 2); // doubled will re-run when count changes because it reads the signal

	rsx! {
		h1 { "Count: {count}" } // Counter will re-render when count changes because it reads the signal
		h2 { "Doubled: {doubled}" }
		button {
			onclick: move |_| *count.write() += 1, // Writing to the signal rerenders Counter
			"Increment"
		}
		button {
			onclick: move |_| count.with_mut(|count| *count += 1), // use with_mut to mutate the signal
			"Increment with with_mut"
		}
	}
}
```

### Passing Signals

In Dioxus 0.7, `Signal<T>` is a shallowly copyable handle with internal
mutability.

- **Pass by Value**: Always pass `Signal` by value to child components,
  controllers, and asynchronous closures.
- **Avoid Borrowing**: Do not use `&mut Signal<T>` or `&Signal<T>` for handles.
  Passing by value prevents complex borrow checker conflicts especially when
  multiple UI elements or tasks need to write to the same state simultaneously.

```rust
// ✅ Correct: Passing by value
pub fn update_name(mut store: Signal<AppStore>, name: String) {
    store.write().name = name;
}

// ❌ Avoid: Passing by reference
pub fn update_name(store: &mut Signal<AppStore>, name: String) { ... }
```

## Context API

The Context API allows you to share state down the component tree. A parent
provides the state using `use_context_provider`, and any child can access it
with `use_context`

```rust
#[component]
fn App() -> Element {
	let mut theme = use_signal(|| "light".to_string());
	use_context_provider(|| theme); // Provide a type to children
	rsx! { Child {} }
}

#[component]
fn Child() -> Element {
	let theme = use_context::<Signal<String>>(); // Consume the same type
	rsx! {
		div {
			"Current theme: {theme}"
		}
	}
}
```

# Async

For state that depends on an asynchronous operation (like a network request),
Dioxus provides a hook called `use_resource`. This hook manages the lifecycle of
the async task and provides the result to your component.

- The `use_resource` hook takes an `async` closure. It re-runs this closure
  whenever any signals it depends on (reads) are updated
- The `Resource` object returned can be in several states when read:

1. `None` if the resource is still loading
2. `Some(value)` if the resource has successfully loaded

```rust
let mut dog = use_resource(move || async move {
	// api request
});

match dog() {
	Some(dog_info) => rsx! { Dog { dog_info } },
	None => rsx! { "Loading..." },
}
```

# Routing

All possible routes are defined in a single Rust `enum` that derives `Routable`.
Each variant represents a route and is annotated with `#[route("/path")]`.
Dynamic Segments can capture parts of the URL path as parameters by using
`:name` in the route string. These become fields in the enum variant.

The `Router<Route> {}` component is the entry point that manages rendering the
correct component for the current URL.

You can use the `#[layout(NavBar)]` to create a layout shared between pages and
place an `Outlet<Route> {}` inside your layout component. The child routes will
be rendered in the outlet.

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
	#[layout(NavBar)] // This will use NavBar as the layout for all routes
		#[route("/")]
		Home {},
		#[route("/blog/:id")] // Dynamic segment
		BlogPost { id: i32 },
}

#[component]
fn NavBar() -> Element {
	rsx! {
		a { href: "/", "Home" }
		Outlet<Route> {} // Renders Home or BlogPost
	}
}

#[component]
fn App() -> Element {
	rsx! { Router::<Route> {} }
}
```

```toml
dioxus = { version = "0.7.1", features = ["router"] }
```

# Fullstack

Fullstack enables server rendering and ipc calls. It uses Cargo features
(`server` and a client feature like `web`) to split the code into a server and
client binaries.

```toml
dioxus = { version = "0.7.1", features = ["fullstack"] }
```

## Server Functions

Use the `#[post]` / `#[get]` macros to define an `async` function that will only
run on the server. On the server, this macro generates an API endpoint. On the
client, it generates a function that makes an HTTP request to that endpoint.

```rust
#[post("/api/double/:path/&query")]
async fn double_server(number: i32, path: String, query: i32) -> Result<i32, ServerFnError> {
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	Ok(number * 2)
}
```

## Hydration

Hydration is the process of making a server-rendered HTML page interactive on
the client. The server sends the initial HTML, and then the client-side runs,
attaches event listeners, and takes control of future rendering.

### Errors

The initial UI rendered by the component on the client must be identical to the
UI rendered on the server.

- Use the `use_server_future` hook instead of `use_resource`. It runs the future
  on the server, serializes the result, and sends it to the client, ensuring the
  client has the data immediately for its first render.
- Any code that relies on browser-specific APIs (like accessing `localStorage`)
  must be run _after_ hydration. Place this code inside a `use_effect` hook.

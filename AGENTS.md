See also @AGENTS_DIOXUS.md

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
    dependencies (e.g., `chrono`).

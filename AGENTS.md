# Development Guidelines

## Task Tracking

- **Use the CLI:** When task tracking is needed, ALWAYS use the `bd` command line tool for reading, creating, and updating tasks.
- **No Direct File Editing:** If you need to inspect or change tracked task state, use `bd` commands only; do not manually read or edit `.beads` database/storage files.
- **When Tracking Is Required:** Create or update `bd` tasks for deferred work, work expected to span sessions, or complex work with meaningful risk/scope.
- **When Tracking Is Not Required:** Do not create/update `bd` tasks for simple, self-contained work expected to be completed in a single session, unless explicitly requested.
- **Do Not Read Files:** NEVER attempt to parse or read files in the `.beads` directory directly. The file format is internal and subject to change.
- **Listing Tasks:** Use `bd ready` to see tasks ready for work. Use `bd list` to see all tasks.
- **Viewing Details:** Use `bd show <id>` to see task details.
- **Updating Tasks:** Use `bd update <id> <status>` to update task status.
- **Creating Tasks:** Use `bd create <title>` to create a new task.
- **Deleting Tasks:** Use `bd delete <id>` to delete a task.
- **Updating Task Details:** Use `bd update <id> <field> <value>` to update task details.

When working on tracked efforts, keep `bd` tasks accurate and record newly discovered deferred/complex follow-up work.

## Package Management

- Use `pnpm`, not `npm`.
- Use `pnpm dlx` for running scripts.
- Use `cargo` for Rust.
- Use `just` for running commands.
- **Dioxus Components Vendoring:** NEVER run `dx components add` directly in this repository. Always use `cargo xtask update-dioxus-components` so vendoring, pins, and branch/worktree workflow stay consistent.

## Environment Initialization

- **Dependency Installation:** Run `pnpm install` and `pnpm preflight`.
- **Clean Install:** If you need to clean the environment, use
  `scripts/aggressive-git-clean.sh` followed by `pnpm install` and `pnpm preflight`.

## Git Workflow

- **Clean Tree Rule:** Before starting unrelated work or a new development
  phase, run `git status`. If the working tree is not clean, STOP and notify the
  user.
- **Feature Branch Isolation Rule:** Every semantically independent task MUST be
  done on a different feature branch. If the current branch already contains
  unrelated work, STOP and create a new branch for the new task.
- **PR Isolation Rule:** Every PR MUST represent one cohesive concern. Reuse an
  existing branch/PR only for follow-up commits that are directly in scope for
  that PR (e.g., review feedback or missed tests for that same change).
- **New Task Branch Workflow:** For new independent work, branch directly from
  the upstream base branch to avoid mutating local base-branch history:
  ```bash
  git fetch origin
  git switch -c codex/<task-slug> --track origin/<base-branch>
  ```
- **PR Creation Scope Check:** Before pushing or creating/updating a PR, check
  whether the current branch already has an open PR:
  ```bash
  gh pr list --head <current-branch>
  ```
  If an open PR exists and your changes are semantically independent from that
  PR, DO NOT push to that branch. Create a new feature branch and open a new
  PR.
- **Git Commit Rule:** The Agent MAY commit changes autonomously when:
  1. Work is complete and all quality gates pass (`just verify` succeeds)
  2. Changes are logically cohesive and address a single concern
  3. The Agent clearly communicates what is being committed and why

  The Agent MUST NOT commit if:
  - Quality gates fail (tests, lints, builds)
  - Changes are incomplete or experimental
  - The user explicitly asks to review before committing

  **Communication Protocol:** Before committing, the Agent MUST:
  - Summarize what changed and why
  - Confirm quality gates passed
  - State the intended commit message
  - Then proceed with the commit unless the user intervenes

- **Git Presubmit Rule:** NEVER use `--no-verify`. On presubmit failure: fix
  trivial issues and retry; otherwise STOP AND WAIT.
- **Foreground Commit Rule:** ALWAYS run `git commit` in the foreground
  (synchronously). Presubmit hooks often fail or warn; immediate feedback is
  required to retry or fix issues promptly.
- **Verification Protocol:** The Agent must NEVER report a shell command as
  successful based on log output alone. The Agent MUST verify that the process
  `Exit Code` is `0` (via `command_status`) before claiming success.

## Coding Guidelines

### TypeScript Strictness

- **No `any`:** Use interfaces, generics, or utility types (`Pick`, `Omit`) instead.
- **No unsafe casting:** `as` and `!` are prohibited unless bridging external data with runtime validation (Zod/type guards).
- **Type errors = logic bugs:** Fix the implementation or data structures—never relax types to silence the compiler.
- **Narrow `unknown` immediately:** Only use `unknown` for truly dynamic runtime values; narrow via control flow.
- **Exhaust unions:** Handle all cases in `switch` statements; use `assertUnreachable` if needed.
- **Stop if stuck:** If you cannot express a type without `any` or `as`, ask the user for help—do not bypass the type system.

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

- All new code must have tests.
- **Verification Strategy:** While `just verify` is the gold standard for full verification, you MAY use your judgement to select the appropriate level of verification:
  - **Full Verification:** Run `just verify` for complex logic changes, refactors, or when touching critical paths.
  - **Standard Testing:** Run `just test` (or `just test-e2e`) for routine logic changes where static analysis is less likely to catch issues.
  - **Presubmit Reliance:** For documentation, formatting, or trivial changes, you MAY rely on the `git push` presubmit hooks (which run `just check`) rather than running verification commands manually.

### Test Commands

```bash
# All unit tests (monorepo-wide)
just test

# All E2E tests (monorepo-wide)
just test-e2e

# Scripts package unit tests (via just wrapper around pnpm test)
just test-scripts

# Pass through vitest args to scripts tests (note the `--`)
just test-scripts -- check-dependency-health.test.ts

# Pass through Playwright args to e2e tests (note the `--`)
just test-e2e -- tests/e2e/specs/due-dates.spec.ts --project=e2e-mobile

# Fully build everything and re-run all tests including e2e (monorepo-wide)
just verify
```

### Rust Validation

The `just verify` command includes Rust checks via `just check-rust`. You
can also run Rust checks independently:

```bash
# Full Rust validation (fmt, clippy, WASM build, dx build, tests)
just check-rust

# Individual checks during development
cargo fmt --check                                    # Formatting
cargo clippy --all-targets -- -D warnings           # Lints
cargo build --target wasm32-unknown-unknown -p tasklens-store  # WASM build
dx build -p tasklens-ui                             # Dioxus build
cargo test                                          # Tests
```

## Testing Strategy

For a detailed implementation guide, see
[docs/design/testing.md](docs/design/testing.md).

### AI Agent Instructions

- **Prefer `userEvent` over `fireEvent`**: Always use
  `@testing-library/user-event`. Use of `fireEvent` from
  `@testing-library/react` is strictly prohibited as it doesn't simulate real
  browser interactions and often leads to flaky tests in async environments.

### Executable Specs & Style Guide (Code-First Gherkin)

Tests are **Executable Specifications**. They should read as high-level
narratives using the project's **Ubiquitous Language**. We use a **Code-First
Gherkin** pattern where scenarios are written in TypeScript using a strictly
typed actor fixture (`I`).

**Ubiquitous Language Rule:** ALWAYS use domain terms (`Inbox`, `Plan`, `Do`,
`Balance`, `Context`). See `docs/design/prd.md` for the dictionary. Reject
implementation-level terms in test narratives.

## Testing Workflows

### Running Specific Tests

Always run pnpm-backed tests through `just` recipes. Do not invoke `pnpm test`
or `pnpm exec playwright test` directly.

To run only a specific scripts Vitest file:

```bash
just test-scripts -- check-dependency-health.test.ts
```

To run tests matching a Vitest name pattern:

```bash
just test-scripts -- check-dependency-health.test.ts -t "distribution"
```

To run a targeted Playwright spec/project:

```bash
just test-e2e -- tests/e2e/specs/due-dates.spec.ts --project=e2e-mobile
```

# Learnings

## E2E & BDD Strategy (Playwright)

- **Semantic Selectors**: Avoid checking CSS styles or fragile labels. Use stable data attributes (e.g., `data-testid`, `data-urgency`).
- **Timezone Pitfalls**: `page.clock.setFixedTime()` sets system time (often UTC), but `new Date()` uses browser timezone. Align them or use ISO strings.
- **Declarative Steps**: Prefer high-level domain actions (`I.Given.cleanWorkspace()`) over implementation details.
- **Case-Sensitive BDD**: Steps often expect TitleCase for enums (e.g., `Urgent`). Check `STATUS_MAP` in `all.steps.ts`.
- **WASM Init Race**: Use `page.waitForFunction` to ensure custom WASM APIs are attached to `window` before calling them.
- **Focus Traps**: Avoid `expect(locator).toBeFocused()` in dialogs with focus traps; directly interact with inputs instead.
- **Dialog Stacking**: Ensure dialogs explicitly close and unmount to avoid backdrop occlusion or focus theft.
- **Debugging**:
  - **Console Relays**: Use `page.on("console", ...)` in the Page Object to forward browser logs to the Node process.
  - **Dioxus Toast "Parking"**: Dioxus parks toasts off-screen (`right: -1000px`) instead of `display: none`. They are "visible" to Playwright but may occlude interactions.
- **Isolation & Persistence**:
  - **Worker Collision**: Disable `fullyParallel` if using stateful `IndexedDB` to avoid origin collisions.
  - **Fixture Isolation**: Use custom fixtures (`alice`, `bob`) instead of module-level variables to prevent state leakage between tests.
  - **Reload Race**: `repo.import()` followed by `location.reload()` can race. Add a settle delay or await persistence confirmation.
- **IndexedDB Cleanup**: `indexedDB.deleteDatabase()` is async; wrap in a Promise awaiting `onsuccess`.
- **LocalStorage/Gloo**: `gloo-storage` JSON-encodes values (e.g., `"\"automerge:123\""`). `JSON.parse()` before asserting.

## Automerge & Autosurgeon Patterns

- **Asymmetric Serialization**: Use `Hydrate` (broad, accepts Text or Scalar) and `Reconcile` (strict, writes legacy format) asymmetrically when bridging old schemas. Use `reconciler.text()` to force `Text` type for IDs.
- **`hydrate_prop` vs `MaybeMissing`**: `hydrate_prop::<Option<T>>` fails if a key is missing. Use `hydrate_prop::<MaybeMissing<T>>` for optional keys that might be absent from the Automerge map.
- **Numeric Type Strictness**: Automerge documents fluctuate between `ScalarValue::Int` and `F64`. Treat them as equal if mathematical values match to avoid diff noise.
- **Realized Text**: `assert_doc!` realizes `Text` as a list of character maps. Use an `am_text` helper to convert strings for cleaner assertions.

## Rust & Dioxus Development

- **Workspace Management**: Use a root `Cargo.toml` with `[workspace]`. Isolate Rust crates in `crates/`. Centralize shared dependencies in `[workspace.dependencies]`.
- **WASM Gating**: Use `[target.'cfg(target_arch = "wasm32")'.dependencies]` for web-only crates. Provide mock/no-op implementations for native targets to keep `cargo check` passing on host.
- **Tree-Walking & Clippy**: Deep recursion triggers clippy warnings. Use a `FlattenContext` struct to group stateful references and reduce function signature size.
- **Inheritance vs. Aggregation**: Handle scheduling inheritance (due dates) during tree-walking in the view layer, passing "effective" values to keep leaf components stateless.
- **Dioxus State & Context**:
  - **Hydration Race**: Initialize store with default/empty state synchronously in `use_context_provider` to avoid "unexpected None" errors. Hydrate asynchronously from persistence later.
  - **Async Locking**: Never hold a `Signal` or `RefCell` write lock across an `.await`.
  - **Infinite Loops**: `AutoCommit::get_heads` requires `&mut self`, triggering signal writes. Avoid using it in reactive effects without explicit change checks.
- **Async Testing**: Use `pool.run_until()` with `futures::executor::LocalPool`. `pollster::block_on` does not poll background tasks spawned on the spawner.
- **WASM Debugging**:
  - Use `console_error_panic_hook` for readable panics.
  - Avoid `std::time` on `wasm32-unknown-unknown`; use `web-time` or enable `js` features on crates like `chrono`.
  - Prefer `expect("message")` over `unwrap()` for better trace context in WASM logs.
- **Numeric Interop**: Use `f64` for ALL numeric fields crossing the FFI/Automerge boundary to ensure IEEE 754 compatibility with JavaScript.
- **Dioxus Router**: Custom URL parameter types must implement `Display` and `FromStr`.

## Project Management

- **Milestone Granularity**: Avoid "View"-based milestones which often hide complex mutation/component dependencies. Break by **Capability** (e.g., "Data Rendering", "Status Toggling") to keep changesets reviewable.
- **Migration Source of Truth**: Port types from the **Persistence/Zod** layer, not the BDD/Test layer, to ensure storage compatibility.
- **Workflow Authority**: `docs/plan/rust_migration.md` is the source of truth for the migration. Do not create ephemeral implementation plans.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

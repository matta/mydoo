# Tasks: Moonrepo Migration

**Feature**: `003-moonrepo-migration`
**Status**: Completed

## Implementation Strategy

The migration was performed in a "safe overlay" manner. We configured Moonrepo alongside Turbo initially, allowing for verification of Moon commands without breaking the existing build. Once Moon was verified to reproduce all Turbo capabilities (build, test, check), we switched the CI and root scripts, and finally removed Turbo.

- **Phase 1**: Initialize Moonrepo and pin toolchain versions.
- **Phase 2**: Define global task inheritance (lint, types) to match Turbo's root-level config.
- **Phase 3**: Configure project-specific tasks and update root `package.json` scripts to delegate to Moon.
- **Phase 4**: Migrate CI pipeline to use `moon ci`.
- **Phase 5**: Re-implement complex presubmit checks.
- **Phase 6**: Clean up Turbo configuration.

## Dependencies

- Phase 1 (Setup) -> Phase 2 (Global Config)
- Phase 2 (Global Config) -> Phase 3 (Developer Workflow)
- Phase 3 (Developer Workflow) -> Phase 4 (CI Pipeline)
- Phase 3 (Developer Workflow) -> Phase 5 (Presubmit)
- Phase 4 & 5 -> Phase 6 (Cleanup)

## Phase 1: Setup (Project Initialization)

**Goal**: Initialize Moonrepo structure and pin the exact Node.js/pnpm toolchain versions to match the current environment.

- [X] T001 Initialize `.moon` configuration directory structure
- [X] T002 [P] Create `.prototools` to pin moon version (v1.32.2 or latest stable)
- [X] T003 Configure `.moon/workspace.yml` with project discovery (apps/*, packages/*), git settings, and file exclusions (FR-005)
- [X] T004 Configure `.moon/toolchain.yml` to strictly pin Node.js 24 and pnpm 10.26.1
- [X] T004a Configure global pass-through environment variables (ANTIGRAVITY_AGENT, GEMINI_CLI, CI) in `.moon/tasks.yml` (FR-006)

## Phase 2: Foundational (Global Configuration)

**Goal**: Establish the global task graph that mirrors `turbo.json`'s pipeline, enabling inheritance for standard tasks.

- [X] T005 Define global `lint` task (biome check) in `.moon/tasks.yml`
- [X] T006 [P] Define global `format` task (biome format) in `.moon/tasks.yml`
- [X] T007 [P] Define global `typecheck` task (tsc) in `.moon/tasks.yml`
- [X] T008 Configure root-level project by adding a `moon.yml` in the repository root ("root-project")
- [X] T009 Define root-only tasks (`check-deps`, `check-filenames`) in the root `moon.yml`

## Phase 3: Developer Standard Workflow (User Story 1)

**Goal**: Enable developers to run standard commands (`pnpm build`, `pnpm test`) via Moon, transparently replacing the internal Turbo engine.

**Independent Test**: `moon run :build` produces identical output to `turbo run build`.

- [X] T010 [US1] Create `apps/client/moon.yml` mapping `build`, `test`, `test-e2e` tasks
- [X] T011 [P] [US1] Create `packages/tasklens/moon.yml` mapping `build`, `test` tasks
- [X] T012 [US1] Verify `check-types` inheritance works for all projects via `moon run :typecheck`
- [X] T013 [US1] Update `package.json` script `build` to run `moon run :build`
- [X] T014 [US1] Update `package.json` script `test` to run `moon run :test`
- [X] T015 [US1] Update `package.json` script `check` to run `moon run :check` (aliasing the group of check tasks)
- [X] T016 [US1] Update `package.json` script `fix` to run `moon run :format` and `moon run :lint -- --write`

## Phase 4: CI Pipeline Integrity (User Story 2)

**Goal**: Migrate GitHub Actions to use Moonrepo's managed environment and caching, ensuring PR verification remains rigorous.

**Independent Test**: CI job succeeds on a new branch with `moon ci`.

- [X] T017 [US2] Update `.github/workflows/ci.yml` to replace `setup-node`/`pnpm-action` with `moonrepo/setup-moon-action`
- [X] T018 [US2] Update `.github/workflows/ci.yml` to replace manual `actions/cache` for turbo with moon's caching strategy (or configured native cache)
- [X] T019 [US2] Replace explicit `pnpm` commands in CI steps with a single `moon ci` (or `moon run ...` sequence) invocation

## Phase 5: Presubmit Verification (User Story 3)

**Goal**: Re-implement the "Safe Commit" protocols (`presubmit:agent`) using Moon's task runner capabilities to execute the full dependency graph.

**Independent Test**: `pnpm presubmit:agent` runs all required checks including staged file validation.

- [X] T020 [US3] Create a composite task or alias in `.moon/tasks.yml` for `presubmit:agent`
- [X] T021 [US3] Ensure `presubmit:agent` depends on `lint`, `typecheck`, `test`, `test-e2e`, and root checks
- [X] T022 [US3] Verify `check-filenames` and `check-context` scripts run correctly under Moon invocation
- [X] T023 [US3] Update `package.json` script `presubmit:agent` to invoke the new Moon alias

## Phase 6: Polish & Cleanup

**Goal**: Remove legacy Turbo configuration and finalize the migration.

- [X] T024 Remove `turbo.json` file
- [X] T025 Remove `turbo` dependency from `package.json`
- [X] T026 Verify all `pnpm` scripts function without Turbo
- [X] T027 [P] Update `CONTRIBUTING.md` or developer docs to reference Moonrepo commands (if specific docs exist)
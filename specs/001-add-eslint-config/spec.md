# Feature Specification: Add ESLint Configuration

**Feature Branch**: `001-add-eslint-config`
**Created**: 2025-12-30
**Status**: Draft
**Input**: User description: "add eslint configurations to this repository... keep the config simple... disable all eslint warnings but for the eslint-plugin-import warning import/no-namespace... Integrate this into the turbo build properly... Running 'pnpm lint' twice in a row from the repo root must show the second eslint run as cached..."

## Clarifications

### Session 2025-12-30
- Q: ESLint Configuration Format → A: Flat Config (`eslint.config.js`)
- Q: ESLint Version Constraint → A: Latest Stable (v9.x)
- Q: Linting File Scope → A: TypeScript & JavaScript (.ts, .tsx, .js, .jsx)
- Q: Base Configuration Strategy → A: Minimalist (No base config, explicitly enable ONLY `import/no-namespace`)
- Q: Exit Code for Warnings → A: Exit 0 (Success)
- Q: Linting Orchestration Strategy → A: Turbo delegates to per-package `lint` scripts
- Q: Turbo Output Caching → A: No file outputs, only cache logs
- Q: Global vs. Local ESLint Dependency → A: Install ESLint/Plugins only in the repository root
- Q: Root `pnpm lint` Command Implementation → A: `pnpm lint` in root runs `turbo run lint`
- Q: Configuration of Include/Exclude Patterns → A: Centralized in root `eslint.config.js`

### Session 2025-12-30 (Part 2)
- Q: Lint Script Composition → A: Top-level `lint` runs `turbo` tasks (`lint-root`, `lint-eslint`) in parallel
- Q: Turbo Task Definition for ESLint → A: `lint` task aggregates independent tasks (`lint-eslint`, `lint-biome`, etc.) via `dependsOn`
- Q: Existing Linter Migration → A: Rename existing package `lint` scripts to `lint-<tool>` (e.g., `lint-biome`)
- Q: Individual Top-Level Aliases → A: Add `lint-eslint`, `lint-biome`, etc. aliases to root `package.json`
- Q: Caching Grain for `eslint.config.js` → A: Root `eslint.config.js` is a global input for all `lint-eslint` tasks

### Session 2025-12-30 (Part 3)
- Q: `lint-root` and `turbo lint` Aggregation → A: Aggregation via `turbo.json` (lint depends on `//#lint-root`, etc.)
- Q: ESLint Integration with Staged Linting → A: Update root `lint-staged` to include ESLint check

### Session 2025-12-30 (Part 4)
- Q: Naming Convention for Staged Lint Tasks → A: `lint-<tool>-staged` (e.g., `lint-biome-staged`)
- Q: Orchestration of Staged Linting → A: Sequential execution in root `lint-staged` script
- Q: Handling ROLLING_CONTEXT.md in Staged Checks → A: Rename `check-staged` to `check-context-staged`
- Q: Filename Check in Staged Workflow → A: Create `lint-filenames-staged` for staged files
- Q: Scope of Linter Scaffolding → A: ESLint only (plus orchestration infrastructure)

### Session 2025-12-30 (Part 5)
- Q: Confirm Task Renaming to `kebab-case` → A: Rename all tasks to hyphenated format (e.g., `lint-biome`, `lint-eslint-staged`) per `docs/guidance/task-naming.md`
- Q: Turbo Task Topology with Aggregates → A: Use `lint` as the top-level aggregate task name

### Session 2025-12-30 (Part 6)
- Q: Renaming the `pre-commit` task → A: `lint-staged` (Canonical name for pre-commit aggregate check)
- Q: Script renaming for `lint-deps` (Knip) → A: `lint-deps-check` (Action: lint, Subject: deps, Mode: check)
- Q: Renaming the `typecheck` task → A: `check-types` (Action: check, Subject: types)

### Session 2025-12-30 (Part 7)
- Q: Primary Task Renaming → A: `check-style` (Replacing `lint` per new guidance synonyms ban)
- Q: Staged Check Aggregate Renaming → A: `check-staged` (Action: check, State: staged)
- Q: Tool-Specific Staged Check Renaming → A: `check-eslint-staged`, `check-biome-staged`, etc.

### Session 2025-12-30 (Part 8)
- Q: Naming the Staged Aggregate Task → A: `check-staged` (Aligned with Safety Rule for read-only validation)
- Q: Domain Aggregate Naming for Style Checking → A: Use `check-style` as the aggregate for ESLint and Biome

### Session 2025-12-30 (Part 9)
- Q: Scope of `check` and `fix` Aggregates → A: `check` = All static analysis (Style + Types + Deps); `fix` = Autofixers
- Q: Parallelism for `check` and `fix` → A: `check` (parallel); `fix` (sequential)

### Session 2025-12-30 (Part 10)
- Q: Scope of Caching Verification in Test Protocols → A: Verify caching for all major aggregate commands (`check`, `check-style`, `fix`)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer runs linting locally (Priority: P1)

A developer wants to check their code for specific style violations (namespace imports) to ensure adherence to project standards without being overwhelmed by other lint noise.

**Why this priority**: Core functionality request.

**Independent Test**: Run `pnpm check-style` in the root and verify output.

**Acceptance Scenarios**:

1. **Given** a codebase with `import * as namespace` statements, **When** the developer runs `pnpm check-style`, **Then** the output reports `import/no-namespace` warnings for those lines.
2. **Given** a codebase with other common lint violations (e.g. unused vars), **When** the developer runs `pnpm check-style`, **Then** the output DOES NOT report errors or warnings for those lines.
3. **Given** the current existing code, **When** the developer runs `pnpm check-style`, **Then** the command completes without modifying any logic (no auto-fixes applied).

---

### User Story 2 - Turbo Build Integration & Caching (Priority: P1)

The build system needs to execute linting efficiently, utilizing caching to prevent redundant work in local development and CI.

**Why this priority**: Required for efficient developer workflow in a monorepo.

**Independent Test**: Run `pnpm check-style`, then run it again immediately.

**Acceptance Scenarios**:

1. **Given** a fresh state, **When** `pnpm check-style` is run successfully, **Then** Turbo executes the task.
2. **Given** a successful previous check run, **When** `pnpm check-style`, `pnpm check`, or `pnpm fix` (if idempotent) is run again without code changes, **Then** Turbo reports the task as `FULL TURBO` (cached) and executes nearly instantly.
3. **Given** a new package is added, **When** it extends the root configuration, **Then** it inherits the unified rules.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a unified ESLint configuration file at the repository root using the **Flat Config format (`eslint.config.js`)** and **ESLint v9.x**.
- **FR-002**: System MUST NOT use separate configuration files for different environments (test, dev, prod) or file types.
- **FR-003**: System MUST NOT use a base configuration (like `eslint:recommended`). It MUST explicitly enable ONLY the targeted rule to ensure a "clean" lint run.
- **FR-004**: System MUST enable the `eslint-plugin-import` rule `import/no-namespace`.
- **FR-005**: The `import/no-namespace` rule MUST be configured as a **warning** (not an error).
- **FR-006**: The style checking task MUST be integrated into the `turbo.json` pipeline.
- **FR-007**: The style checking task MUST be configured for caching in Turbo (inputs/outputs defined correctly).
- **FR-008**: Individual packages in the monorepo MUST extend the root configuration to ensure consistency.
- **FR-009**: The implementation MUST NOT automatically fix or modify existing code causing style warnings.
- **FR-010**: The style checking task MUST target TypeScript and JavaScript files (`.ts, .tsx, .js, .jsx`).
- **FR-011**: The style checking process MUST exit with code 0 when only warnings are present.
- **FR-012**: Turbo MUST execute style checking by delegating to individual `check-<tool>` scripts in each package's `package.json`, ensuring granular caching per tool and package.
- **FR-013**: The `check-style` task in `turbo.json` MUST be configured with an empty `outputs` array to indicate that only console output should be cached.
- **FR-014**: ESLint and the `eslint-plugin-import-x` (for v9 compatibility) MUST be installed as development dependencies in the repository root (workspace root).
- **FR-015**: The root `package.json` MUST contain a `check-style` script that executes `turbo run check-style`.
- **FR-016**: Common ignore patterns (e.g., `dist`, `node_modules`, build artifacts) MUST be centralized in the root `eslint.config.js`.
- **FR-017**: The root `check-style` command MUST trigger parallel execution of all configured style checkers via distinct Turbo tasks (e.g., `check-eslint`, `check-biome`).
- **FR-018**: Each package MUST have a `check-eslint` script that runs `eslint .` (or specific target) to be orchestratable by Turbo.
- **FR-019**: The `turbo.json` `check-style` task MUST utilize `dependsOn` to aggregate all project-wide style checking tasks (e.g., `check-eslint`, `check-biome`, `//#check-style-root`, `//#check-filenames`, `//#check-deps`) rather than running a command itself.
- **FR-020**: All existing per-package scripts MUST be renamed to follow the `[action]-[subject]-[state]` pattern (e.g., `lint` → `check-biome`) using kebab-case with no colons and no banned terms like `lint`.
- **FR-021**: The root `package.json` MUST provide alias scripts for each specific style checker (e.g., `"check-eslint": "turbo run check-eslint"`) to allow for global execution of individual tools.
- **FR-022**: The root `eslint.config.js` MUST be defined as a global input for all `check-eslint` tasks in `turbo.json` to ensure configuration changes invalidate the cache across all packages.
- **FR-023**: The root `check-staged` script MUST be updated to include an ESLint check on staged files alongside Biome; this script acts as the primary gate for all staged quality checks.
- **FR-024**: The root `package.json` MUST define distinct scripts for each staged check following the canonical hierarchy (e.g., `check-biome-staged`, `check-eslint-staged`, `check-filenames-staged`).
- **FR-025**: Staged style checking tasks MUST be executed sequentially in the root `check-staged` script to ensure reliable execution on transient git state.
- **FR-026**: The existing `check-staged` script MUST be renamed to `check-context-staged` to follow the project naming guidance.
- **FR-027**: The root `check-filenames-staged` script MUST be created to run the filename convention check on staged files.
- **FR-028**: All project documentation (e.g., `AGENTS.md`, `docs/guidance/*.md`), CI workflows (e.g., `.github/workflows/ci.yml`), IDE configurations (e.g., `.vscode/settings.json`), and scripts referencing quality gates MUST be updated to match the new `kebab-case` segment-based task names (e.g., replacing `lint` with `check-style`, `pre-commit` with `check-staged`, `typecheck` with `check-types`, `lint:deps` with `check-deps`). The `pre-commit` script MUST be removed.
- **FR-029**: System MUST provide top-level `check` and `fix` scripts in the root `package.json` and in each package's `package.json`.
- **FR-030**: The top-level `check` command MUST aggregate and execute all read-only validation tasks (e.g., `check-style`, `check-types`, `check-deps`) in parallel.
- **FR-031**: The top-level `fix` command MUST aggregate and execute all mutation tasks (e.g., `fix-style`, `fix-format`) sequentially to avoid race conditions.
- **FR-032**: Test protocols MUST verify Turbo caching functionality by executing major aggregate commands (`check`, `check-style`, `fix`) twice in succession and confirming the second run is cached (`FULL TURBO`).

### Edge Cases

- **New Packages**: New packages added to the monorepo should easily adopt the standard config.
- **Plugin Dependencies**: Ensuring `eslint-plugin-import-x` is available to all packages consuming the config.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `pnpm check-style` execution on the entire repo completes successfully (exit code 0 if only warnings present).
- **SC-002**: Consecutive execution of major aggregate commands (`check`, `check-style`, `fix`) without changes takes significantly less time (e.g., <1 second) due to Turbo caching.
- **SC-003**: Output contains strictly `import/no-namespace` warnings and zero other style violations.
- **SC-004**: No file content changes (diff) are observed in source code files after running the check-style command.

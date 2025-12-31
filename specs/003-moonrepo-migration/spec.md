# Feature Specification: Moonrepo Migration

**Feature Branch**: `003-moonrepo-migration`  
**Created**: Tuesday, December 30, 2025  
**Status**: Draft  
**Input**: User description: "moonrepo build system: I want to switch from turbo to moonrepo (https://moonrepo.dev/). The migration must preserve all existing characteristics of the turbo config (same command set, same dependencies, same file exclusions, same pnpm script names, etc.). the @ci.yml must be updated to work with it as well."

## Clarifications

### Session 2025-12-30
- Q: Should moonrepo manage the Node.js/pnpm toolchain? → A: Yes, moonrepo will manage Node.js and pnpm versions to ensure consistency.
- Q: How should CI artifacts be cached? → A: Use GitHub Actions Cache (via standard actions) to store moon artifacts.
- Q: Should the .moon configuration directory be committed to git? → A: Yes, all moon configuration files (.moon/*.yml) will be tracked in version control.
- Q: Should the moon CLI version be pinned? → A: Yes, the moon version will be pinned (e.g., via .prototools) to ensure toolchain stability.
- Q: Should turbo files be removed immediately? → A: Yes, turbo.json and the turbo dependency will be removed as part of the migration.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Standard Workflow (Priority: P1)

As a developer, I want to execute standard lifecycle commands (build, test, lint) using the same CLI commands I currently use, so that my workflow remains uninterrupted while the underlying engine switches to moonrepo.

**Why this priority**: Minimizing disruption to the development team is the primary constraint of this migration.

**Independent Test**: Execute `pnpm check`, `pnpm build`, and `pnpm test` in a fresh environment.

**Acceptance Scenarios**:

1. **Given** a clean repository state, **When** I run `pnpm build`, **Then** all project artifacts are generated successfully without error.
2. **Given** a clean repository state, **When** I run `pnpm check`, **Then** the full suite of linters, type checkers, and dependency checks runs and passes.
3. **Given** a repository with changes, **When** I run `pnpm fix`, **Then** the code is auto-formatted and fixed using the same rules as before.

---

### User Story 2 - CI Pipeline Integrity (Priority: P1)

As a CI system (GitHub Actions), I want to execute the validation pipeline using the new moonrepo configuration, so that pull requests are verified with the same rigor as before.

**Why this priority**: Ensuring code quality gates remain active and effective is critical for project health.

**Independent Test**: Create a PR and observe the GitHub Actions "CI" workflow.

**Acceptance Scenarios**:

1. **Given** a new Pull Request, **When** the CI workflow triggers, **Then** it executes the build, test, and lint steps using moonrepo.
2. **Given** a cached CI run, **When** the CI workflow triggers again on the same commit, **Then** it utilizes caching to reduce execution time (if applicable/configured).

---

### User Story 3 - Presubmit Verification (Priority: P2)

As an AI Agent or Developer, I want to run the rigorous `presubmit:agent` and `presubmit:human` targets, ensuring all checks (including staged-file checks) pass before committing.

**Why this priority**: These custom composite tasks are essential for the project's "Safe Commit" protocol.

**Independent Test**: Run `pnpm presubmit:agent`.

**Acceptance Scenarios**:

1. **Given** I am ready to commit, **When** I run `pnpm presubmit:agent`, **Then** it executes the full dependency graph of checks (biome, eslint, filenames, context, types, tests) as defined in the original configuration.

### Edge Cases

- **Developer missing binary**: What happens if the developer does not have the `moon` binary installed globally?
    - *Expectation*: The project relies on `proto` and moon's managed toolchain. The setup scripts or `pnpm` wrappers must ensure the toolchain is bootstrapped automatically, avoiding manual global install requirements.
- **Cache invalidation**: How does the system handle the transition of cache artifacts?
    - *Expectation*: Old turbo caches will be ignored; new moonrepo caches will be generated. First run will be slower.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST replace the `turbo` build system with `moonrepo` for all task orchestration.
- **FR-002**: The system MUST preserve the behavior of all existing `pnpm` script commands (e.g., `build`, `test`, `check`, `fix`, `presubmit:agent`).
- **FR-003**: The system MUST implement a moonrepo configuration that mirrors the dependency graph, inputs, outputs, and caching behavior of the existing `turbo.json`.
    - *Note*: This includes complex root-level tasks like `//#check-deps` and `//#check-filenames`.
- **FR-004**: The system MUST update `.github/workflows/ci.yml` to utilize moonrepo for the `ci` job, replacing turbo invocations.
- **FR-005**: The system MUST respect existing file exclusions (ignore patterns) when calculating file hashes for caching.
- **FR-006**: The system MUST support "pass-through" environment variables (e.g., `ANTIGRAVITY_AGENT`, `GEMINI_CLI`, `CI`) as currently configured.
- **FR-007**: The system MUST configure moonrepo's toolchain to explicitly manage Node.js and pnpm versions, ensuring identical environments across all development and CI machines.
- **FR-008**: The system MUST configure GitHub Actions caching in `ci.yml` to persist moonrepo artifacts across runs, using the native GH Actions cache service.
- **FR-009**: The system MUST commit all moonrepo configuration files (typically in `.moon/` directory) to version control to ensure shared build definitions across the team.
- **FR-010**: The system MUST pin the version of the moon CLI (e.g., via `.prototools`) to ensure a consistent execution environment for the build orchestrator itself.
- **FR-011**: The system MUST remove the `turbo.json` file and the `turbo` dependency from `package.json` as part of the migration.

### Key Entities

- **Task Configuration**: The definition of tasks (inputs, outputs, commands, dependencies) currently residing in `turbo.json`, to be migrated to moonrepo's configuration format.
- **CI Workflow**: The definition of the automated build pipeline.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of existing `pnpm` root scripts execute successfully and produce the expected exit codes (0 for success, non-0 for failure).
- **SC-002**: The CI pipeline (GitHub Actions) completes successfully (Green) for a known-good commit.
- **SC-003**: The `moonrepo` configuration covers 100% of the tasks previously defined in `turbo.json`.
- **SC-004**: No existing `pnpm` scripts are removed or renamed.

# Specification: Remove Turborepo and Migrate to Just

## Overview
This track involves decommissioning Turborepo as the monorepo orchestrator. All build logic, validation pipelines, and task dependencies will be migrated to the root `justfile` and idiomatic `pnpm` scripts. The goal is to simplify the toolchain, favor sequential execution for reliability, and centralize control in `just`.

## Functional Requirements
- **Toolchain Cleanup:** Remove `turbo.json`, `turbo-exclusions.json`, and the `turbo` dependency from the root `package.json`.
- **Just Orchestration:**
    - Re-implement all core tasks (`build`, `test`, `test-e2e`, `check`, `fix`, `generate`, `check-agent`, `verify`) as recipes in the root `justfile`.
    - Use `just` dependency syntax (e.g., `test: build check-types`) to enforce execution order.
    - Explicitly list package paths in recipes (e.g., `cd crates/tasklens-ui && ...`) instead of relying on `pnpm -r` or dynamic discovery to maintain precise control.
- **Environment Variables:** Adopt an **implicit inheritance** strategy. Commands within recipes will rely on the existing shell environment (e.g., `FEATURE_FILTER`, `CI`) without explicit re-declaration in `just`.
- **Workflow Preservation:**
    - The `check-agent` and `verify` workflows must be preserved as sequential "meta-recipes" that call other recipes in order.
    - `check-agent` depends on `check-human` (validation) -> `test` -> `test-e2e`.
    - `verify` runs `check-rust` -> `fix` -> `check-agent`.
- **Package Scripts:** Update `package.json` scripts to act as thin wrappers around `just` recipes where appropriate, ensuring standard commands like `pnpm build` still function.

## Non-Functional Requirements
- **Sequential Execution:** Favor simplicity and debuggability. Parallel execution is out of scope for this migration.
- **Idiomatic Usage:** Use `pnpm` only for JS/TS package management (e.g., `pnpm install`, `pnpm exec`) and `just` for all cross-package orchestration.

## Acceptance Criteria
- `turbo.json` is deleted and `pnpm-lock.yaml` no longer contains Turborepo.
- `just verify` successfully runs linting, type-checking, and tests for all packages in the correct sequence.
- All existing `package.json` scripts are either removed (if redundant) or successfully redirect to `just` equivalents.
- Task dependencies (e.g., building before testing) are verified to work correctly via `just`.

## Out of Scope
- **Task Caching:** Turborepo's local and remote caching will not be replaced.
- **Dynamic Package Discovery:** We will use hardcoded paths in the `justfile` as requested.
- **Generating Code:** The `generate` pipeline was found to be dead and will be removed, not migrated.

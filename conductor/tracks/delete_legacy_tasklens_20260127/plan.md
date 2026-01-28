# Implementation Plan - Delete Legacy TypeScript TaskLens Package

## Phase 1: Configuration Decoupling [checkpoint: 8272f85]
Decouple the `packages/tasklens` package from the monorepo workspace and build configurations to prevent build errors during deletion.

- [x] Task: Remove `packages/tasklens` from `pnpm-workspace.yaml`. efc6682
    - [x] Remove the entry from the `packages` list.
- [x] Task: Remove project references in `crates/tasklens-ui`. 889dec2
    - [x] Edit `crates/tasklens-ui/tsconfig.json` to remove the reference to `../../packages/tasklens`.
- [x] Task: Scan and remove dependencies in other packages. 889dec2
    - [x] Check `apps/client/package.json` and remove any dependency on `@tasklens/core` (or equivalent).
    - [x] Check root `package.json` (if applicable).
- [x] Task: Clean up `turbo.json`. 889dec2
    - [x] Remove any pipeline configurations specifically targeting the deleted package.
- [x] Task: Conductor - User Manual Verification 'Configuration Decoupling' (Protocol in workflow.md) 8272f85

## Phase 2: Source Removal
Delete the TypeScript source code and package configuration while strictly preserving the test fixtures.

- [x] Task: Delete source and test directories. 0bfa08e
    - [x] Delete `packages/tasklens/src/`.
    - [x] Delete `packages/tasklens/tests/`.
- [x] Task: Delete package configuration files. 0bfa08e
    - [x] Delete `packages/tasklens/package.json`.
    - [x] Delete `packages/tasklens/tsconfig.json`.
    - [x] Delete `packages/tasklens/turbo.json`.
    - [x] Delete `packages/tasklens/vitest.config.ts`.
    - [x] Delete any other root-level files in `packages/tasklens` EXCEPT the `specs` directory.
- [x] Task: Verify Fixture Preservation. 0bfa08e
    - [x] Confirm `packages/tasklens/specs/compliance/fixtures/` exists and contains YAML files.
- [x] Task: Conductor - User Manual Verification 'Source Removal' (Protocol in workflow.md) 0bfa08e

## Phase 3: Dependency & Tooling Cleanup
Remove unused dependencies and update project-wide tooling configurations.

- [x] Task: Update Knip configuration. 0bfa08e
    - [x] Check `knip.json` and remove `packages/tasklens` from entry points or workspaces.
- [x] Task: Update ESLint/Prettier ignores. 0bfa08e
    - [x] Remove `packages/tasklens` from `.eslintignore` or `.prettierignore` if explicitly listed (optional, as the files are gone).
- [x] Task: Prune unused dependencies. 0bfa08e
    - [x] Run `pnpm install` to update the lockfile.
    - [x] Identify and remove dependencies that were *only* used by `packages/tasklens` (e.g., specific dev dependencies) from the root `package.json` if they were hoisted, or rely on `knip` to identify them later.
- [x] Task: Conductor - User Manual Verification 'Dependency & Tooling Cleanup' (Protocol in workflow.md) 0bfa08e

## Phase 4: System Verification
Verify that the monorepo is healthy and the Rust crate can still access the fixtures.

- [ ] Task: Clean install.
    - [ ] Run `pnpm install` to update the lockfile and verify workspace integrity.
- [ ] Task: Verify Monorepo Health.
    - [ ] Run `pnpm check` (which includes knip).
    - [ ] Run `pnpm fix`.
- [ ] Task: Verify Full Build & Test Suite.
    - [ ] Run `pnpm verify` to ensure no regressions in other packages.
- [ ] Task: Verify Rust Compliance Tests.
    - [ ] Run `cargo test -p tasklens-core` to ensure it can still read the fixtures.
- [ ] Task: Conductor - User Manual Verification 'System Verification' (Protocol in workflow.md)
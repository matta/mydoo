# Implementation Plan - Delete Legacy TypeScript TaskLens Package

## Phase 1: Configuration Decoupling
Decouple the `packages/tasklens` package from the monorepo workspace and build configurations to prevent build errors during deletion.

- [x] Task: Remove `packages/tasklens` from `pnpm-workspace.yaml`. efc6682
    - [x] Remove the entry from the `packages` list.
- [ ] Task: Remove project references in `crates/tasklens-ui`.
    - [ ] Edit `crates/tasklens-ui/tsconfig.json` to remove the reference to `../../packages/tasklens`.
- [ ] Task: Scan and remove dependencies in other packages.
    - [ ] Check `apps/client/package.json` and remove any dependency on `@tasklens/core` (or equivalent).
    - [ ] Check root `package.json` (if applicable).
- [ ] Task: Clean up `turbo.json`.
    - [ ] Remove any pipeline configurations specifically targeting the deleted package.
- [ ] Task: Conductor - User Manual Verification 'Configuration Decoupling' (Protocol in workflow.md)

## Phase 2: Source Removal
Delete the TypeScript source code and package configuration while strictly preserving the test fixtures.

- [ ] Task: Delete source and test directories.
    - [ ] Delete `packages/tasklens/src/`.
    - [ ] Delete `packages/tasklens/tests/`.
- [ ] Task: Delete package configuration files.
    - [ ] Delete `packages/tasklens/package.json`.
    - [ ] Delete `packages/tasklens/tsconfig.json`.
    - [ ] Delete `packages/tasklens/turbo.json`.
    - [ ] Delete `packages/tasklens/vitest.config.ts`.
    - [ ] Delete any other root-level files in `packages/tasklens` EXCEPT the `specs` directory.
- [ ] Task: Verify Fixture Preservation.
    - [ ] Confirm `packages/tasklens/specs/compliance/fixtures/` exists and contains YAML files.
- [ ] Task: Conductor - User Manual Verification 'Source Removal' (Protocol in workflow.md)

## Phase 3: Dependency & Tooling Cleanup
Remove unused dependencies and update project-wide tooling configurations.

- [ ] Task: Update Knip configuration.
    - [ ] Check `knip.json` and remove `packages/tasklens` from entry points or workspaces.
- [ ] Task: Update ESLint/Prettier ignores.
    - [ ] Remove `packages/tasklens` from `.eslintignore` or `.prettierignore` if explicitly listed (optional, as the files are gone).
- [ ] Task: Prune unused dependencies.
    - [ ] Run `pnpm install` to update the lockfile.
    - [ ] Identify and remove dependencies that were *only* used by `packages/tasklens` (e.g., specific dev dependencies) from the root `package.json` if they were hoisted, or rely on `knip` to identify them later.
- [ ] Task: Conductor - User Manual Verification 'Dependency & Tooling Cleanup' (Protocol in workflow.md)

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
# Implementation Plan: Remove Turborepo and Migrate to Just

This plan details the steps to migrate monorepo orchestration from Turborepo to `just`, favoring sequential execution and explicit package listing in the root `justfile`.

## Phase 1: Audit and Foundation [checkpoint: ec42b50]
Audit the existing task graph and establish the core `justfile` structure.

- [x] Task: Map existing Turbo tasks to Just recipes a62b431
    - [x] Create a mapping document of every `turbo.json` task and its dependencies.
    - [x] Identify all root-level `package.json` scripts that currently rely on `turbo`.
- [x] Task: Initialize Justfile structure and variables 069af01
    - [x] Define variables for package paths in the `justfile`.
    - [x] Set up the default recipe to list available commands.
    - [x] **Cleanup:** Remove the dead `generate` scripts from `package.json` and `turbo.json`.
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: Core Build Pipeline [checkpoint: 434c9e3]
Migrate build processes and ensure dependencies are explicit.

- [x] Task: Implement package-specific `build` recipes 0ecea7e
    - [x] Add recipes for building `tasklens-ui` (wrapping `dx build`), `docs`, etc.
- [x] Task: Implement root `build` recipe 98a5e60
    - [x] Define the root `build` recipe to run package builds sequentially.
    - [x] Verify that `just build` compiles the entire project correctly.
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: Validation and Linting [checkpoint: 33312e6]
Migrate static analysis and type-checking tasks.

- [x] Task: Implement `check-style` (Linting) recipes d5fd01f
    - [x] Migrate Biome, ESLint, and Prettier checks for all packages and root.
- [x] Task: Implement `check-types` (TypeScript) recipes 1490305
    - [x] Migrate `tsc` checks for all TS packages.
- [x] Task: Implement `check-rust` (Clippy/Fmt) recipes d011434
    - [x] Ensure existing Rust audit logic is integrated into the `just` flow.
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: Testing Suite [checkpoint: 1887938]
Migrate unit tests and E2E tests.

- [x] Task: Implement `test` (Unit) recipes 38a3fb3
    - [x] Migrate Vitest and Cargo test executions.
- [x] Task: Implement `test-e2e` (Playwright) recipes 8faf01f
    - [x] Migrate Playwright execution.
    - [x] Ensure build dependencies (building the UI) are explicit in the recipe or documented.
- [x] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)

## Phase 5: Finalization and Orchestration
Remove Turbo and wrap all workflows in Just.

- [x] Task: Implement `fix` and `verify` recipes 83c85f1
    - [ ] Create the high-level `verify` recipe (sequential `check-style` -> `check-types` -> `test`).
    - [ ] Create the `check-agent` recipe.
- [x] Task: Redirect `package.json` scripts to Just e610d43
    - [ ] Update root `package.json` scripts to call `just <recipe>`.
- [ ] Task: Remove Turborepo dependencies
    - [ ] Delete `turbo.json` and `turbo-exclusions.json`.
    - [ ] Uninstall `turbo` from the root package.
    - [ ] Run `pnpm install` to update the lockfile.
- [ ] Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)

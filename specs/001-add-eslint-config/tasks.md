# Tasks: Add ESLint Configuration

**Feature**: Add ESLint Configuration (001-add-eslint-config)
**Status**: Todo
**Spec**: [spec.md](./spec.md)

## Phase 1: Setup

_Initialize project dependencies and configuration._

- [x] T001 Install `eslint`, `typescript-eslint`, `eslint-plugin-import-x` as dev dependencies in `package.json`
- [x] T002 Create `eslint.config.js` at root with `import/no-namespace` rule configured as warning and centralize ignore patterns

## Phase 2: Foundational

_Blocking prerequisites: Global renaming of scripts to align with new naming convention._

- [x] T003 [P] Rename `lint` script to `check-biome` in `apps/client/package.json`
- [x] T004 [P] Rename `lint` script to `check-biome` in `packages/tasklens/package.json`
- [x] T005 [P] Rename `lint` script to `check-biome` in `package.json` (root)
- [x] T006 [P] Rename `typecheck` script to `check-types` in `apps/client/package.json`
- [x] T007 [P] Rename `typecheck` script to `check-types` in `packages/tasklens/package.json`
- [x] T008 [P] Rename `typecheck` script to `check-types` in `package.json` (root)
- [x] T009 [P] Rename `lint:deps` script to `check-deps` in `package.json` (root)
- [x] T010 Rename `check-staged` script to `check-context-staged` in `package.json` (root)

## Phase 3: User Story 1 - Developer runs linting locally (P1)

_Goal: Developer can run `pnpm check-style` to see unified linting results._
_Independent Test: Run `pnpm check-style` and verify output contains only expected warnings._

- [x] T011 [P] [US1] Add `check-eslint` script (running `eslint .`) to `apps/client/package.json`
- [x] T012 [P] [US1] Add `check-eslint` script (running `eslint .`) to `packages/tasklens/package.json`
- [x] T013 [US1] Add `check-eslint` alias script to `package.json` (root)
- [x] T014 [US1] Add `check-style` aggregate script (running `turbo run check-style`) to `package.json` (root)
- [x] T015 [US1] Manual verification: Run `pnpm check-style` and ensure `import/no-namespace` warnings appear

## Phase 4: User Story 2 - Turbo Build Integration & Caching (P1)

_Goal: Linting is efficiently cached and integrated into the global build pipeline._
_Independent Test: Run `pnpm check-style` twice; second run should be `FULL TURBO`._

- [x] T016 [US2] Define `check-eslint` task in `turbo.json` with `eslint.config.js` as global input
- [x] T017 [US2] Define `check-style` aggregate task in `turbo.json` (depends on `check-biome`, `check-eslint`)
- [x] T018 [US2] Define `check` aggregate task in `turbo.json` (depends on `check-style`, `check-types`, `check-deps`)
- [x] T019 [US2] Define `fix-style` aggregate task in `turbo.json` (depends on `fix-eslint`, `fix-biome`)
- [x] T019a [US2] Define `fix` aggregate task in `turbo.json` (depends on `fix-style`, `fix-format`)
- [x] T020 [US2] Add `check` and `fix` scripts to `package.json` (root)
- [x] T020a [US2] Add `check` and `fix` scripts to `apps/client/package.json`
- [x] T020b [US2] Add `check` and `fix` scripts to `packages/tasklens/package.json`
- [x] T021 [US2] Manual verification: Run `pnpm check-style` twice to confirm Turbo caching

## Phase 5: Polish & Cross-Cutting

_Staged checks and documentation updates._

- [x] T022 Implement `check-filenames-staged` script in `package.json` (root)
- [x] T023 Implement `check-eslint-staged` script in `package.json` (root)
- [x] T024 Implement `check-biome-staged` script in `package.json` (root)
- [x] T025 Create new `check-staged` aggregate script (sequential execution) in `package.json` (root) and remove the legacy `pre-commit` script
- [x] T026 Update `AGENTS.md` and all documentation to reflect new task names (`check-style`, `check-staged`) and the removal of `pre-commit`
- [x] T027 Update `docs/guidance/task-naming.md` if necessary to reflect new conventions

## Dependencies

1. **Setup** -> **Foundational**
2. **Foundational** -> **User Story 1** (Need renamed scripts to aggregate)
3. **User Story 1** -> **User Story 2** (Need scripts to exist before Turbo config)
4. **User Story 2** -> **Polish** (Staged checks rely on base scripts)

## Implementation Strategy

- **MVP**: Complete through Phase 3 to get ESLint running.
- **Full Feature**: Complete Phase 4 and 5 to ensure CI/CD and Git hook integration.

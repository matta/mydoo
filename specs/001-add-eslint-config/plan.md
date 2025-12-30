# Implementation Plan - Add ESLint Configuration

**Feature**: Add ESLint Configuration (001-add-eslint-config)
**Status**: In Progress
**Spec**: [spec.md](./spec.md)

## Technical Context

| Category | Details | Status |
| :--- | :--- | :--- |
| **Languages** | TypeScript, JavaScript | Clear |
| **Frameworks** | ESLint (v9.x), TurboRepo | Clear |
| **Infrastructure** | Local-only (config files) | Clear |
| **Data** | N/A (Static Analysis) | Clear |
| **Integrations** | Turbo (`check`, `fix`), Git Hooks (`check-staged`) | Clear |
| **Security** | N/A | Clear |

## Constitution Check

| Principle | Evaluation | Decision |
| :--- | :--- | :--- |
| **Fidelity First** | N/A (Tooling) | Pass |
| **Executable Specs** | Tests defined in spec scenarios | Pass |
| **Local-First** | N/A (Tooling) | Pass |
| **Strict Git Hygiene** | Improves hygiene via stricter pre-commit checks and standard aggregates | Pass |
| **Derived State** | N/A | Pass |

## Phases

### Phase 0: Outline & Research

#### 1. Unknowns & Dependencies

- **Unknown 1**: Exact syntax for flat config with `typescript-eslint` + `eslint-plugin-import-x`.
    - *Research*: Resolved in `research.md`.
- **Unknown 2**: `turbo` configuration for sequential execution of `fix`.
    - *Research*: Resolved. Use `--concurrency=1` or chain scripts in `package.json`.

#### 2. Research Findings (`research.md`)

- **Decision**: Use `eslint-plugin-import-x` for v9 compatibility.
- **Decision**: `check` aggregates `check-style`, `check-types`, `check-deps` (Parallel).
- **Decision**: `fix` aggregates `fix-style`, `fix-format` (Sequential).
- **Decision**: All tasks follow `[action]-[subject]-[state]` kebab-case.

### Phase 1: Design & Contracts

#### 1. Data Model (`data-model.md`)

*N/A - Infrastructure change.*

#### 2. Contracts

- **CLI Contract (Root & Packages)**:
    - `pnpm check`: Aggregate gate (Parallel).
    - `pnpm fix`: Aggregate gate (Sequential).
    - `pnpm check-style`: ESLint + Biome aggregate.
    - `pnpm check-staged`: Pre-commit aggregate.

### Phase 2: Implementation

#### 1. Scaffolding & Renaming

- Install `eslint`, `typescript-eslint`, `eslint-plugin-import-x`.
- Create `eslint.config.js` at root.
- **Global Rename**: Update all existing `package.json` scripts and `turbo.json` tasks to comply with `docs/guidance/task-naming.md`.
- Update `AGENTS.md` and `scripts/*` references.

#### 2. Task Topology (`turbo.json`)

- Define `check-eslint`, `check-biome`, `check-types`, `check-deps`.
- Define aggregate `check-style`.
- Define aggregate `check` (dependsOn: `check-style`, `check-types`, `check-deps`).
- Define `fix-eslint`, `fix-biome`.
- Define aggregate `fix` (sequential).

#### 3. Staged Quality Gate

- Update `check-staged` to run `check-biome-staged && check-eslint-staged && check-filenames-staged && check-context-staged`.

#### 4. Verification

- Execute `pnpm check` and `pnpm fix`.
- Verify caching by running twice (Success Criteria SC-002).

## Artifacts

- [x] `research.md`
- [ ] `eslint.config.js`
- [ ] Updated `package.json` (Root & Packages)
- [ ] Updated `turbo.json`
- [ ] Updated `AGENTS.md`
# Implementation Plan: Moonrepo Migration

**Branch**: `003-moonrepo-migration` | **Date**: Tuesday, December 30, 2025 | **Spec**: [specs/003-moonrepo-migration/spec.md](spec.md)
**Input**: Feature specification from `specs/003-moonrepo-migration/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

The goal is to replace the `turbo` build system with `moonrepo` while preserving the exact same developer workflow (commands, scripts) and CI verification rigor. This involves:
1.  Initializing moonrepo configuration (`.moon` directory) to manage the workspace.
2.  Configuring toolchain management for Node.js and pnpm to ensure environmental consistency.
3.  Migrating all tasks from `turbo.json` (build, test, check, etc.) to moon's `tasks.yml` or project-specific `moon.yml` files.
4.  Updating `package.json` scripts to invoke `moon` instead of `turbo`.
5.  Refactoring `.github/workflows/ci.yml` to use `moon ci` and native caching, removing turbo-specific caching steps.
6.  Removing `turbo` dependencies and configuration.

## Technical Context

**Language/Version**: TypeScript / Node.js 24 (Strictly Managed by Moon)
**Primary Dependencies**: `moon` (CLI), `proto` (Toolchain manager)
**Storage**: N/A (Build system only)
**Testing**: `vitest` (Unit), `playwright` (E2E) - Orchestrated by Moon
**Target Platform**: Linux (CI), macOS/Windows/Linux (Dev)
**Project Type**: Monorepo (pnpm workspace)
**Performance Goals**: Parity or improvement over Turbo for task execution and caching.
**Constraints**: Must run in existing CI environment (GitHub Actions). Offline capability for local dev (once tools installed).
**Scale/Scope**: Current workspace (~2 packages/apps), extensible to more.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Fidelity First**: Does the testing plan avoid JSDOM for logic? (Tier 1/3 focus) - *N/A: Migration doesn't change test environments, just orchestration.*
- [x] **Local-First**: Is offline capability and conflict resolution (CRDTs) considered? - *N/A: Build system change.*
- [x] **Architecture**: Does the design respect the Client-Centric / No-Block-on-Network rule? - *N/A: Build system change.*
- [x] **State Separation**: Are domain selectors and UI hooks clearly distinguished? - *N/A: Build system change.*

## Project Structure

### Documentation (this feature)

```text
specs/003-moonrepo-migration/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Configuration
.moon/
├── workspace.yml        # Workspace settings & project discovery
├── toolchain.yml        # Node/pnpm version management
└── tasks.yml            # Global task definitions (lint, format)

# Root-level configuration
.prototools              # Pin moon version
package.json             # Updated scripts

# App/Package configuration (migrated from turbo pipeline)
apps/
└── client/
    └── moon.yml         # Project-specific tasks

packages/
└── tasklens/
    └── moon.yml         # Project-specific tasks
```

**Structure Decision**: Standard Moonrepo configuration layout (`.moon` folder) plus `moon.yml` files alongside `package.json` in each project.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | | |
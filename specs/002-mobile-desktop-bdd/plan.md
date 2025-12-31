# Implementation Plan: Cross-Platform BDD Execution

**Branch**: `002-mobile-desktop-bdd` | **Date**: 2025-12-30 | **Spec**: [specs/002-mobile-desktop-bdd/spec.md](spec.md)
**Input**: Feature specification from `/specs/002-mobile-desktop-bdd/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement cross-platform BDD test execution by configuring Playwright to run scenarios against both Desktop (Chrome) and Mobile (Pixel 7 emulation) environments. This involves updating `playwright.config.ts` to use Playwright "Projects" for separate configurations and updating the CI pipeline (`.github/workflows/ci.yml`) to execute these projects sequentially. The goal is to ensure functional parity and UI responsiveness across devices without managing separate test suites.

## Technical Context

**Language/Version**: TypeScript / Node.js
**Primary Dependencies**: Playwright (`@playwright/test`), `playwright-bdd`
**Storage**: N/A (Test Configuration)
**Testing**: Playwright (E2E/BDD)
**Target Platform**: Desktop (Chrome) & Mobile (Pixel 7 Emulation)
**Project Type**: Web Application (Monorepo)
**Performance Goals**: Sequential execution accepted (approx 2x duration); parallel execution reserved for future optimization.
**Constraints**: Platform-specific test logic must use polymorphic abstractions (no skipping).
**Scale/Scope**: 100% of BDD scenarios.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Fidelity First**: Does the testing plan avoid JSDOM for logic? (Tier 1/3 focus) - *Yes, uses real browsers via Playwright.*
- [x] **Local-First**: Is offline capability and conflict resolution (CRDTs) considered? - *Yes, tests run against the local-first client.*
- [x] **Architecture**: Does the design respect the Client-Centric / No-Block-on-Network rule? - *Yes, infrastructure change only.*
- [x] **State Separation**: Are domain selectors and UI hooks clearly distinguished? - *N/A (Infrastructure).*

## Project Structure

### Documentation (this feature)

```text
specs/002-mobile-desktop-bdd/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (N/A)
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
apps/client/
├── playwright.config.ts  # Main configuration update
└── package.json          # Script updates

.github/workflows/
└── ci.yml                # CI pipeline update
```

**Structure Decision**: Modify existing `apps/client` configuration and root CI workflow.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | | |
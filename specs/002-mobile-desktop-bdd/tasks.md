# Tasks: Cross-Platform BDD Execution

**Feature**: `002-mobile-desktop-bdd`
**Status**: Pending

## Phase 1: Setup
*Goal: Initialize configuration for multi-project support.*

- [x] T001 Update `playwright.config.ts` to implement `projects` array with 'Desktop Chrome' and 'Mobile Pixel 7' configurations in `apps/client/playwright.config.ts`
- [x] T002 Update `package.json` scripts to support running specific projects (e.g., `test:e2e:mobile`, `test:e2e:desktop`) in `apps/client/package.json`

## Phase 2: Foundational
*Goal: Ensure reporting and infrastructure can handle multiple test runs.*

- [x] T003 Configure Playwright HTML reporter to support merged/grouped results in `apps/client/playwright.config.ts`
- [x] T004 Validate that `playwright-bdd` generation is compatible with shared project execution (no code changes expected, verification step) in `apps/client/playwright.config.ts`

## Phase 3: Desktop Verification (US1)
*Goal: Verify that existing functionality remains stable on Desktop.*

- [x] T005 [US1] Run full BDD suite using `Desktop Chrome` project and verify all tests pass locally in `apps/client`

## Phase 4: Mobile Verification (US2)
*Goal: Enable mobile emulation and ensure responsiveness.*

- [x] T006 [US2] Run full BDD suite using `Mobile Pixel 7` project and identify any immediate layout/interaction failures in `apps/client`
- [x] T007 [US2] [P] Refactor any failing tests to use platform-agnostic locators or interactions (polymorphic abstractions) if failures occur (placeholder for potential refactoring) in `apps/client/tests/e2e/steps/`

## Phase 5: CI Pipeline Validation (US3)
*Goal: Enforce cross-platform checks in the build pipeline.*

- [x] T008 [US3] Update `.github/workflows/ci.yml` to execute Desktop project then Mobile project sequentially
- [x] T009 [US3] Configure CI artifact upload to preserve the combined HTML report in `.github/workflows/ci.yml`

## Phase 6: Polish
*Goal: Documentation and final cleanup.*

- [x] T010 Update project README to document new test commands and mobile debugging workflow in `apps/client/README.md`

## Dependencies

- Phase 1 must complete before Phase 3 and 4.
- Phase 3 and 4 can be executed in parallel (locally).
- Phase 5 requires stable tests from Phase 3 and 4.

## Parallel Execution Examples

- **Mobile & Desktop Verification**: Developers can run `pnpm test:e2e --project='bdd-desktop'` and `pnpm test:e2e --project='bdd-mobile'` in separate terminals.

## Implementation Strategy

- **MVP**: Enable the configuration and ensure Desktop passes (regression safety).
- **Fast Follow**: Enable Mobile and fix any immediate layout issues.
- **Final**: Enforce in CI.

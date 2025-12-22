# Implementation Plan: Phase 1 (Infrastructure)

**Goal**: Establish the tooling, dependencies, and core data wiring required for the View Layer.

## Step 1: Install Dependencies

_Install Mantine UI library and testing tools._

- [x] Install Mantine core packages (`@mantine/core`, hooks, form, dates) + `dayjs`.
- [x] Install PostCSS dependencies.
- [x] Install Test dependencies (`vitest`, `jsdom`, `@testing-library/react`, `@playwright/test`).

**Validation**

- [x] `pnpm install` -> Clean exit
- [x] `pnpm build` -> Pass

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 2: Configure Build & Test System

_Setup PostCSS, Vitest, and Playwright._

- [x] Create `postcss.config.cjs` (Mantine preset).
- [x] Create `apps/client/vitest.config.ts` (JSDOM environment).
- [x] Create `playwright.config.ts` (Base configuration).
- [x] Update `apps/client/package.json` scripts (`test`, `test:e2e`).

**Validation**

- [x] `pnpm lint` -> Pass
- [x] `pnpm --filter client test` -> Pass (0 tests is fine, just checks config load)

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 3: View Layer Scaffolding

_Create the directory structure for MVVM pattern._

- [ ] Create `viewmodel/{projections,intents,containers}`.
- [ ] Create `components/{primitives,composites,modals,layouts}`.
- [ ] Create `tests/` directory.

**Validation**

- [ ] `tree apps/client/src` shows new folders

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 4: Core Wiring (App Entry)

_Setup providers and global styles._

- [ ] Update `apps/client/src/main.tsx`:
  - Wrap with `<MantineProvider>`.
  - Ensure `<AutomergeProvider>` is present.
  - Import Mantine CSS (`@mantine/core/styles.css`, etc).

**Validation**

- [ ] `pnpm build` -> Pass
- [ ] `pnpm dev` -> Browser loads with Mantine styles (inspect element to verify)

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 5: Implement `useDocument` Hook

_Create the foundational hook for data access._

- [ ] Create `apps/client/src/viewmodel/projections/useDocument.ts`.
- [ ] Implement hook to retrieve/create document handle from Context.
- [ ] Write integration test: `tests/useDocument.test.tsx`.

**Validation**

- [ ] `pnpm lint` -> Pass
- [ ] `pnpm --filter client test` -> Pass (1 test passing)

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

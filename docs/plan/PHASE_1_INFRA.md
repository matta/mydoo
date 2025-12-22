# Implementation Plan: Phase 1 (Infrastructure)

**Goal**: Establish the tooling, dependencies, and core data wiring required for the View Layer.

## Step 1: Install Dependencies

_Install Mantine UI library and testing tools._

- [x] Install Mantine core packages (`@mantine/core`, hooks, form, dates) + `dayjs`.
- [x] Install PostCSS dependencies.
- [x] Install Test dependencies (`vitest`, `jsdom`, `@testing-library/react`, `@playwright/test`).

**Validation**

- [x] `pnpm fix` -> Pass
- [x] `pnpm install` -> Clean exit
- [x] `pnpm build` -> Pass
- [x] `pnpm test` -> Pass
- [x] `pnpm test:e2e` -> Pass

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

- [x] `pnpm fix` -> Pass
- [x] `pnpm lint` -> Pass
- [x] `pnpm --filter client test` -> Pass
- [x] `pnpm --filter client test:e2e` -> Pass

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 3: View Layer Scaffolding

_Create the directory structure for MVVM pattern._

- [x] Create `viewmodel/{projections,intents,containers}`.
- [x] Create `components/{primitives,composites,modals,layouts}`.
- [x] Create `tests/` directory.

**Validation**

- [x] `pnpm fix` -> Pass
- [x] `tree apps/client/src` shows new folders
- [x] `pnpm build` -> Pass
- [x] `pnpm --filter client test` -> Pass
- [x] `pnpm --filter client test:e2e` -> Pass

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

**Validation**

- [x] `pnpm fix` -> Pass
- [x] `pnpm build` -> Pass
- [x] `pnpm dev` -> Browser loads with Mantine styles (inspect element to verify)
- [x] `pnpm --filter client test` -> Pass
- [x] `pnpm --filter client test:e2e` -> Pass

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

## Step 5: Implement `useDocument` Hook

_Create the foundational hook for data access._

- [ ] Create `apps/client/src/viewmodel/projections/useDocument.ts`.
- [ ] Implement hook to retrieve/create document handle from Context.
- [ ] Write integration test: `tests/useDocument.test.tsx`.

**Validation**

**Validation**

- [ ] `pnpm fix` -> Pass
- [ ] `pnpm lint` -> Pass
- [ ] `pnpm --filter client test` -> Pass (1 test passing)
- [ ] `pnpm --filter client test:e2e` -> Pass

**Completion**

- [ ] 🛑 STOP and prompt for user review.
- [ ] 💾 Request git commit.

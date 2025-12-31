# Quickstart: Cross-Platform BDD Execution

**Feature**: `002-mobile-desktop-bdd`

## Prerequisites

- Ensure dependencies are installed: `pnpm install`
- Ensure Playwright browsers are installed: `npx playwright install`

## Running Tests

### 1. Run All Platforms (Sequential/Parallel depending on config)
Runs both Desktop and Mobile suites.
```bash
pnpm test:e2e
```

### 2. Run Desktop Only
Target the desktop Chrome configuration.
```bash
pnpm test:e2e --project=bdd-desktop
```

### 3. Run Mobile Only
Target the Pixel 7 emulation configuration.
```bash
pnpm test:e2e --project=bdd-mobile
```

## Debugging Mobile Tests

To see the mobile viewport and interactions visually:
```bash
pnpm test:e2e --project=bdd-mobile --ui
```
This opens the Playwright UI mode where you can watch the emulator tap and interact with the responsive layout.

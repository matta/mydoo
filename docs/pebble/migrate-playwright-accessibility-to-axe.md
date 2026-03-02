---
id: issue-wz72nyhi64j
title: Migrate Playwright accessibility to Axe
status: done
priority: 30
created_at: 2026-03-02T14:59:36.028341685+00:00
modified_at: 2026-03-02T14:59:36.038335791+00:00
resolved_at: 2026-03-02T14:59:36.038332700+00:00
tags:
  - task
---
Playwright has deprecated and removed page.accessibility. Replace its current usage with Axe or the newer Locator.ariaSnapshot() method. Remove the declare module hack in playwright.d.ts.\n\nTidbit: The locator.ariaSnapshot() method allows you to programmatically create a YAML representation of accessible elements within a locator's scope, especially helpful for generating snapshots dynamically during test execution.\n\nExample:\nconst snapshot = await page.locator('body').ariaSnapshot();\nconsole.log(snapshot);

## Acceptance Criteria

1. Replace all usages of page.accessibility.snapshot() with Axe checks or locator.ariaSnapshot() (including in .agent/skills/playwright-debug/SKILL.md).\n2. Remove the module augmentation in crates/tasklens-ui/tests/e2e/playwright.d.ts.\n3. Ensure all E2E tests still pass.

---
*Imported from beads issue mydoo-yr4*

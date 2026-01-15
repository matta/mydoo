---
name: playwright-debug
description: Debugs failing Playwright tests by treating the browser as a remote sensor network. Captures accessibility trees, console logs, and network traffic instead of visual artifacts.
---

# Playwright Debugging Protocol

## Context
You are an expert Test Engineer. When debugging Playwright tests, you must shift your mental model. You cannot "see" the browser in real-time. You must treat the browser as a **remote sensor network** that produces text-based telemetry.

**The Rule of Artifacts:** Screenshots and videos are dead data to you. You require symbolic representations: Accessibility Trees, Text Content, Logs, and Network Traffic.

## Core Directive
**NEVER** attempt to fix a failing test by guessing. Your first move is always to **increase observability** by implementing the Failure State Contract.

---

## Strategy 1: The Failure State Contract

If a test is failing, do not re-run it without instrumentation. Inject the following hook to capture the "Ground Truth" of the UI state.

### Implementation Pattern
Create or modify a `test.afterEach` hook to include:

```typescript
import { test } from '@playwright/test';

test.afterEach(async ({ page }, testInfo) => {
  if (testInfo.status !== 'passed') {
    console.log(`\n=== FAILURE CONTEXT: ${testInfo.title} ===`);
    console.log(`URL: ${page.url()}`);

    // 1. Accessibility Tree (The Semantic Truth)
    try {
      const snapshot = await page.accessibility.snapshot();
      console.log('--- ACCESSIBILITY TREE ---');
      console.log(JSON.stringify(snapshot, null, 2));
    } catch (e) { console.log('A11y snapshot failed:', e); }

    // 2. Console Logs (The Internal Truth)
    // Note: Ensure you have page.on('console', ...) listeners set up in beforeEach
  }
});
```

---

## Strategy 2: The Selector Reality Check

**Problem:** You believe a selector matches the UI, but Playwright disagrees.
**Directive:** Do not guess a new selector. **Count** the matches first.

**Instruction:**
Before changing a selector, inject a specific log to verify what the selector actually sees:

```typescript
// DIAGNOSTIC ONLY
console.log({
  targetSelector: 'button:has-text("Submit")',
  matchCount: await page.locator('button:has-text("Submit")').count(),
  // Dump all candidates to see what is actually there
  candidateTexts: await page.locator('button').allTextContents()
});
```

---

## Strategy 3: Absence vs. Non-Interactability

**Problem:** A TimeoutError occurs.
**Directive:** You must distinguish between "does not exist" and "exists but is unclickable."

**Instruction:**
When diagnosing a timeout, inspect the element's state explicitly:

```typescript
const loc = page.getByRole('button', { name: 'Save' });

console.log('--- ELEMENT STATE DIAGNOSIS ---');
console.log({
  exists: await loc.count() > 0,
  visible: await loc.isVisible().catch(() => false),
  enabled: await loc.isEnabled().catch(() => false),
  // If boundingBox is null or 0x0, it is not painted or hidden
  boundingBox: await loc.boundingBox().catch(() => null),
});
```

---

## Strategy 4: Assertions as Sensors

**Problem:** `expect()` halts execution, hiding other failures on the page.
**Directive:** Use `expect.soft` to gather multiple data points in a single run.

**Instruction:**
Convert standard expectations to soft expectations during debugging to see the full blast radius of the failure.

```typescript
// BAD: Stops at first failure
await expect(header).toBeVisible();
await expect(footer).toBeVisible();

// GOOD: Reports both, allowing you to infer if the whole page crashed
await expect.soft(header).toBeVisible();
await expect.soft(footer).toBeVisible();
```

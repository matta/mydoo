/**
 * TYPE AUGMENTATION FOR PLAYWRIGHT
 *
 * Purpose:
 * This file exists to fix a gap in the official TypeScript definitions for Playwright.
 * At runtime, the `page` object has an `accessibility` namespace with a `snapshot()` method,
 * but the TypeScript types shipped with `@playwright/test` do not currently include it.
 *
 * Mechanism: "Declaration Merging"
 * TypeScript allows us to "re-open" an existing module and interface to add new properties.
 * By declaring the module "@playwright/test" and the interface "Page" again here,
 * TypeScript merges our definition with the official one.
 *
 * Result:
 * This globally adds `accessibility: { snapshot(...) }` to the `Page` type throughout
 * the entire project, allowing us to use `page.accessibility.snapshot()` without
 * manual casting or "any" types.
 */
import type { ElementHandle } from "@playwright/test";

// Augment the existing module
declare module "@playwright/test" {
  interface Page {
    accessibility: {
      snapshot(options?: {
        interestingOnly?: boolean;
        root?: ElementHandle;
      }): Promise<unknown>;
    };
  }
}

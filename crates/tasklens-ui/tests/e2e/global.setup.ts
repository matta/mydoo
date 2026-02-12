import fs from "node:fs";
import path from "node:path";
import { expect, test as setup } from "@playwright/test";

import { snapshotDir } from "./snapshot-paths";

setup.describe.configure({ mode: "serial" });

setup.beforeAll(() => {
  fs.mkdirSync(snapshotDir, { recursive: true });
});

setup("prepare empty-db snapshot", async ({ browser }) => {
  const context = await browser.newContext({ serviceWorkers: "block" });
  const page = await context.newPage();

  await page.goto("/");
  await page.waitForLoadState("domcontentloaded");
  await expect(page.locator('[data-app-state="ready"]')).toBeAttached({
    timeout: 30_000,
  });

  await context.storageState({
    indexedDB: true,
    path: path.join(snapshotDir, "empty-db.json"),
  });

  await context.close();
});

setup("prepare sample-db snapshot", async ({ browser }) => {
  const context = await browser.newContext({ serviceWorkers: "block" });
  const page = await context.newPage();

  await page.goto("/");
  await page.waitForLoadState("domcontentloaded");
  await expect(page.locator('[data-app-state="ready"]')).toBeAttached({
    timeout: 30_000,
  });

  const hasSeedFn = await page.evaluate(() => {
    return typeof Reflect.get(window, "tasklensSeedSampleData") === "function";
  });

  if (hasSeedFn) {
    await page.evaluate(async () => {
      const fn = Reflect.get(window, "tasklensSeedSampleData");
      await fn();
    });

    await expect(page.locator('[data-app-state="ready"]')).toBeAttached({
      timeout: 30_000,
    });
    await expect(page.getByRole("heading", { name: "Plan" })).toBeVisible();
  }

  await context.storageState({
    indexedDB: true,
    path: path.join(snapshotDir, "sample-db.json"),
  });

  await context.close();
});

setup("record app assets HAR", async ({ browser }) => {
  const context = await browser.newContext({ serviceWorkers: "block" });
  await context.routeFromHAR(path.join(snapshotDir, "app-assets.har"), {
    update: true,
    updateContent: "attach",
    updateMode: "minimal",
  });

  const page = await context.newPage();
  await page.goto("/");
  await page.waitForLoadState("domcontentloaded");
  await expect(page.locator('[data-app-state="ready"]')).toBeAttached({
    timeout: 30_000,
  });

  await context.close();
});

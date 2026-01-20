import { createBdd } from "playwright-bdd";
import { expect, test } from "../fixtures";

const { When, Then } = createBdd(test);

// We need to share state between steps.
// In BDD, we usually use the World or share via closure if tests are sequential and not parallel in same worker?
// Playwright-bdd steps run in the same fixture context usually, but variables outside are shared across tests in the file (module scope).
// To be safe and test-isolated, we should attach it to the `ctx` or use a custom fixture.
// But valid BDD setup often uses a World object.
// Here I'll use a module-level variable but reset it? Or better, use `test.use`?
// Simpler: Just use a variable, as tests run in isolation (one worker per test usually).
// But parallel execution might mess this up if multiple tests use this file.
// Ideally, use `testInfo` or a custom fixture.
// For now, I'll store it in a map keyed by testId or similar, OR just assume single test scenario.
// I'll stick to a variable for simplicity, assuming one scenario running at a time per worker.

let downloadedPath: string | null = null;
let originalDocId: string | null = null;

When("I export the document as binary", async ({ page }) => {
  // Store original Doc ID
  originalDocId = await page.evaluate(() =>
    localStorage.getItem("mydoo:doc_id"),
  );

  // Find Options menu
  await page.getByRole("button", { name: "Options" }).click();

  // Prepare for download
  const downloadPromise = page.waitForEvent("download");

  // Click Download Binary
  await page.getByRole("menuitem", { name: "Download Binary" }).click();

  const download = await downloadPromise;

  // Save to a temp file path (Playwright manages this until the context closes)
  downloadedPath = await download.path();

  // Close menu if not closed (it should be)
});

When("I import the binary document", async ({ page }) => {
  if (!downloadedPath) throw new Error("No downloaded file found");

  // Options menu
  await page.getByRole("button", { name: "Options" }).click();

  // Handle file chooser
  const fileChooserPromise = page.waitForEvent("filechooser");
  // Use text to be more lenient with roles
  await page.getByText("Upload Binary").click();
  const fileChooser = await fileChooserPromise;

  await fileChooser.setFiles(downloadedPath);

  // Wait for the import to finish and restore the original Doc ID
  await expect(async () => {
    const currentId = await page.evaluate(() =>
      localStorage.getItem("mydoo:doc_id"),
    );
    expect(currentId).toBe(originalDocId);
  }).toPass({ timeout: 10000 });

  // Wait for the reload to reflect the change
  await page.waitForTimeout(500); // Give the app a moment to trigger reload
  await page.waitForLoadState("domcontentloaded");
});

Then("the document URL should be preserved", async ({ page }) => {
  // The URL might have different query params (like ?seed=true),
  // but the underlying document should be the same.
  // We verify this by checking the doc_id in localStorage.
  const currentDocId = await page.evaluate(() =>
    localStorage.getItem("mydoo:doc_id"),
  );

  expect(currentDocId).toBe(originalDocId);
});

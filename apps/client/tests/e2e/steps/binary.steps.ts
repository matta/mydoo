import { createBdd } from "playwright-bdd";
import { expect, test } from "../fixtures";

const { When, Then } = createBdd(test);

When("I export the document as binary", async ({ page, binaryContext }) => {
  // Store original Doc ID
  binaryContext.originalDocId = await page.evaluate(() =>
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
  binaryContext.downloadedPath = await download.path();
});

Then("the document ID should be different", async ({ page, binaryContext }) => {
  const currentId = await page.evaluate(() =>
    localStorage.getItem("mydoo:doc_id"),
  );
  expect(currentId).not.toBe(binaryContext.originalDocId);
});

When("I import the binary document", async ({ page, binaryContext }) => {
  if (!binaryContext.downloadedPath)
    throw new Error("No downloaded file found");

  // Options menu
  await page.getByRole("button", { name: "Options" }).click();

  // Handle file chooser
  const fileChooserPromise = page.waitForEvent("filechooser");
  // Use text to be more lenient with roles
  await page.getByText("Upload Binary").click();
  const fileChooser = await fileChooserPromise;

  await fileChooser.setFiles(binaryContext.downloadedPath);

  // Wait for the import to finish and restore the original Doc ID in localStorage
  await expect(async () => {
    const currentId = await page.evaluate(() =>
      localStorage.getItem("mydoo:doc_id"),
    );
    expect(currentId).toBe(binaryContext.originalDocId);
  }).toPass({ timeout: 10000 });

  // The app triggers window.location.reload() after setting localStorage.
  // We must wait for the page to actually reload and be ready.
  // We wait for the 'commit' of the navigation or a reload.
  await page.waitForNavigation({ waitUntil: "load", timeout: 15000 });
});

Then(
  "the document URL should be preserved",
  async ({ page, binaryContext }) => {
    // The URL might have different query params (like ?seed=true),
    // but the underlying document should be the same.
    // We verify this by checking the doc_id in localStorage.
    const currentDocId = await page.evaluate(() =>
      localStorage.getItem("mydoo:doc_id"),
    );

    expect(currentDocId).toBe(binaryContext.originalDocId);
  },
);

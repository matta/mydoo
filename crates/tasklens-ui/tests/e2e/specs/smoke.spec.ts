import { expect, test } from "../fixtures";

test.describe("Smoke Test", () => {
  test("User launches the app", async ({ I }) => {
    // Given

    // Then
    await I.Then.pageTitleContains("TaskLens");
  });

  test("Sample data can be seeded without reloading", async ({ I, page }) => {
    const beforeSeedUrl = page.url();

    await I.Given.seededWithSampleData();

    await expect(page).toHaveURL(beforeSeedUrl);
    await expect(page.getByTestId("task-item")).not.toHaveCount(0);
  });
});

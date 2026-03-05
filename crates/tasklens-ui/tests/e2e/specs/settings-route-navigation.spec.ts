import { expect, test } from "../fixtures";

test.describe("Settings Route Navigation", () => {
  test("Plan entry: opens settings from Plan with ctx=plan", async ({
    I,
    page,
  }) => {
    await I.When.switchToPlanView();

    await page.getByTestId("settings-button").click();

    await expect(page.getByTestId("settings-page")).toBeVisible();
    await expect(page).toHaveURL(/\/settings\?ctx=plan/);
  });

  test("Do entry: opens settings from Do with ctx=do", async ({ I, page }) => {
    await I.When.switchToDoView();

    await page.getByTestId("settings-button").click();

    await expect(page.getByTestId("settings-page")).toBeVisible();
    await expect(page).toHaveURL(/\/settings\?ctx=do/);
  });

  test("In-App Parity Rule: close-settings navigates back like browser Back", async ({
    I,
    page,
  }) => {
    await I.When.switchToPlanView();
    await expect(page).toHaveURL(/\/plan/);

    // Navigate to settings from Plan
    await page.getByTestId("settings-button").click();
    await expect(page.getByTestId("settings-page")).toBeVisible();

    // Click close-settings
    await page.getByTestId("close-settings").click();

    // Should navigate back to Plan (same as browser Back would do)
    await expect(page).toHaveURL(/\/plan/);
  });
});

import { expect, test } from "@playwright/test";

test.describe("Task Completion Lifecycle", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    // Clear any existing state if needed (handled by Incognito browser context usually)
  });

  test("Bug Repro: Completed tasks should remain visible until refreshed", async ({
    page,
  }) => {
    // 1. Create a task
    const taskTitle = `Remediation Task ${Date.now()}`;
    await page.getByPlaceholder("Add a new task...").fill(taskTitle);
    await page.getByPlaceholder("Add a new task...").press("Enter");

    // Verify it appears
    const taskRow = page.getByText(taskTitle);
    await expect(taskRow).toBeVisible();

    // 2. Complete the task
    // Find the checkbox associated with this task.
    // Using a more robust selector strategy: find the row containing the text, then find the checkbox within it.
    // Note: Depends on DOM structure. Assuming TaskRow contains both.
    // For now, using the aria-label strategy used in interactions.spec.ts
    const checkbox = page.getByRole("checkbox", {
      name: `Complete ${taskTitle}`,
    });
    await expect(checkbox).toBeVisible();
    await checkbox.click();

    // 3. VERIFY: It should STILL be visible (Strikethrough check is secondary, main point is existence)
    // CURRENT BUG: It disappears immediately. This assertion EXPECTS it to stay.
    await expect(taskRow).toBeVisible({ timeout: 2000 });

    // 4. VERIFY: It should have strikethrough (optional visual check, but good to have)
    // This depends on implementation details (CSS class or style), leaving broad for now
    // const rowContainer = page.locator('div', { has: page.getByText(taskTitle) }).last();
    // await expect(rowContainer).toHaveCSS('text-decoration', /line-through/);

    // 5. Click Refresh/Update button
    const refreshButton = page.getByRole("button", { name: /update|refresh/i });
    await expect(refreshButton).toBeVisible();
    await refreshButton.click();

    // 6. VERIFY: Now it should disappear
    await expect(taskRow).toBeHidden();
  });
});

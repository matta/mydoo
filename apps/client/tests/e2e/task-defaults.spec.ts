import { expect, test } from "@playwright/test";
import { PlanPage } from "./pages/plan-page";

test.describe("Task Defaults", () => {
  let planPage: PlanPage;

  test.beforeEach(async ({ page }) => {
    planPage = new PlanPage(page);
    await planPage.primeWithSampleData();
  });

  test("should have correct defaults (Importance 1.0, Lead Time 8 Hours) when creating a new task", async ({
    page,
  }) => {
    // 1. Navigate to Plan View
    await planPage.switchToPlanView();

    // 2. Open "Create Task" modal via "Add Task at Top" (or similar generic create trigger)
    // We can use the generic create trigger to check defaults for a root task
    const addTop = page.getByLabel("Add Task at Top");
    if (await addTop.isVisible()) {
      await addTop.click();
    } else {
      // Fallback: Add sibling to first task if "Add Task at Top" isn't visible
      const firstTask = page.locator(`[data-testid="task-item"]`).first();
      await firstTask.hover();
      await firstTask.getByTestId("task-menu-trigger").click();
      await page.getByRole("menuitem", { name: "Add Sibling" }).click();
    }

    const modal = page.getByRole("dialog", { name: "Create Task" });
    await expect(modal).toBeVisible();

    // 3. Verify Importance Default (1.0)
    // Importance is a slider. We can check the text "Importance: 1.00"
    await expect(modal.getByText("Importance: 1.00")).toBeVisible();

    // 4. Verify Lead Time Defaults (8 Hours)
    // Check "Lead Time" input value with ID #lead-time-scalar-input
    await expect(modal.locator("#lead-time-scalar-input")).toHaveValue("8");

    // Check "Unit" select value with ID #lead-time-unit-select
    await expect(modal.locator("#lead-time-unit-select")).toHaveValue("Hours");
  });

  test("should have correct defaults when adding a child task", async ({
    page,
  }) => {
    await planPage.switchToPlanView();
    const taskTitle = "Root Task for Defaults";
    await planPage.createTask(taskTitle);

    // open the editor for the task we just created
    await planPage.openTaskEditor(taskTitle);

    const editModal = page.getByRole("dialog", { name: "Edit Task" });
    await editModal.getByRole("button", { name: "Add Child" }).click();

    const createModal = page.getByRole("dialog", { name: "Create Task" });
    await expect(createModal).toBeVisible();

    // 3. Verify Importance Default (1.0)
    await expect(createModal.getByText("Importance: 1.00")).toBeVisible();

    // 4. Verify Lead Time Defaults (8 Hours)
    await expect(createModal.locator("#lead-time-scalar-input")).toHaveValue(
      "8",
    );
    await expect(createModal.locator("#lead-time-unit-select")).toHaveValue(
      "Hours",
    );
  });
});

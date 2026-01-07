import { test } from "./fixtures";

test.describe("Regression Coverage: Full Journey", () => {
  test.describe("Desktop View", () => {
    test.use({ viewport: { width: 1280, height: 720 } });

    test("Complete Desktop Journey", async ({ plan, page }) => {
      await page.goto("/");
      await plan.switchToPlanView();

      await test.step("Create and verify focus on first task", async () => {
        const rootTitle = "Desktop Root";
        await plan.addFirstTask(rootTitle);
        await plan.verifyTaskVisible(rootTitle);
      });

      await test.step("Rename and verify focus in Edit Mode", async () => {
        const rootTitle = "Desktop Root";
        const renamedTitle = "Desktop Root Renamed";
        await plan.editTaskTitle(rootTitle, renamedTitle);
        await plan.verifyTaskVisible(renamedTitle);
      });

      await test.step("Add sibling and verify focus", async () => {
        const rootTitle = "Desktop Root Renamed";
        const siblingTitle = "Desktop Sibling";
        await plan.addSibling(rootTitle, siblingTitle);
        await plan.verifyTaskVisible(siblingTitle);
      });

      await test.step("Add child and verify auto-expand", async () => {
        const rootTitle = "Desktop Root Renamed";
        const childTitle = "Desktop Child";
        await plan.openTaskEditor(rootTitle);
        await plan.addChild(childTitle);
        await plan.verifyTaskVisible(childTitle);
      });
    });
  });

  test.describe("Mobile View", () => {
    test.use({ viewport: { width: 375, height: 667 } });

    test("Complete Mobile Journey", async ({ plan, page }) => {
      await page.goto("/");
      await plan.switchToPlanView();

      await test.step("Create first task", async () => {
        const rootTitle = "Mobile Root";
        await plan.addFirstTask(rootTitle);
        await plan.verifyTaskVisible(rootTitle);
      });

      await test.step("Add sibling via footer", async () => {
        // Mobile footer 'Add Task at Top'
        await page.getByLabel("Add Task at Top").click();
        const modal = page.getByRole("dialog", { name: "Create Task" });
        const titleInput = modal.getByRole("textbox", { name: "Title" });
        await titleInput.fill("Mobile Sibling");
        await modal.getByRole("button", { name: "Create Task" }).click();

        await plan.verifyTaskVisible("Mobile Sibling");
      });

      await test.step("Add child via context menu and verify auto-drill", async () => {
        const rootTitle = "Mobile Root";
        const childTitle = "Mobile Child";

        // Open context menu (Task actions)
        const rootRow = page
          .locator(`[data-testid="task-item"]`, { hasText: rootTitle })
          .first();
        await rootRow.getByLabel("Task actions").click();
        await page.getByRole("menuitem", { name: "Add Child" }).click();

        const modal = page.getByRole("dialog", { name: "Create Task" });
        await modal.getByRole("textbox", { name: "Title" }).fill(childTitle);
        await modal.getByRole("button", { name: "Create Task" }).click();

        // Verify Auto-drill (breadcrumb shows parent)
        await plan.mobileVerifyViewTitle(rootTitle);
        await plan.verifyTaskVisible(childTitle);
      });
    });
  });
});

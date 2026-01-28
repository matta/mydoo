import { test } from "../fixtures";

test.describe("Task Creation Defaults", () => {
  test.beforeEach(async ({ I }) => {
    // Background: Given I have a workspace seeded with sample data
    // (Existing background says seeded, but cleanWorkspace is safer for testing defaults)
    await I.When.switchToPlanView();
  });

  test("New tasks have correct default values", async ({ I }) => {
    // When I open the Create Task modal
    await I.When.opensCreateTaskModal();

    // Then I should see "Importance: 1.00"
    await I.Then.shouldSeeText("Importance: 1.00");

    // And I should see Lead Time "8" "Hours"
    await I.Then.shouldSeeLeadTime("8", "Hours");

    // And I should see the "Place" selector
    await I.Then.shouldSeeSelector("Place");

    // And I should see the "Schedule Type" selector
    await I.Then.shouldSeeSelector("Schedule Type");
  });

  test("Child tasks inherit defaults correctly", async ({ I }) => {
    // Given I have a task "Root Task for Defaults"
    await I.Given.taskExists("Root Task for Defaults");

    // When I add a child to "Root Task for Defaults"
    await I.When.opensAddChildModal("Root Task for Defaults");

    // Then I should see "Importance: 1.00"
    await I.Then.shouldSeeText("Importance: 1.00");

    // And I should see Lead Time "8" "Hours"
    await I.Then.shouldSeeLeadTime("8", "Hours");
  });
});

import { test } from "../fixtures";

test.describe("Sequential Projects", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.onHomePage();
  });

  test("Sequential tasks are blocked until previous sibling is done", async ({
    I,
  }) => {
    await I.Given.taskExists("Project Alpha");
    await I.Given.marksTaskAsSequential("Project Alpha");
    await I.Given.addsChildTask("Step 1", "Project Alpha");
    await I.Given.addsChildTask("Step 2", "Project Alpha");

    await I.Then.taskIsVisible("Step 1"); // Implicitly checks Do List if we switch view, but let's be explicit
    // The BDD said "visible in Do list". But we are currently in Plan view from addsChildTask.
    // Let's switch to Do view to match "visible in Do list" semantics
    await I.When.switchToDoView();
    await I.Then.taskIsVisible("Step 1");
    await I.Then.taskIsHidden("Step 2");

    await I.When.completesTaskFromDoList("Step 1");
    // Auto-refresh might handle it, or we need to refresh manually as per scenario
    await I.When.refreshesDoList();

    await I.Then.taskIsVisible("Step 2");
  });
});

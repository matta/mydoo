import { test } from "../fixtures";

test.describe("Plan Management", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.cleanWorkspace();
    await I.Given.seededWithSampleData();
  });

  test("Render task hierarchy", async ({ I }) => {
    await I.When.switchToPlanView();
    await I.Then.taskIsVisible("Project Alpha");
    await I.Then.taskIsVisible("Buy Groceries");
    await I.Then.taskIsHidden("Research Requirements");

    await I.When.expandsTask("Project Alpha");
    await I.Then.taskIsVisible("Research Requirements");
    await I.Then.taskIsVisible("Design UI Mocks");
  });

  test("Find in Plan from Do view", async ({ I }) => {
    await I.When.switchToDoView();
    await I.Then.taskIsVisible("Research Requirements");

    await I.When.findsInPlan("Research Requirements");
    await I.Then.shouldBeInPlanView();
    await I.Then.taskIsVisible("Project Alpha");
    await I.Then.taskIsVisible("Research Requirements");
    await I.Then.taskIsVisible("Design UI Mocks");
  });

  test("Edit task properties and persist", async ({ I }) => {
    await I.When.createTask("Task to Edit");
    await I.When.renamesTask("Task to Edit", "Edited Task Title");
    await I.Then.taskIsVisible("Edited Task Title");
    await I.Then.taskIsHidden("Task to Edit");

    await I.When.reloadsPage();
    await I.When.switchToPlanView();
    await I.Then.taskIsVisible("Edited Task Title");
  });

  test("Delete task with cascade", async ({ I }) => {
    await I.Given.taskWithChild("Parent Task", "Child Task");
    await I.When.deletesTask("Parent Task");
    await I.Then.taskIsHidden("Parent Task");
    await I.Then.taskIsHidden("Child Task");
  });

  test("Persist data across reloads", async ({ I }) => {
    await I.When.createTask("Persistent Task");
    await I.When.reloadsPage();
    await I.Then.shouldSeeInPlanView("Persistent Task");
  });
});

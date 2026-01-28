import { test } from "../fixtures";

test.describe("Add One Task", () => {
  test("Add task in Do view", { tag: "@add-one-task-do" }, async ({ I }) => {
    await I.Given.onHomePage();
    await I.When.switchToDoView();
    await I.When.createTaskInDoView("Add Task in Do View");
    await I.Then.taskIsVisible("Add Task in Do View");
  });

  test(
    "Add task in Plan view",
    { tag: "@add-one-task-plan" },
    async ({ I }) => {
      await I.Given.onHomePage();
      await I.When.switchToPlanView();
      await I.When.createTask("Add Task in Plan View");
      await I.Then.taskIsVisible("Add Task in Plan View");
    },
  );
});

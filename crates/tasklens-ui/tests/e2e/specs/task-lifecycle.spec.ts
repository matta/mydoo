import { test } from "../fixtures";

test.describe("Task Lifecycle", () => {
  test("Full Desktop Journey", async ({ I }) => {
    await I.When.switchToPlanView();
    await I.When.createsFirstTask("Desktop Root");
    await I.Then.taskIsVisible("Desktop Root");

    await I.When.renamesTask("Desktop Root", "Desktop Root Renamed");
    await I.Then.taskIsVisible("Desktop Root Renamed");
    await I.Then.taskIsHidden("Desktop Root");

    await I.When.addsSiblingTo("Desktop Sibling", "Desktop Root Renamed");
    await I.Then.taskIsVisible("Desktop Root Renamed");
    await I.Then.taskIsVisible("Desktop Sibling");

    await I.When.addsChild("Desktop Root Renamed", "Desktop Child");
    await I.Then.taskIsVisible("Desktop Child");
  });

  test("Basic Create and Complete", async ({ I }) => {
    await I.When.createTask("New E2E Task");
    await I.Then.taskIsVisible("New E2E Task");

    await I.When.completesTask("New E2E Task");
    await I.Then.shouldSeeMarkedAsCompleted("New E2E Task");

    await I.When.clearsCompletedTasks();
    await I.Then.taskIsHidden("New E2E Task");
  });

  test("Completed tasks stay visible until refresh", async ({ I }) => {
    await I.When.createTaskInDoView("Remediation Task");
    await I.Then.taskIsVisible("Remediation Task");

    await I.When.completesTask("Remediation Task");
    await I.Then.taskIsVisible("Remediation Task");

    await I.When.refreshesDoList();
    await I.Then.taskIsHidden("Remediation Task");
  });
});

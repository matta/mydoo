import { test } from "../fixtures";

test.describe("Routine Tasks", () => {
  test("Routine task reappears after lead time", async ({ I }) => {
    await I.Given.createsRoutineTask("Water Plants", "1 days", "12 hours");
    await I.When.switchToDoView();
    await I.Then.taskIsVisible("Water Plants");

    await I.When.completesTaskFromDoList("Water Plants");
    await I.Then.shouldSeeMarkedAsCompleted("Water Plants");

    await I.When.refreshesDoList();
    await I.Then.taskIsHidden("Water Plants");

    await I.When.waits("14 hours");
    await I.When.refreshesDoList();
    await I.Then.taskIsVisible("Water Plants");
  });
});

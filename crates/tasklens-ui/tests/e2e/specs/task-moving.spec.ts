import { test } from "../fixtures";

test.describe("Task Moving", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.cleanWorkspace();
    await I.Given.seededWithSampleData();
  });

  test("Move Task to Another Parent", async ({ I }) => {
    await I.Given.taskExists("Move Root");
    await I.Given.taskExists("Move Target");
    await I.Given.taskAsChildOf("Move Child", "Move Root");

    await I.When.movesTaskTo("Move Child", "Move Target");
    await I.When.switchToPlanView();
    await I.When.expandsTask("Move Target");
    await I.Then.taskIsVisible("Move Child");
  });

  test("Prevents Moving Task to Own Descendant (Cycle Detection)", async ({
    I,
  }) => {
    await I.Given.taskExists("Cycle Parent");
    await I.Given.taskAsChildOf("Cycle Child", "Cycle Parent");
    await I.Given.taskAsChildOf("Cycle Grandchild", "Cycle Child");

    await I.When.opensMovePickerFor("Cycle Parent");
    await I.Then.shouldSeeDisabledOrHiddenInMovePicker("Cycle Child");
    await I.Then.shouldSeeDisabledOrHiddenInMovePicker("Cycle Grandchild");
  });
});

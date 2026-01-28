import { test } from "../fixtures";

test.describe("Mobile Journeys", () => {
  // Mobile tests should be tagged or run with a mobile project config
  // For now, we rely on the project config to set the viewport

  test.beforeEach(async ({ I }) => {
    await I.Given.onHomePage();
    await I.Given.seededWithSampleData();
    await I.When.switchToPlanView();
  });

  test("Mobile Smoke Test", async ({ I }) => {
    await I.Then.shouldSeeMobileBottomBar();
  });

  test.skip("Add Child via Drill Down", async ({ I }) => {
    await I.When.drillsDownInto("Deep Work Project");
    await I.Then.viewTitleShouldBe("Deep Work Project");

    await I.When.createTask("Drill Child");
    await I.Then.taskIsVisible("Drill Child");

    await I.When.navigatesUpLevel();
    await I.Then.shouldSeeMobileBottomBar();
  });

  test.skip("Deep drill-down navigation", async ({ I }) => {
    await I.When.drillsDownInto("Deep Work Project");
    await I.When.drillsDownInto("Module A");
    await I.When.drillsDownInto("Component X");

    await I.Then.shouldSeeInBreadcrumbs("Deep Work Project");
    await I.Then.shouldSeeInBreadcrumbs("Module A");
    await I.Then.shouldSeeInBreadcrumbs("Component X");
  });
});

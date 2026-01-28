import { test } from "../fixtures";

test.describe("Smoke Test", () => {
  test("User launches the app", async ({ I }) => {
    // Given
    await I.Given.onHomePage();

    // Then
    await I.Then.pageTitleContains("TaskLens");
  });
});

import { test } from "../fixtures";

test.describe("Smoke Test", () => {
  test("User launches the app", async ({ I }) => {
    // Given

    // Then
    await I.Then.pageTitleContains("TaskLens");
  });
});

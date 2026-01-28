import { test } from "../fixtures";

test.describe("Document Binary Import/Export", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.onHomePage();
    await I.Given.documentExists();
  });

  test("Export and Import Preserves Document Identity", async ({ I, plan }) => {
    // Given
    await I.Given.taskExistsInView("Medieval Quest", "Plan");
    const oldId = await plan.getCurrentDocumentId();

    // When
    const filePath = await I.When.downloadsDocument();
    await I.When.clearsApplicationState();
    await I.When.uploadsDocument(filePath);

    // Then
    await I.Then.taskIsVisible("Medieval Quest");
    await I.Then.documentIdShouldRemain(oldId);
    await I.Then.documentUrlShouldUseSchema("automerge:");
  });
});

import { test } from "../fixtures";

test.describe("Document Switching", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.cleanWorkspace();
  });

  test("User creates a new document", async ({ I }) => {
    // Given
    await I.Given.onHomePage();
    const oldId = await I.Then.getCurrentDocumentId();

    // When
    await I.When.createsNewDocument();

    // Then
    await I.Then.documentIdChanges(oldId);
    await I.Then.documentShouldBeEmpty();
  });

  test("User switches to an existing document by ID", async ({ I }) => {
    // Given
    await I.Given.documentWithTask("A", "Task in A");
    await I.Given.documentWithTask("B", "Task in B");

    // When
    await I.When.switchesToDocument("A");

    // Then
    await I.Then.documentIdShouldBe("A");
    await I.Then.taskIsVisible("Task in A");

    // When
    await I.When.switchesToDocument("B");

    // Then
    await I.Then.documentIdShouldBe("B");
    await I.Then.taskIsVisible("Task in B");
  });
});

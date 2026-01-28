import { expect, test } from "../fixtures";

test.describe("Document Binary Import/Export", () => {
  test("Export from Alice and Import to Bob Preserves Document Identity", async ({
    // alice and bob are custom fixtures from ../fixtures.ts that provide
    // completely isolated browser contexts (incognito-like profiles).
    alice,
    bob,
  }) => {
    let oldId: string;
    let filePath: string;

    // 1. Alice creates a task and exports the document
    await test.step("Alice exports document", async () => {
      await alice.plan.goto("/");
      await alice.plan.createTask("Medieval Quest");
      const id = await alice.plan.getCurrentDocumentId();
      if (!id) throw new Error("Alice Doc ID is undefined");
      oldId = id;
      filePath = await alice.plan.downloadDocument();
    });

    // 2. Bob imports the document
    await test.step("Bob imports document", async () => {
      await bob.plan.goto("/");
      // Verify Bob starts empty
      await expect(bob.page.getByTestId("task-item")).toHaveCount(0);

      await bob.plan.uploadDocument(filePath);
    });

    // 3. Verify Bob has the data and the same ID
    await test.step("Bob verifies imported document", async () => {
      await bob.plan.verifyTaskVisible("Medieval Quest");
      const newId = await bob.plan.getCurrentDocumentId();
      expect(newId).toBe(oldId);

      const url = await bob.plan.getDetailedDocumentUrl();
      expect(url).toContain("automerge:");
    });
  });
});

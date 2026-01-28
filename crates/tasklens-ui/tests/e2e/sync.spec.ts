import { expect, test } from "./fixtures";

test.describe("End-to-End Synchronization", () => {
  test("Alice and Bob can sync changes", async ({ alice, bob, syncServer }) => {
    const syncUrl = `ws://localhost:${syncServer.getPort()}/sync`;

    // 1. Setup Alice
    let aliceDocId: string;
    await test.step("Setup Alice", async () => {
      await alice.plan.goto("/");
      await alice.plan.openSyncSettings();
      await alice.plan.setSyncServerUrl(syncUrl);
      await alice.plan.saveSyncSettings();
      await expect(alice.page.getByTestId("sync-status-button")).toContainText(
        "Connected",
        { timeout: 15000 },
      );

      await alice.plan.createTask("Sync Task");
      const id = await alice.plan.getCurrentDocumentId();
      expect(id).toBeDefined();
      if (!id) throw new Error("Alice Doc ID is undefined");
      aliceDocId = id;
    });

    // 2. Setup Bob (Join Alice's Document)
    await test.step("Setup Bob", async () => {
      await bob.plan.goto("/");
      await bob.plan.openSyncSettings();
      await bob.plan.setSyncServerUrl(syncUrl);
      await bob.plan.saveSyncSettings();
      await expect(bob.page.getByTestId("sync-status-button")).toContainText(
        "Connected",
        { timeout: 15000 },
      );

      await bob.plan.switchToDocument(aliceDocId);
    });

    // 3. One-way Sync: Alice -> Bob
    await test.step("Bob sees Alice's task", async () => {
      // Verify remote
      // Wait for sync propagation (allow more time for network + 500ms poll)
      await expect(
        bob.page.getByText("Sync Task", { exact: true }).first(),
      ).toBeVisible({ timeout: 15000 });
    });

    // 4. Two-way Sync: Bob -> Alice
    await test.step("Bob completes the task, Alice sees it", async () => {
      await bob.plan.completeTask("Sync Task");
      await bob.plan.verifyTaskCompleted("Sync Task"); // Verify local update first
      await alice.plan.verifyTaskCompleted("Sync Task");
    });
  });
});

import { expect, test } from "./fixtures";

test.describe("End-to-End Synchronization", () => {
  test("Alice and Bob can sync changes", async ({ alice, bob, syncServer }) => {
    const syncUrl = `ws://localhost:${syncServer.getPort()}/sync`;

    // 1. Setup Alice
    await test.step("Setup Alice", async () => {
      await alice.plan.primeWithSampleData();
      await alice.plan.waitForAppReady();
      await alice.plan.openSyncSettings();
      await alice.plan.setSyncServerUrl(syncUrl);
      await alice.plan.saveSyncSettings();
      await expect(alice.page.getByTestId("sync-status-button")).toContainText(
        "Connected",
        { timeout: 15000 },
      );
    });

    // 2. Setup Bob (Same Master Key -> Same DocID)
    await test.step("Setup Bob", async () => {
      await bob.plan.primeWithSampleData();
      await bob.plan.waitForAppReady();
      await bob.plan.openSyncSettings();
      await bob.plan.setSyncServerUrl(syncUrl);
      await bob.plan.saveSyncSettings();
      await expect(bob.page.getByTestId("sync-status-button")).toContainText(
        "Connected",
        { timeout: 15000 },
      );
    });

    // 3. One-way Sync: Alice -> Bob
    await test.step("Alice creates a task, Bob sees it", async () => {
      await alice.plan.createTask("Sync Task");
      // Verify local
      await alice.plan.verifyTaskVisible("Sync Task");
      // Verify remote
      // Wait for sync propagation (allow more time for network + 500ms poll)
      await expect(
        bob.page.getByText("Sync Task", { exact: true }).first(),
      ).toBeVisible({ timeout: 15000 });
    });

    // 4. Two-way Sync: Bob -> Alice
    await test.step("Bob completes the task, Alice sees it", async () => {
      await bob.plan.completeTask("Sync Task");
      await alice.plan.verifyTaskCompleted("Sync Task");
    });
  });
});

import { test } from "../fixtures";

test.describe("Sync Settings", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.cleanWorkspace();
  });

  test("User can change and persist sync server URL", async ({ I }) => {
    const customUrl = "ws://localhost:9999/sync";

    // 1. Change the URL
    await I.When.opensSyncSettings();
    await I.When.changesSyncServerUrl(customUrl);
    await I.When.savesSyncSettings();

    // 2. Verify it persisted after refresh
    await I.When.opensSyncSettings();
    await I.Then.syncServerUrlShouldBe(customUrl);
  });

  test("Sync settings modal can be closed without saving", async ({ I }) => {
    const customUrl = "ws://some-other-server/sync";

    await I.When.opensSyncSettings();
    await I.When.changesSyncServerUrl(customUrl);

    // Close by clicking the toggle button again
    await I.When.closesSyncSettings();

    // Re-open and verify it didn't change (as we didn't save)
    await I.When.opensSyncSettings();
    await I.Then.syncServerUrlShouldBe("ws://localhost:3000/sync");
  });
});

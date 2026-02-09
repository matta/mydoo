import { test } from "../fixtures";

test.describe("Sync Reconnection", () => {
  test("App reconnects after server restart", async ({ I }) => {
    // Given the user is connected to the sync server
    await I.When.connectsToSyncServer();
    await I.Then.syncStatusShouldBe("Connected");

    // When the sync server goes down
    await I.When.syncServerGoesDown();

    // Then the app should show it is disconnected
    await I.Then.syncStatusShouldNotBe("Connected");

    // When the sync server comes back up
    await I.When.syncServerComesBackUp();

    // Then the app should automatically reconnect
    await I.Then.syncStatusShouldBe("Connected");
  });
});

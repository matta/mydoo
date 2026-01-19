import * as net from "node:net";
import { expect, test } from "@playwright/test";
import { SyncServerHelper } from "./sync-server";

test.describe("SyncServerHelper", () => {
  // Use a random port between 4000-5000 to avoid conflicts during parallel execution
  const port = Math.floor(Math.random() * 1000) + 4000;
  const server = new SyncServerHelper(port);

  test.beforeAll(async () => {
    console.log("Building server...");
    await server.build();
    console.log("Server built.");
  });

  test.afterEach(async () => {
    await server.stop();
  });

  test("should start and accept TCP connections", async () => {
    await server.start();

    // Verify connection
    await expect
      .poll(async () => {
        return new Promise<boolean>((resolve) => {
          const socket = new net.Socket();
          socket.setTimeout(1000);
          socket.on("connect", () => {
            socket.destroy();
            resolve(true);
          });
          socket.on("error", () => {
            socket.destroy();
            resolve(false);
          });
          socket.on("timeout", () => {
            socket.destroy();
            resolve(false);
          });
          socket.connect(port, "127.0.0.1");
        });
      })
      .toBeTruthy();
  });
});

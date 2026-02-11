import { createServer, Socket } from "node:net";
import { expect, test } from "@playwright/test";
import { getEphemeralPort, SyncServerHelper } from "./sync-server";

test.describe("SyncServerHelper", () => {
  test.describe.configure({ mode: "serial" });

  let server: SyncServerHelper;

  test.beforeEach(async () => {
    const port = await getEphemeralPort();
    server = new SyncServerHelper(port);
  });

  test.afterEach(async () => {
    if (server) await server.stop();
  });

  test("should start and accept TCP connections", async () => {
    await server.start();

    // Verify connection
    await expect
      .poll(
        async () => {
          return new Promise<boolean>((resolve) => {
            const socket = new Socket();
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
            socket.connect(server.getPort(), "127.0.0.1");
          });
        },
        { timeout: 10000 },
      )
      .toBeTruthy();
  });

  test("getEphemeralPort returns a bindable TCP port", async () => {
    const port = await getEphemeralPort();
    const blocker = createServer();

    await new Promise<void>((resolve, reject) => {
      blocker.once("error", reject);
      blocker.listen(port, () => resolve());
    });

    await new Promise<void>((resolve, reject) => {
      blocker.close((err) => {
        if (err) {
          reject(err);
          return;
        }
        resolve();
      });
    });
  });

  test("stop waits for process exit so server can restart on the same port", async () => {
    await server.start();
    await server.stop();
    await server.start();

    await expect
      .poll(async () => {
        return new Promise<boolean>((resolve) => {
          const socket = new Socket();
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
          socket.connect(server.getPort(), "127.0.0.1");
        });
      })
      .toBeTruthy();
  });
});

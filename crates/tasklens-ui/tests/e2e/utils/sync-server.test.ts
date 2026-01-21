import * as net from "node:net";
import { expect, test } from "@playwright/test";
import { SyncServerHelper } from "./sync-server";

const getFreePort = async (): Promise<number> => {
  return new Promise((resolve, reject) => {
    const srv = net.createServer();
    srv.listen(0, () => {
      const port = (srv.address() as net.AddressInfo).port;
      srv.close((err) => {
        if (err) reject(err);
        else resolve(port);
      });
    });
    srv.on("error", reject);
  });
};

test.describe("SyncServerHelper", () => {
  test.describe.configure({ mode: "serial" });

  let server: SyncServerHelper;

  test.beforeEach(async () => {
    const port = await getFreePort();
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
            socket.connect(server.getPort(), "127.0.0.1");
          });
        },
        { timeout: 10000 },
      )
      .toBeTruthy();
  });
});

import { createServer, Socket } from "node:net";
import { expect, test } from "@playwright/test";
import { getEphemeralPort, SyncServerHelper } from "./sync-server";

const canConnect = async (port: number): Promise<boolean> => {
  return new Promise((resolve) => {
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
    socket.connect(port, "127.0.0.1");
  });
};

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

    await expect
      .poll(
        async () => {
          return canConnect(server.getPort());
        },
        { timeout: 10000 },
      )
      .toBeTruthy();
  });

  test("ensureHealthy restarts a crashed server process", async () => {
    await server.start();
    const pid = server.getPid();
    expect(pid).toBeDefined();
    if (pid === undefined) {
      throw new Error("Sync server pid is undefined after start");
    }

    process.kill(pid, "SIGKILL");

    await expect
      .poll(
        () => {
          return server.isRunning();
        },
        { timeout: 5000 },
      )
      .toBeFalsy();

    await server.ensureHealthy();

    await expect
      .poll(
        async () => {
          return canConnect(server.getPort());
        },
        { timeout: 10000 },
      )
      .toBeTruthy();
  });

  test("stop waits for process exit so restart can reuse the same port", async () => {
    await server.start();
    await server.stop();
    await server.start();

    await expect
      .poll(
        async () => {
          return canConnect(server.getPort());
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

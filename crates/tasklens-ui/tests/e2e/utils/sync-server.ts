import { type ChildProcess, spawn } from "node:child_process";
import fs from "node:fs";
import { createServer } from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Allocates a random available ephemeral TCP port from the OS.
 */
export const getEphemeralPort = async (): Promise<number> => {
  return await new Promise((resolve, reject) => {
    const server = createServer();

    server.once("error", (error) => {
      reject(error);
    });

    server.listen(0, () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("Could not allocate an ephemeral port"));
        return;
      }

      const { port } = address;
      server.close((closeErr) => {
        if (closeErr) {
          reject(closeErr);
          return;
        }
        resolve(port);
      });
    });
  });
};

/**
 * Waits until the child process exits.
 * Returns false when the timeout elapses before exit.
 */
const waitForChildExit = async (
  child: ChildProcess,
  timeoutMs: number,
): Promise<boolean> => {
  if (child.exitCode !== null || child.signalCode !== null) {
    return true;
  }

  return await new Promise((resolve) => {
    const onExit = () => {
      clearTimeout(timeoutHandle);
      resolve(true);
    };
    const timeoutHandle = setTimeout(() => {
      child.off("exit", onExit);
      resolve(false);
    }, timeoutMs);
    child.once("exit", onExit);
  });
};

export class SyncServerHelper {
  private serverProcess: ChildProcess | null = null;
  private port: number;
  private dbPath: string;

  constructor(port: number = 3030) {
    this.port = port;
    const randomSuffix = Math.random().toString(36).substring(2, 6);
    this.dbPath = path.resolve(
      `./test-scratch/test-sync-db-${port}-${randomSuffix}`,
    );
  }

  getPort() {
    return this.port;
  }

  async start(): Promise<string> {
    if (this.serverProcess) {
      throw new Error("Sync server is already running");
    }

    // Ensure clean state
    if (fs.existsSync(this.dbPath)) {
      fs.rmSync(this.dbPath, { recursive: true, force: true });
    }
    fs.mkdirSync(this.dbPath, { recursive: true });

    // utils -> e2e -> tests -> tasklens-ui -> crates -> mydoo/package.json
    // We need to point to mydoo/scripts/sync-server.js
    const scriptPath = path.resolve(
      __dirname,
      "../../../../../scripts/sync-server.js",
    );
    console.log(
      `Starting sync server: node ${scriptPath} --port ${this.port} --database-path ${this.dbPath}`,
    );

    this.serverProcess = spawn(
      "node",
      [
        scriptPath,
        "--port",
        this.port.toString(),
        "--database-path",
        this.dbPath,
      ],
      {
        stdio: ["ignore", "pipe", "pipe"],
      },
    );

    const startupMarker = `Sync server listening on port ${this.port}`;
    let startupResolved = false;
    let startupOutput = "";
    let startupErrorOutput = "";

    const startupPromise = new Promise<void>((resolve, reject) => {
      const timeoutHandle = setTimeout(() => {
        reject(
          new Error(
            `Timed out waiting for sync server startup on port ${this.port}. stdout: ${startupOutput.trim()} stderr: ${startupErrorOutput.trim()}`,
          ),
        );
      }, 10000);

      const cleanup = () => {
        clearTimeout(timeoutHandle);
        this.serverProcess?.stdout?.off("data", onStdout);
        this.serverProcess?.stderr?.off("data", onStderr);
        this.serverProcess?.off("exit", onExit);
      };

      const onStdout = (chunk: Buffer | string) => {
        const text = chunk.toString();
        startupOutput += text;
        if (process.env.SHOW_CONSOLE) {
          process.stdout.write(text);
        }
        if (!startupResolved && startupOutput.includes(startupMarker)) {
          startupResolved = true;
          cleanup();
          resolve();
        }
      };

      const onStderr = (chunk: Buffer | string) => {
        const text = chunk.toString();
        startupErrorOutput += text;
        if (process.env.SHOW_CONSOLE) {
          process.stderr.write(text);
        }
      };

      const onExit = (code: number | null, signal: NodeJS.Signals | null) => {
        if (startupResolved) {
          return;
        }
        cleanup();
        reject(
          new Error(
            `Sync server exited before startup (code=${code}, signal=${signal}). stderr: ${startupErrorOutput.trim()}`,
          ),
        );
      };

      this.serverProcess?.stdout?.on("data", onStdout);
      this.serverProcess?.stderr?.on("data", onStderr);
      this.serverProcess?.once("exit", onExit);
    });

    try {
      await startupPromise;
      console.log(`Sync server running on ws://localhost:${this.port}`);
    } catch (e) {
      console.error("Failed to wait for sync server port", e);
      await this.stop();
      throw e;
    }

    return `ws://localhost:${this.port}`;
  }

  async stop() {
    const child = this.serverProcess;
    this.serverProcess = null;

    if (child) {
      console.log("Stopping sync server...");
      child.kill("SIGINT");
      const stopped = await waitForChildExit(child, 5000);

      if (!stopped) {
        console.warn(
          "Sync server did not exit after SIGINT; forcing SIGKILL on process",
        );
        child.kill("SIGKILL");
        await waitForChildExit(child, 5000);
      }
    }

    // Clean up
    if (fs.existsSync(this.dbPath)) {
      fs.rmSync(this.dbPath, { recursive: true, force: true });
    }
  }
}

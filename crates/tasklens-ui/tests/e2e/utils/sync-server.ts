import { type ChildProcess, spawn } from "node:child_process";
import fs from "node:fs";
import { createServer } from "node:net";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import waitPort from "wait-port";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const STARTUP_TIMEOUT_MS = 10_000;
const HEALTHCHECK_TIMEOUT_MS = 750;
const EXIT_TIMEOUT_MS = 5_000;

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

export class SyncServerHelper {
  private serverProcess: ChildProcess | null = null;
  private readonly port: number;
  private readonly dbPath: string;
  private stopping = false;

  private readonly showLogs: boolean;

  constructor(port: number = 3030) {
    this.port = port;
    const randomSuffix = Math.random().toString(36).substring(2, 6);
    this.dbPath = path.resolve(
      `./test-scratch/test-sync-db-${port}-${randomSuffix}`,
    );
    this.showLogs = !!(
      process.env.SHOW_SYNC_SERVER || process.env.SHOW_CONSOLE
    );
  }

  getPort(): number {
    return this.port;
  }

  getPid(): number | undefined {
    return this.serverProcess?.pid;
  }

  isRunning(): boolean {
    const process = this.serverProcess;
    return process !== null && process.exitCode === null;
  }

  async restart(): Promise<string> {
    await this.stop();
    return this.start();
  }

  async ensureHealthy(): Promise<void> {
    if (!this.isRunning()) {
      if (this.showLogs) {
        console.warn(
          `[sync-server:${this.port}] Server is not running. Restarting.`,
        );
      }
      await this.start();
      return;
    }

    const isPortOpen = await this.waitForPort(HEALTHCHECK_TIMEOUT_MS);
    if (!isPortOpen) {
      if (this.showLogs) {
        console.warn(
          `[sync-server:${this.port}] Server process exists but port is unavailable. Restarting.`,
        );
      }
      await this.restart();
    }
  }

  async start(): Promise<string> {
    if (this.isRunning()) {
      const isPortOpen = await this.waitForPort(HEALTHCHECK_TIMEOUT_MS);
      if (isPortOpen) {
        return this.getBaseUrl();
      }
      if (this.showLogs) {
        console.warn(
          `[sync-server:${this.port}] Found stale process without open port. Recycling.`,
        );
      }
      await this.stop();
    }

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
    if (this.showLogs) {
      console.log(
        `Starting sync server: node ${scriptPath} --port ${this.port} --database-path ${this.dbPath}`,
      );
    }

    const child = spawn(
      "node",
      [
        scriptPath,
        "--port",
        this.port.toString(),
        "--database-path",
        this.dbPath,
      ],
      {
        stdio: this.showLogs ? "inherit" : "ignore",
      },
    );
    this.serverProcess = child;
    this.stopping = false;

    // Record unexpected process exits so test logs clearly show server crashes.
    child.on("exit", (code, signal) => {
      const codeText = code === null ? "null" : String(code);
      const signalText = signal ?? "none";
      if (this.stopping) {
        if (this.showLogs) {
          console.log(
            `[sync-server:${this.port}] Exited (code=${codeText}, signal=${signalText}).`,
          );
        }
      } else {
        console.error(
          `[sync-server:${this.port}] Exited unexpectedly (code=${codeText}, signal=${signalText}).`,
        );
      }

      if (this.serverProcess === child) {
        this.serverProcess = null;
      }
    });

    const isOpen = await this.waitForPort(STARTUP_TIMEOUT_MS);
    if (!isOpen) {
      console.error("Failed to wait for sync server port.");
      await this.stop();
      throw new Error(`Sync server failed to start on port ${this.port}`);
    }
    if (this.showLogs) {
      console.log(`Sync server running on ${this.getBaseUrl()}`);
    }

    return this.getBaseUrl();
  }

  async stop(): Promise<void> {
    if (this.serverProcess) {
      if (this.showLogs) {
        console.log("Stopping sync server...");
      }
      this.stopping = true;
      this.serverProcess.kill("SIGINT");
      await this.waitForExitOrForceKill(this.serverProcess);
      this.stopping = false;
      this.serverProcess = null;
    }

    if (fs.existsSync(this.dbPath)) {
      fs.rmSync(this.dbPath, { recursive: true, force: true });
    }
  }

  private getBaseUrl(): string {
    return `ws://localhost:${this.port}`;
  }

  private async waitForPort(timeoutMs: number): Promise<boolean> {
    const result = await waitPort({
      host: "localhost",
      port: this.port,
      timeout: timeoutMs,
      output: "silent",
    });
    return result.open;
  }

  private async waitForExitOrForceKill(process: ChildProcess): Promise<void> {
    if (process.exitCode !== null) {
      return;
    }

    const exited = await Promise.race<boolean>([
      new Promise<boolean>((resolve) => {
        process.once("exit", () => resolve(true));
      }),
      delay(EXIT_TIMEOUT_MS).then(() => false),
    ]);

    if (exited || process.exitCode !== null) {
      return;
    }

    console.warn(
      `[sync-server:${this.port}] SIGINT timed out after ${EXIT_TIMEOUT_MS}ms. Sending SIGKILL.`,
    );
    process.kill("SIGKILL");
    await new Promise<void>((resolve) => {
      process.once("exit", () => resolve());
    });
  }
}

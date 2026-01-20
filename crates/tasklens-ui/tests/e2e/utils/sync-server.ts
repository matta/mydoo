import { type ChildProcess, exec, spawn } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execAsync = promisify(exec);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export class SyncServerHelper {
  private port: number;
  private process: ChildProcess | null = null;
  private rootDir: string;

  constructor(port: number) {
    this.port = port;
    // __dirname is .../crates/tasklens-ui/tests/e2e/utils
    // root is 4 levels up: utils -> e2e -> tests -> tasklens-ui -> crates -> root
    this.rootDir = path.resolve(__dirname, "../../../../../");
  }

  getPort(): number {
    return this.port;
  }

  async build(): Promise<void> {
    console.log(`[SyncServer] Building binary at ${this.rootDir}...`);
    try {
      await execAsync("cargo build -p tasklens-sync-server", {
        cwd: this.rootDir,
      });
    } catch (e) {
      console.error("[SyncServer] Build failed:", e);
      throw e;
    }
  }

  async start(): Promise<void> {
    if (this.process) {
      console.warn("[SyncServer] Server already running.");
      return;
    }

    const binaryPath = path.join(
      this.rootDir,
      "target/debug/tasklens-sync-server",
    );
    if (!fs.existsSync(binaryPath)) {
      throw new Error(
        `Sync server binary not found at ${binaryPath}. Did you run build()?`,
      );
    }

    console.log(`[SyncServer] Starting server on port ${this.port}...`);

    // Use test-results directory for ephemeral DB files
    // __dirname is .../crates/tasklens-ui/tests/e2e/utils
    // Package root is ../../../
    const packageRoot = path.resolve(__dirname, "../../../");
    const testResultsDir = path.join(
      packageRoot,
      "test-results",
      "sync-server-db",
    );

    if (!fs.existsSync(testResultsDir)) {
      fs.mkdirSync(testResultsDir, { recursive: true });
    }

    const dbPath = path.join(testResultsDir, `updates_${this.port}.db`);

    // Clean up old db if exists
    if (fs.existsSync(dbPath)) {
      try {
        fs.unlinkSync(dbPath);
      } catch (e) {
        console.warn(`[SyncServer] Failed to cleanup old DB ${dbPath}:`, e);
      }
    }

    this.process = spawn(
      binaryPath,
      ["--port", this.port.toString(), "--database-path", dbPath],
      {
        env: { ...process.env, RUST_LOG: "info" },
        stdio: "pipe",
      },
    );

    const readyPromise = new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error("Timeout waiting for sync server to start"));
      }, 5000); // 5s timeout

      // We attach the listener before checking output
      const checkData = (data: Buffer | string) => {
        const str = data.toString();
        // Check for the log "listening on"
        if (str.includes("listening on")) {
          clearTimeout(timeout);
          resolve();
        }
      };

      this.process?.stdout?.on("data", checkData);
      this.process?.stderr?.on("data", checkData); // Sometimes logs go to stderr

      // Also reject if process exits early
      this.process?.on("exit", (code) => {
        clearTimeout(timeout);
        reject(new Error(`Sync server exited prematurely with code ${code}`));
      });
    });

    this.process.stdout?.on("data", (data) =>
      console.log(`[SyncServer stdout] ${data}`),
    );

    // Wait for readiness
    try {
      await readyPromise;
      console.log(`[SyncServer] Server ready on port ${this.port}`);
    } catch (e) {
      console.error(e);
      // Clean up if failed
      await this.stop();
      throw e;
    }
    this.process.stderr?.on("data", (data) =>
      console.error(`[SyncServer stderr] ${data}`),
    );

    this.process.on("error", (err) => {
      console.error("[SyncServer] Failed to start subprocess:", err);
    });

    this.process.on("exit", (code, signal) => {
      // Ignore expected kills
      if (code !== null && code !== 0 && code !== 137 && signal !== "SIGTERM") {
        console.error(
          `[SyncServer] Process exited with code ${code} signal ${signal}`,
        );
      }
      this.process = null;
    });
  }

  async stop(): Promise<void> {
    if (this.process) {
      console.log("[SyncServer] Stopping server...");
      this.process.kill("SIGTERM");
      // Ensure we clean up reference
      this.process = null;
      // Brief pause to allow OS to reclaim port
      await new Promise((r) => setTimeout(r, 100));
    }
  }
}

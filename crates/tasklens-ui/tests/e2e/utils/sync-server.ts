import { type ChildProcess, spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import waitPort from "wait-port";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export class SyncServerHelper {
  private serverProcess: ChildProcess | null = null;
  private port: number;
  private dbPath: string;

  constructor(port: number = 3030) {
    this.port = port;
    this.dbPath = path.resolve(`./test-sync-db-${port}`);
  }

  getPort() {
    return this.port;
  }

  async start(): Promise<string> {
    // Ensure clean state
    if (fs.existsSync(this.dbPath)) {
      fs.rmSync(this.dbPath, { recursive: true, force: true });
    }
    fs.mkdirSync(this.dbPath, { recursive: true });

    // utils -> e2e -> tests -> tasklens-ui -> crates -> mydoo/package.json
    // We need to point to mydoo/scripts/sync-server.mjs
    const scriptPath = path.resolve(__dirname, "../../../../../scripts/sync-server.mjs");
    console.log(`Starting sync server: node ${scriptPath} --port ${this.port} --database-path ${this.dbPath}`);

    this.serverProcess = spawn("node", [
      scriptPath,
      "--port",
      this.port.toString(),
      "--database-path",
      this.dbPath,
    ], {
      stdio: "inherit",
    });

    try {
      await waitPort({
        host: "localhost",
        port: this.port,
        timeout: 10000,
        output: "silent",
      });
      console.log(`Sync server running on ws://localhost:${this.port}`);
    } catch (e) {
      console.error("Failed to wait for sync server port", e);
      await this.stop();
      throw e;
    }

    return `ws://localhost:${this.port}`;
  }

  async stop() {
    if (this.serverProcess) {
      console.log("Stopping sync server...");
      this.serverProcess.kill("SIGINT");
      this.serverProcess = null;
    }
    // Clean up
    if (fs.existsSync(this.dbPath)) {
      fs.rmSync(this.dbPath, { recursive: true, force: true });
    }
  }
}


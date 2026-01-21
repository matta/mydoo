import fs from "node:fs";
import http from "node:http";
import { Repo } from "@automerge/automerge-repo";
import { WebSocketServerAdapter } from "@automerge/automerge-repo-network-websocket";
import { NodeFSStorageAdapter } from "@automerge/automerge-repo-storage-nodefs";
import { WebSocketServer } from "ws";

const args = process.argv.slice(2);
const portIdx = args.indexOf("--port");
const port = portIdx !== -1 ? parseInt(args[portIdx + 1], 10) : 3030;
const dbPathIdx = args.indexOf("--database-path");
const dbPath =
  dbPathIdx !== -1 ? args[dbPathIdx + 1] : "automerge-sync-server-data";

if (!fs.existsSync(dbPath)) {
  fs.mkdirSync(dbPath, { recursive: true });
}

const server = http.createServer((_req, res) => {
  res.writeHead(200, { "Content-Type": "text/plain" });
  res.end("Sync server is running\n");
});

const wss = new WebSocketServer({ noServer: true });
const config = {
  network: [new WebSocketServerAdapter(wss)],
  storage: new NodeFSStorageAdapter(dbPath),
  peerId: `sync-server-${port}`,
  sharePolicy: async () => true, // Share everything for tests
};
const _serverRepo = new Repo(config);

server.on("upgrade", (request, socket, head) => {
  wss.handleUpgrade(request, socket, head, (socket) => {
    wss.emit("connection", socket, request);
  });
});

server.listen(port, () => {
  console.log(
    `Sync server listening on port ${port} with database-path ${dbPath}`,
  );
});

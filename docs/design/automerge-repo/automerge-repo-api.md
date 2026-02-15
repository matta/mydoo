> **Note:** This document describes the design/protocol of the external `automerge-repo` package (Node.js) for reference purposes. It does not document the implementation within this repository.

# Automerge Repo API Reference

## Overview

The `@automerge/automerge-repo` library provides a facility for managing a collection of Automerge documents. It handles the "plumbing" of a local-first application, including:

- **Storage**: Persisting documents to local storage (IndexedDB, filesystem, etc.).
- **Networking**: Synchronizing documents with peers over various transports (WebSocket, MessageChannel, BroadcastChannel, etc.).
- **Synchronization**: Managing the sync protocol and document lifecycle.

The core architecture revolves around the **Repo**, which acts as the central manager, and **DocHandle**s, which provide access to individual documents. The system is designed to be agnostic to the underlying storage and network implementations, which are plugged in via adapters.

## Installation & Basic Usage

Install the package via npm or yarn:

```bash
npm install @automerge/automerge-repo
```

### Basic Integration

To use the library, instantiate a `Repo` with your desired storage and network adapters, then use it to create or find documents.

```typescript
import { Repo } from "@automerge/automerge-repo";
// Example adapters (sold separately or included in monorepo)
import { IndexedDBStorageAdapter } from "@automerge/automerge-repo-storage-indexeddb";
import { BrowserWebSocketClientAdapter } from "@automerge/automerge-repo-network-websocket";

// Initialize the Repo
const repo = new Repo({
  storage: new IndexedDBStorageAdapter("my-app-db"),
  network: [new BrowserWebSocketClientAdapter("wss://sync.example.com")],
});

// Create a new document
const handle = repo.create<{ count: number }>();

// Modify the document
handle.change((doc) => {
  doc.count = 0;
});

// Listen for changes
handle.on("change", ({ doc }) => {
  console.log("New count:", doc.count);
});
```

## Data Structures / Types

### `AutomergeUrl`

A string representing a unique identifier for an Automerge document, optionally including a specific version (heads). Format: `automerge:<base58check-encoded-document-id>`.

### `DocumentId`

A unique identifier for a document, encoded as a base58check string.

### `PeerId`

A string identifying a peer in the network.

### `RepoConfig`

Configuration object for the `Repo` constructor.

| Field              | Type                                   | Required | Description                                                                   |
| :----------------- | :------------------------------------- | :------- | :---------------------------------------------------------------------------- |
| `storage`          | `StorageAdapterInterface`              | No       | Adapter for persisting data. If omitted, data is in-memory only.              |
| `network`          | `NetworkAdapterInterface[]`            | No       | Array of network adapters for syncing. Default is `[]`.                       |
| `peerId`           | `PeerId`                               | No       | ID of this peer. Defaults to a random UUID-based ID.                          |
| `sharePolicy`      | `(peerId: PeerId) => Promise<boolean>` | No       | Function determining if we should share data with a peer. Default allows all. |
| `saveDebounceRate` | `number`                               | No       | Debounce time for saving to storage in ms. Default is 100.                    |

### `SyncInfo`

Information about the synchronization state of a document.

| Field               | Type       | Description                                 |
| :------------------ | :--------- | :------------------------------------------ |
| `lastHeads`         | `UrlHeads` | The heads of the document at the last sync. |
| `lastSyncTimestamp` | `number`   | Timestamp of the last sync.                 |

## API Reference

### Class: `Repo`

The `Repo` is the main entry point for the library. It manages the collection of documents and coordinates storage and networking.

#### `constructor(config: RepoConfig)`

Creates a new `Repo` instance.

| Parameter | Type         | Required | Description            |
| :-------- | :----------- | :------- | :--------------------- |
| `config`  | `RepoConfig` | No       | Configuration options. |

#### `create<T>(initialValue?: T): DocHandle<T>`

Creates a new document with an optional initial value and returns a handle to it.

| Parameter      | Type | Required | Description                    |
| :------------- | :--- | :------- | :----------------------------- |
| `initialValue` | `T`  | No       | Initial state of the document. |

**Returns:** `DocHandle<T>` - A handle to the newly created document.

**Example:**

```typescript
const handle = repo.create({ title: "New Doc" });
```

#### `find<T>(id: AutomergeUrl | DocumentId): Promise<DocHandle<T>>`

Locates a document by its ID. If the document is not in memory, it attempts to load it from storage or request it from peers.

| Parameter | Type                           | Required | Description                     |
| :-------- | :----------------------------- | :------- | :------------------------------ |
| `id`      | `AutomergeUrl` \| `DocumentId` | Yes      | The ID of the document to find. |

**Returns:** `Promise<DocHandle<T>>` - Resolves with the document handle when the document is ready (state is `ready`).

**Errors:**

- Throws if the document is `unavailable` (neither in storage nor on network) or `deleted`.
- Throws `AbortError` if operation is aborted via signal.

**Example:**

```typescript
const handle = await repo.find("automerge:2DK...");
```

#### `delete(id: DocumentId): void`

Deletes a document from the local repository and storage. This removes it from the synchronizer as well.

| Parameter | Type         | Required | Description                       |
| :-------- | :----------- | :------- | :-------------------------------- |
| `id`      | `DocumentId` | Yes      | The ID of the document to delete. |

#### `clone<T>(clonedHandle: DocHandle<T>): DocHandle<T>`

Creates a new `DocHandle` by cloning the history of an existing handle. The new document will have a unique ID but share the same change history.

| Parameter      | Type           | Required | Description                                      |
| :------------- | :------------- | :------- | :----------------------------------------------- |
| `clonedHandle` | `DocHandle<T>` | Yes      | The handle to clone. Must be in a `ready` state. |

**Returns:** `DocHandle<T>` - A new handle to the cloned document.

**Errors:**

- Throws if `clonedHandle` is not ready.

---

### Class: `DocHandle<T>`

A wrapper around a single Automerge document. It manages the document's lifecycle and allows reading/writing.

#### Properties

| Property     | Type           | Description                            |
| :----------- | :------------- | :------------------------------------- |
| `documentId` | `DocumentId`   | The unique identifier of the document. |
| `url`        | `AutomergeUrl` | The fully qualified Automerge URL.     |

#### `change(callback: (doc: T) => void, options?: ChangeOptions): void`

Modifies the document. The callback receives a mutable proxy of the document state.

| Parameter  | Type               | Required | Description                      |
| :--------- | :----------------- | :------- | :------------------------------- |
| `callback` | `(doc: T) => void` | Yes      | Function modifying the document. |
| `options`  | `ChangeOptions`    | No       | Options like commit message.     |

**Errors:**

- Throws if the handle is not `ready`.
- Throws if the handle is in view-only mode (fixed heads).

**Example:**

```typescript
handle.change((doc) => {
  doc.items.push("new item");
});
```

#### `doc(): T`

Returns the current immutable state of the document.

**Returns:** `T` - The document state.

**Errors:**

- Throws if the handle is not `ready`.

#### `isReady(): boolean`

Returns `true` if the document is loaded and ready for access.

#### `whenReady(states?: HandleState[]): Promise<void>`

Waits until the document enters one of the specified states (default: `['ready']`).

| Parameter | Type            | Required | Description                              |
| :-------- | :-------------- | :------- | :--------------------------------------- |
| `states`  | `HandleState[]` | No       | States to wait for. Default `['ready']`. |

**Errors:**

- Throws if the wait times out or the document becomes unavailable/deleted.

#### `view(heads: UrlHeads): DocHandle<T>`

Returns a specific version of the document at the given heads. The returned handle is read-only.

| Parameter | Type       | Required | Description                     |
| :-------- | :--------- | :------- | :------------------------------ |
| `heads`   | `UrlHeads` | Yes      | The heads defining the version. |

**Returns:** `DocHandle<T>` - A read-only handle at the specified version.

#### `history(): UrlHeads[]`

Returns the list of all historic heads (versions) of the document in topological order.

**Returns:** `UrlHeads[]` - Array of heads. Returns `undefined` if not ready.

#### `delete(): void`

Marks the document as deleted, removing it from storage and sync.

#### `broadcast(message: unknown): void`

Sends an ephemeral message to connected peers interested in this document. Not persisted.

| Parameter | Type      | Required | Description          |
| :-------- | :-------- | :------- | :------------------- |
| `message` | `unknown` | Yes      | The message payload. |

---

### Interface: `NetworkAdapterInterface`

Defines the contract for network adapters.

#### `connect(peerId: PeerId, peerMetadata?: PeerMetadata): void`

Initiates connection to the network.

#### `send(message: Message): void`

Sends a message to a specific peer.

#### `disconnect(): void`

Disconnects from the network.

#### Events

- `peer-candidate`: Emitted when a new peer is discovered.
- `peer-disconnected`: Emitted when a peer disconnects.
- `message`: Emitted when a message is received.

---

### Interface: `StorageAdapterInterface`

Defines the contract for storage adapters.

#### `load(key: StorageKey): Promise<Uint8Array | undefined>`

Retrieves data for a given key.

#### `save(key: StorageKey, data: Uint8Array): Promise<void>`

Saves data for a given key.

#### `remove(key: StorageKey): Promise<void>`

Deletes data for a given key.

#### `loadRange(keyPrefix: StorageKey): Promise<Chunk[]>`

Loads all data items whose keys start with the given prefix.

#### `removeRange(keyPrefix: StorageKey): Promise<void>`

Deletes all data items whose keys start with the given prefix.

---

### Utilities: `AutomergeUrl`

Helper functions for determining IDs and URLs.

#### `isValidAutomergeUrl(str: unknown): boolean`

Returns `true` if the string is a valid `automerge:` URL.

#### `parseAutomergeUrl(url: AutomergeUrl): ParsedAutomergeUrl`

Parses a URL into its `documentId` and optional `heads`.

#### `stringifyAutomergeUrl(options: { documentId: DocumentId, heads?: UrlHeads }): AutomergeUrl`

Constructs a valid Automerge URL from an ID and optional heads.

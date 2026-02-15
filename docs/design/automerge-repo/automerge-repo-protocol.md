> **Note:** This document describes the design/protocol of the external `automerge-repo` package (Node.js) for reference purposes. It does not document the implementation within this repository.

# Automerge Repo Network Protocol Specification

This document specifies the network protocol used by `automerge-repo` (v1.0). It is derived from a reverse engineering analysis of the source code.

## 1. Wire-Level Format

- **Transport:** WebSocket (preferred default), potentially others (MessageChannel, BroadcastChannel) but this spec covers the WebSocket implementation.
- **Framing:** WebSocket Framing. Each WebSocket message corresponds to one protocol message.
- **Encoding:** [CBOR (Concise Binary Object Representation)](https://cbor.io/).
  - Implementation uses `cbor-x`.
- **Endianness:** CBOR handles endianness (Big Endian standard).
- **Data Types:**
  - **DocumentId:** String (Base58Check encoded UUID).
  - **PeerId:** String.
  - **StorageId:** String.
  - **SessionId:** String.
  - **SyncMessage Payload:** `Uint8Array` (Automerge binary sync message).
  - **Heads:** Array of Strings (Base58Check encoded SHA-256 hashes).

## 2. Message Catalog

The protocol consists of two layers:

1.  **Transport Handshake** (WebSocket specific)
2.  **Repo Protocol** (General synchronization messages)

### 2.1. Transport Handshake (WebSocket)

These messages are exchanged immediately upon connection establishment.

#### `join` (Client -> Server)

Sent by the client to initiate the session.

- **type**: `"join"`
- **senderId**: `PeerId` (The client's Peer ID)
- **peerMetadata**: `PeerMetadata` (see below)
- **supportedProtocolVersions**: `["1"]`

#### `peer` (Server -> Client)

Sent by the server to accept the connection.

- **type**: `"peer"`
- **senderId**: `PeerId` (The server's Peer ID)
- **targetId**: `PeerId` (The client's Peer ID)
- **peerMetadata**: `PeerMetadata` (see below)
- **selectedProtocolVersion**: `"1"`

#### `error` (Server -> Client)

Sent by the server if the connection cannot be established (e.g., unsupported protocol version).

- **type**: `"error"`
- **senderId**: `PeerId`
- **targetId**: `PeerId`
- **message**: `String` (Description of the error)

#### **Data Structures**

**PeerMetadata**

```typescript
{
  storageId?: string;   // Unique ID of the peer's storage provider. If present, sync state is persisted.
  isEphemeral?: boolean; // If true, the peer does not persist data (e.g., a client browser tab).
}
```

### 2.2. Repo Protocol (General)

These messages are sent after the handshake is complete. They form the core synchronization logic. All messages are disjoint tagged unions discriminated by the `type` field.

#### `sync`

Carries an Automerge sync message to update a document.

- **type**: `"sync"`
- **senderId**: `PeerId`
- **targetId**: `PeerId`
- **documentId**: `DocumentId` (The document being synced)
- **data**: `Uint8Array` (The Automerge sync message payload)

#### `request`

Sent when a peer wants a document it does not have. Functionally identical to `sync` but signals intent to load.

- **type**: `"request"`
- **senderId**: `PeerId`
- **targetId**: `PeerId`
- **documentId**: `DocumentId`
- **data**: `Uint8Array` (Initial Automerge sync message, typically with no heads)

#### `ephemeral`

Carries transient application data (e.g., cursor positions, presence). Not persisted.

- **type**: `"ephemeral"`
- **senderId**: `PeerId`
- **targetId**: `PeerId`
- **documentId**: `DocumentId`
- **sessionId**: `SessionId` (Random session ID generated at startup)
- **count**: `number` (Sequence number)
- **data**: `Uint8Array` (Application specific payload)

#### `doc-unavailable`

Sent when a peer cannot satisfy a request for a document (it doesn't have it and thinks no one else does).

- **type**: `"doc-unavailable"`
- **senderId**: `PeerId`
- **targetId**: `PeerId`
- **documentId**: `DocumentId`

#### `remote-subscription-change` (Advanced)

Used for gossiping interest in documents (when "Remote Heads Gossiping" is enabled).

- **type**: `"remote-subscription-change"`
- **senderId**: `PeerId`
- **targetId**: `PeerId`
- **add**: `StorageId[]` (Optional: Storage IDs to add to subscription)
- **remove**: `StorageId[]` (Optional: Storage IDs to remove from subscription)

#### `remote-heads-changed` (Advanced)

Notifies a peer that a document's heads have changed on another peer (without sending the full sync message yet).

- **type**: `"remote-heads-changed"`
- **senderId**: `PeerId`
- **targetId**: `PeerId`
- **documentId**: `DocumentId`
- **newHeads**: `Map<StorageId, { heads: string[]; timestamp: number }>`

---

## 3. Protocol Grammar

### 3.1. Connection Establishment

1.  **Client** opens WebSocket connection.
2.  **Client** sends `join` message.
3.  **Server** validates `supportedProtocolVersions`.
    - If valid: Server sends `peer`. Connection is **Open**.
    - If invalid: Server sends `error` and closes socket.

### 3.2. Synchronization Flow

1.  **Discovery**:
    - When a document is opened or created, the `CollectionSynchronizer` determines which peers to share it with (based on `SharePolicy`).
    - Currently, this is often "sync all documents with all connected peers" or "sync based on request".

2.  **Initial Sync**:
    - If Peer A has doc and Peer B connects: Peer A sends `sync` message (generated by `Automerge.generateSyncMessage`).
    - If Peer A _wants_ doc from Peer B: Peer A sends `request` message.

3.  **Sync Loop**:
    - Received `sync` or `request` -> `Automerge.receiveSyncMessage`.
    - If result triggers a response (new heads or need): `Automerge.generateSyncMessage` -> Send `sync`.
    - This continues until both peers are in sync (heads match).

4.  **Unavailable Handling**:
    - If Peer A requests doc from Peer B.
    - Peer B checks local storage. Use `request` to ask its _other_ peers.
    - If Peer B finds it doesn't have it, and _all_ its connected peers return `doc-unavailable` or are also in "wants" state:
      - Peer B sends `doc-unavailable` to Peer A.

## 4. Semantics & Timing

- **Keep-Alive**:
  - The implementation uses standard WebSocket PING/PONG frames.
  - **Server**: Sends PINGs every `5000ms`.
  - **Server**: Terminates connection if client is "dead" (missed PONGs) after `5000ms` check interval.
  - **Client**: Relies on browser/OS WebSocket implementation for PING responses.
- **Retries**:
  - **Client**: If connection fails or closes, retries every `5000ms` (hardcoded).
- **Sync Debounce**:
  - Document changes are debounced by `100ms` before triggering sync messages.
- **Message Size**:
  - Zero-length messages are forbidden and will throw an error or close the connection.

## 5. Ambiguities & Inferences

- **Peer Roles**:
  - The protocol makes a distinction between "Client" and "Server" mainly for the **Handshake** phase (`join` vs `peer`).
  - After the handshake, the protocol is **Symmetric**. Both sides can send `sync`, `request`, etc.
  - However, `automerge-repo` implementation implies a topology where "Servers" often have `StorageAdapter`s and "Clients" (like browsers) might be `isEphemeral: true`.
- **Implicit Versioning**:
  - The code references `ProtocolV1 = "1"`. Usage of `supportedProtocolVersions` implies future negotiation capability, but currently only "1" is supported.
- **Magic Numbers**:
  - `retryInterval = 5000` (5s reconnect loop).
  - `syncDebounceRate = 100` (100ms buffering of ops).
  - `keepAliveInterval = 5000` (5s PING interval).

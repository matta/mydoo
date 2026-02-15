> **Note:** This document describes the protocol of the external Rust `samod` crate (https://github.com/alexjg/samod) for reference purposes. It does not document the implementation within this repository.

# SAMOD Sync Protocol Specification

This document specifies the network synchronization protocol used by the SAMOD repository. The protocol is designed to synchronize Automerge documents between peers.

## 1. Wire-Level Format

- **Transport:** The protocol is transport-agnostic but requires a message-oriented transport (e.g., WebSockets) or a framing layer that preserves message boundaries.
- **Encoding:** Messages are encoded using **CBOR (Concise Binary Object Representation)**.
- **Structure:** Each message is a CBOR Map containing a `type` field which determines the schema of the remaining fields.
- **Endianness:** CBOR standard (Network Byte Order / Big Endian).

### Common Types

- **PeerId:** String.
- **DocumentId:** String.
- **StorageId:** String.
- **Timestamp:** Unsigned 64-bit integer (milliseconds since Unix epoch).

## 2. Message Catalog

All messages are CBOR Maps. The `type` field is mandatory.

### Handshake Messages

#### `join`

Sent by the initiating peer to start the handshake.

- `type`: "join"
- `senderId`: [String] The PeerID of the sender.
- `supportedProtocolVersions`: [Array<String>] List of supported versions (e.g., `["1"]`).
- `metadata`: [Map] (Optional) Peer metadata.
  - `storageId`: [String] (Optional)
  - `isEphemeral`: [Boolean] (Default: false)

#### `peer`

Sent in response to a valid `join` message.

- `type`: "peer"
- `senderId`: [String] The PeerID of the responding peer.
- `targetId`: [String] The PeerID of the peer being addressed.
- `selectedProtocolVersion`: [String] The negotiated protocol version (e.g., "1").
- `metadata`: [Map] (Optional) Peer metadata (same structure as `join`).

### Sync Messages

#### `request`

Requests sync data for a specific document.

- `type`: "request"
- `senderId`: [String]
- `targetId`: [String]
- `documentId`: [String]
- `data`: [Bytes] Automerge sync message payload.

#### `sync`

Carries Automerge sync updates.

- `type`: "sync"
- `senderId`: [String]
- `targetId`: [String]
- `documentId`: [String]
- `data`: [Bytes] Automerge sync message payload.

#### `doc-unavailable`

Indicates the peer does not have the requested document.

- `type`: "doc-unavailable"
- `senderId`: [String]
- `targetId`: [String]
- `documentId`: [String]

### Control Messages

#### `leave`

Advisory message sent before disconnecting.

- `type`: "leave"
- `senderId`: [String]

#### `error`

Reports a protocol error.

- `type`: "error"
- `message`: [String] Description of the error.

#### `ephemeral`

Sends transient data (e.g., cursor positions, presence) related to a document.

- `type`: "ephemeral"
- `senderId`: [String]
- `targetId`: [String]
- `documentId`: [String]
- `sessionId`: [String] Identifier for the ephemeral session.
- `count`: [Uint64] Sequence number or counter.
- `data`: [Bytes] Application-specific payload.

#### `remote-heads-changed`

Notifies that a peer's document heads have ostensibly changed (gossip).

- `type`: "remote-heads-changed"
- `senderId`: [String]
- `targetId`: [String]
- `documentId`: [String]
- `newHeads`: [Map]
  - Key: `StorageId`
  - Value: [Map]
    - `heads`: [Array<String>] Base64 encoded SHA2 hashes.
    - `timestamp`: [Uint64]

#### `remote-subscription-change`

Updates the set of Storage IDs the sender is interested in.

- `type`: "remote-subscription-change"
- `senderId`: [String]
- `targetId`: [String]
- `add`: [Array<String>] (Optional) List of `StorageId`s to add.
- `remove`: [Array<String>] List of `StorageId`s to remove.

## 3. Protocol Grammar (State Machine)

The protocol uses a simple state machine to manage the connection lifecycle.

### States

1.  **WaitingForJoin** (Server/Receiver)
2.  **WaitingForPeer** (Client/Initiator)
3.  **Established**
4.  **Closed**

### Transitions

#### Initiator (Client)

1.  **Start** -> Send `join` -> **WaitingForPeer**
2.  **WaitingForPeer**:
    - Recv `peer` (with supported version) -> **Established**
    - Recv `peer` (unsupported version) -> Send `error` -> **Closed**
    - Recv Other -> Send `error` -> **Closed**

#### Receiver (Server)

1.  **Start** -> **WaitingForJoin**
2.  **WaitingForJoin**:
    - Recv `join` (supported version) -> Send `peer` -> **Established**
    - Recv `join` (unsupported version) -> Send `error` -> **Closed**
    - Recv Other -> Send `error` -> **Closed**

#### Established State

- **Allowed Messages:** `request`, `sync`, `doc-unavailable`, `ephemeral`, `remote-heads-changed`, `remote-subscription-change`.
- **Recv `join` / `peer`:** Protocol violation -> Send `error` -> **Closed**
- **Recv `leave`:** -> **Closed**
- **Recv `error`:** -> **Closed**

## 4. Semantics & Timing

- **Version Negotiation:** The protocol requires exact version matching. Currently, only version `1` is supported.
- **Error Handling:** Protocol errors result in an `error` message followed by immediate closure of the connection.
- **Keep-Alives:** There are no explicit heartbeat/ping messages defined in the wire protocol. Transport-level keep-alives (e.g., WebSocket PING/PONG) should be used if necessary.

## 5. Ambiguities & Inferences

- **Framing:** The code analyzes `WireMessage` which assumes a complete payload buffer. It is inferred that the underlying transport (e.g., WebSocket or a custom TCP framer) handles the delimiting of these CBOR payloads.
- **Timeouts:** No explicit timeout logic (e.g., "Handshake Timeout") was observed in the immediate protocol handler files. It is assumed to be managed by the parent actor or event loop.

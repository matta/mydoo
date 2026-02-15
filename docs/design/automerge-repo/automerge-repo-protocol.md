> **Note:** This document describes the design/protocol of the external `automerge-repo` package (Node.js) for reference purposes. It does not document the implementation within this repository.

# Automerge Repo Protocol Specification

This document specifies the network protocol used by `automerge-repo` (v1.0). It is derived from a reverse engineering analysis of the source code.

## 1. Overview

The protocol allows peers to discover, sync, and exchange documents. It runs over any duplex transport (WebSocket, MessageChannel, etc.). The protocol is message-oriented, with messages serialized as CBOR.

### 1.1 Key Concepts

- **Peer ID**: A unique identifier for a peer (string).
- **Document ID**: A unique identifier for a document (Base58 string, no prefix).
- **Channel ID**: A unique identifier for a communication channel (Document ID is used as Channel ID for document sync).
- **Session**: A connection between two peers.

## 2. Message Structure

Every message is a CBOR-encoded object. The top-level structure varies by message type, but generally follows:

```typescript
type Message =
  | JoinMessage
  | LeaveMessage
  | PeerCandidateMessage
  | SyncMessage
  | EphemeralMessage
  | RequestMessage
  | UnavailableMessage
  | RemoteSubscriptionControlMessage
  | RemoteHeadsChangedMessage
  | RemoteIssueMessage;
```

### 2.1 Common Fields

Most messages (except some control messages) are wrapped or identified by a `type` field.

### 2.2 Protocol Version

The protocol negotiation happens via the `JoinMessage`.

## 3. Message Types

### 3.1 Handshake Messages

These messages establish the session and capabilities.

#### `join` (Type: `join`)

Sent immediately upon connection to announce presence and supported protocol versions.

```typescript
interface JoinMessage {
  type: "join";
  senderId: PeerId;
  supportedProtocolVersions: string[]; // e.g., ["1"]
  metadata?: PeerMetadata; // Arbitrary JSON-serializable object
}
```

#### `peer-candidate` (Type: `peer-candidate`)

Sent to introduce other known peers to the connected peer (for mesh building).

```typescript
interface PeerCandidateMessage {
  type: "peer-candidate";
  senderId: PeerId;
  targetId: PeerId; // The peer being recommended
  location: string; // Connection string / URL
}
```

#### `leave` (Type: `leave`)

Sent before disconnecting to allow graceful cleanup.

```typescript
interface LeaveMessage {
  type: "leave";
  senderId: PeerId;
}
```

### 3.2 Sync Messages

These messages handle the actual synchronization of Automerge documents.

#### `sync` (Type: `sync`)

Carries the Automerge binary sync protocol payload.

```typescript
interface SyncMessage {
  type: "sync";
  senderId: PeerId;
  targetId: PeerId; // Recipient
  documentId: DocumentId;
  data: Uint8Array; // The raw Automerge sync message
}
```

### 3.3 Ephemeral Messages

Used for application-level broadcasting (e.g., cursors, presence) without persistence.

#### `ephemeral` (Type: `ephemeral`)

```typescript
interface EphemeralMessage {
  type: "ephemeral";
  senderId: PeerId;
  targetId: PeerId;
  count: number; // Sequence number
  channelId: ChannelId; // Usually DocumentId
  data: Uint8Array; // CBOR-encoded application payload
}
```

### 3.4 Document Availability & Control

Messages to request documents or signal availability.

#### `request` (Type: `request`)

Sent to request a document that the peer does not have.

```typescript
interface RequestMessage {
  type: "request";
  senderId: PeerId;
  targetId: PeerId;
  documentId: DocumentId;
}
```

#### `unavailable` (Type: `unavailable`)

Response to a `request` when the peer does not have the document.

```typescript
interface UnavailableMessage {
  type: "unavailable";
  senderId: PeerId;
  targetId: PeerId;
  documentId: DocumentId;
}
```

### 3.5 Remote Heads (Gossip)

Used to notify peers about document updates without sending the full sync state immediately.

#### `remote-subscription-change` (Type: `remote-subscription-change`)

Updates the list of documents a peer is interested in.

```typescript
interface RemoteSubscriptionControlMessage {
  type: "remote-subscription-change";
  senderId: PeerId;
  targetId: PeerId;
  add: DocumentId[]; // Start listening to these
  remove: DocumentId[]; // Stop listening to these
}
```

#### `remote-heads-changed` (Type: `remote-heads-changed`)

Notifies that a document has new heads (updates).

```typescript
interface RemoteHeadsChangedMessage {
  type: "remote-heads-changed";
  senderId: PeerId;
  targetId: PeerId;
  documentId: DocumentId;
  newHeads: {
    [documentId: DocumentId]: {
      heads: string[]; // Base58 encoded heads
      timestamp: number;
    };
  };
}
```

## 4. Connection Lifecycle

1.  **Transport Connection**: Transport (e.g., WebSocket) opens.
2.  **Handshake**: Both peers send `join`.
    - Peers validate `senderId`.
    - Peers negotiate protocol version (currently "1").
    - If negotiation fails, connection closes.
3.  **Session Active**:
    - Peers exchange `sync` messages for documents they both have.
    - Peers exchange `request` / `unavailable` / `sync` for missing docs.
    - Peers exchange `ephemeral` messages for presence.
4.  **Termination**:
    - Peer sends `leave`.
    - Transport closes.

## 5. Network Topologies

The protocol supports arbitrary topologies:

- **Client-Server**: Browser connects to a WebSocket server.
- **P2P Mesh**: Browsers connect via WebRTC (via BroadcastChannel or specialized adapters).
- **Gossip**: `peer-candidate` messages allow peers to discover others.

## 6. Implementation Notes

- **CBOR**: Use a standard CBOR encoder/decoder.
- **Binary Format**: Automerge sync messages are opaque binary blobs to this protocol; they are passed directly to the Automerge backend.
- **Error Handling**: Malformed messages should generally result in closing the connection to the offending peer.

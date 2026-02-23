# Sentinel Journal

## 2026-02-15 - [Input Validation Gaps in Domain Logic]

**Vulnerability:** Core domain logic (`Dispatch`) allowed creation of tasks and places with empty titles/names and effectively unbounded strings, relying on frontend validation which can be bypassed.
**Learning:** Local-first CRDT-based applications are vulnerable to bad data ingestion at the sync/domain layer. Compliance tests implicitly relied on empty defaults for missing fields.
**Prevention:** Enforce strict validation (length limits, non-empty checks) at the `Dispatch` layer before applying changes to the Automerge document.

## 2025-02-23 - [DocumentId Log Leakage]

**Vulnerability:** Automerge Document IDs (which are capability-based secrets) were exposed in full via `fmt::Debug` implementations.
**Learning:** In capability-based systems (like Automerge Repo), the ID _is_ the secret key. Standard `Debug` implementations often leak these secrets into logs.
**Prevention:** Implement custom `fmt::Debug` for sensitive types to redact the secret part (e.g., keeping only a prefix/suffix), while reserving `fmt::Display` for necessary serialization/URLs.

## 2026-03-01 - [Unbounded String Allocation in ID Types]
**Vulnerability:** `TaskID` and `PlaceID` types wrap `String` and accept arbitrarily long inputs, potentially causing resource exhaustion or performance degradation in the Automerge document.
**Learning:** Even though IDs are often generated (UUIDs), external actors or malicious clients can inject massive strings if the domain layer blindly accepts them.
**Prevention:** Enforce strict length limits (e.g., 100 chars) on all user-controlled string inputs, including IDs, at the dispatch/action handler level.

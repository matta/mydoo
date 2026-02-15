# Sentinel Journal

## 2026-02-15 - [Input Validation Gaps in Domain Logic]
**Vulnerability:** Core domain logic (`Dispatch`) allowed creation of tasks and places with empty titles/names and effectively unbounded strings, relying on frontend validation which can be bypassed.
**Learning:** Local-first CRDT-based applications are vulnerable to bad data ingestion at the sync/domain layer. Compliance tests implicitly relied on empty defaults for missing fields.
**Prevention:** Enforce strict validation (length limits, non-empty checks) at the `Dispatch` layer before applying changes to the Automerge document.

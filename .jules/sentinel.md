# Sentinel Journal

## 2026-02-15 - [Input Validation Gaps in Domain Logic]

**Vulnerability:** Core domain logic (`Dispatch`) allowed creation of tasks and places with empty titles/names and effectively unbounded strings, relying on frontend validation which can be bypassed.
**Learning:** Local-first CRDT-based applications are vulnerable to bad data ingestion at the sync/domain layer. Compliance tests implicitly relied on empty defaults for missing fields.
**Prevention:** Enforce strict validation (length limits, non-empty checks) at the `Dispatch` layer before applying changes to the Automerge document.

## 2025-02-23 - [DocumentId Log Leakage]

**Vulnerability:** Automerge Document IDs (which are capability-based secrets) were exposed in full via `fmt::Debug` implementations.
**Learning:** In capability-based systems (like Automerge Repo), the ID _is_ the secret key. Standard `Debug` implementations often leak these secrets into logs.
**Prevention:** Implement custom `fmt::Debug` for sensitive types to redact the secret part (e.g., keeping only a prefix/suffix), while reserving `fmt::Display` for necessary serialization/URLs.

## 2025-02-24 - [Verbose Logging in Production]

**Vulnerability:** The application was initialized with `tracing::Level::INFO` unconditionally, which could leak sensitive information (user data, state internals) to the browser console in production builds.
**Learning:** Default logging configurations often favor development convenience over security. Explicit conditional compilation is required to ensure production builds are quiet.
**Prevention:** Use `cfg!(debug_assertions)` to toggle log levels or feature flags to strip logging calls entirely in release builds.

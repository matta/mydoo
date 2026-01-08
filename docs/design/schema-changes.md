# Architecture Decision: Versionless Convergence

**Status:** Adopted | **Context:** Local-First / Distributed Data (Automerge)

## 1. Problem: Distributed Schema Evolution

Centralized "stop-the-world" migrations fail in local-first systems due to:

1.  **No Central Time:** No single moment for simultaneous client upgrades.
2.  **Zombie Data:** Offline clients running old code may sync obsolete field
    structures indefinitely.
3.  **Mixed Versions:** Documents concurrently house structures from multiple
    schema versions.

Enforcing rigid version numbers causes **Semantic Inversion** (misinterpreting
old data) and **Data Loss** (overwriting future data).

## 2. Strategy: Soup of Fields

We treat documents as a fluid **"Soup of Fields"** rather than monolithic
versioned structures. The application acts as a **Lens**, adapting to and
interpreting whichever fields are present.

## 3. Protocol: "Do No Harm"

Clients must strictly adhere to non-destructive I/O patterns.

### Rule A: Surgical Writes

**Never** overwrite entity objects entirely. **Only** modify specific fields.

- ❌ **Destructive:** `doc.tasks[id] = { ...task, title: "New" }` (Erases
  unknown fields)
- ✅ **Surgical:** `doc.tasks[id].title = "New"` (Preserves unknown fields)

### Rule B: Permissive Reads

Runtime validation **must not** reject unknown fields. Schemas must behave as
data carriers for other versions.

- **Implementation:** Use `.passthrough()` on all persistence schemas (e.g.,
  Zod).

## 4. Convergence (Continuous Migration)

Migration is defined as a **Continuous Convergence Function**:

- **Idempotent:** Safe to execute repeatedly.
- **Read-Repair:** Runs upon load or specific triggers.
- **Convergent:** Aggressively prunes deprecated fields by folding them into the
  canonical schema (e.g., merging `notes` into `description`).

## 5. Key Definitions

| Term          | Traditional (SQL)     | Local-First (Automerge)                                   |
| :------------ | :-------------------- | :-------------------------------------------------------- |
| **Version**   | Structural guarantee. | **Target State Indicator**: Heuristic for the "Lens".     |
| **Migration** | One-time transition.  | **Cleanup Crew**: Idempotent, continuous data pruning.    |
| **Lens**      | N/A                   | **Permanent Code**: Translates raw doc state to app view. |

## 6. Verification Strategy

### Tier 1: Round-Trip Fidelity (Future-Proofing)

**Goal:** Ensure `App_V(n)` preserves data created by `App_V(n+1)`. **Method:**
Unit test where a client loads a document containing unknown fields, performs a
write, and asserts the unknown fields remain intact.

### Tier 2: Golden Master (Backward Compatibility)

**Goal:** Ensure `App_V(n)` can open documents from `App_V(n-1)`. **Method:**
Commit `.automerge` binary snapshots from each major version. Test suite
verifies the current app successfully loads and converges these binaries.

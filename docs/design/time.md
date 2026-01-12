# Recommended Time Handling Approach

Based on the [Time Landscape](./time-landscape.md) and our
[Product Requirements](./prd.md), this document defines the standard approach
for handling time in our codebase.

## 1. Core Data Types

We strictly separate **Absolute History** (what happened) from **Civil Intent**
(what _should_ happen).

### A. Absolute Timestamp (History)

Use for all records of past events, system logs, and synchronization states.

- **Format:** **Unix Milliseconds** (`number`).
- **Why:**
  - Matches our database (Automerge/IndexedDB) and PRD requirements (`lastDone`,
    `created_at`).
  - Compact, unambiguous, and trivially sortable.
  - Avoids the "Leap Second" complexity (smearing is acceptable for our domain).
- **PRD usage:** `lastReviewTimestamp`, `lastDone`, `schedule.due` (for _fixed_
  deadlines where the instant is certain).

### B. Civil DateTime (Future Planning)

Use for future scheduling, recurring routines, and user-facing dates.

- **Format:** **ISO 8601 String** (No Offset) + **Zone ID**.
  - Example: `{ date: "2026-01-12T09:00:00", zone: "America/New_York" }`
- **Why:**
  - avoids the [RFC 9557 Semantic Trap](./time-landscape.md#rfc-9557) where
    offsets might conflict with future political changes.
  - Preserves user intent ("9 AM") even if DST rules change.
- **PRD usage:** `schedule.due` (when floating), Calendar appointments.

---

## 2. Serialization & Storage

### In Memory (Runtime)

- **Absolute Timestamp:** Use **`number`** (Unix Millis).
  - _Naming Convention:_ Suffix with `Timestamp` (e.g., `createdTimestamp`,
    `lastReviewTimestamp`, `lastDoneTimestamp`).
  - _Usage:_ Pass numbers around. Instantiate `Date`/`Temporal` only for
    math/formatting.

- **Duration:** Use **`number`** (Milliseconds).
  - _Naming Convention:_ Suffix with `Ms` (e.g., `timeoutMs`, `leadTimeMs`) to
    distinguish quantities of time from points in time.

### In Persistence (Automerge)

- **Timestamps:** Store as `Int` (Unix Millis).
- **Zoned Times:** Store as two separate fields:

  ```typescript
  {
    iso8601: "2026-01-12T09:00:00", // ISO 8601 (Civil DateTime, no offset)
    zone: "America/New_York"    // IANA Zone ID
  }
  ```

  - _Do not_ store as a combined string with offset to avoid "Physics Dominant"
    parser accidents.

---

## 3. Ambiguity handling Policy

When converting **Civil Intent** to **Absolute Time** (e.g., calculating the
next occurrence of a routine), we encounter
[Ambiguity Scenarios](./time-landscape.md#ambiguity-scenarios-civil--zoned).

**Policy:** We adopt the **"Compatible"** standard (matching Java/Temporal
defaults).

1.  **Overlaps (Fall Back):**
    - _Scenario:_ Clock goes 1:59 -> 1:00. Time happens twice.
    - _Action:_ Pick the **Earlier** occurrence (the first pass).
    - _Why:_ Preserves monotonicity; ensures the task doesn't "jump" to the
      future unexpectedly.

2.  **Gaps (Spring Forward):**
    - _Scenario:_ Clock goes 1:59 -> 3:00. Time never happens.
    - _Action:_ **Shift Forward** by the gap duration.
    - _Why:_ If I set a task for 2:30 AM, and 2:30 AM disappears, 3:30 AM is the
      logical semantic equivalent (preserving the interval).

---

## 4. Implementation Guidelines

### Do's

- **DO** use Unix Millis for `lastDone`, `createdTimestamp`, and internal logic.
- **DO** use libraries (`date-fns-tz` or `Temporal`) for all Zoned math. Never
  do math on `Date` objects manually.
- **DO** explicit handling of Time Zones for future events.

### Don'ts

- **DON'T** use `Date.parse()` on strings without reliable offsets.
- **DON'T** trust client-side local time for shared deadlines; always normalize
  to UTC or an explicit Zone.
- **DON'T** use "Unix Seconds" (confusion risk with Millis). Standardize on
  Millis.

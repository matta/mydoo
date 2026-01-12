# Fundamental Time Concepts

This document classifies the **five** fundamental data types required to handle
time correctly in software. These definitions separate **physics** (absolute
time) from **civilization** (calendars and politics). Unless otherwise noted,
all "Civil" types assume the **ISO 8601 / Gregorian** calendar system.

## 1. Absolute Timestamp

**Definition:** A specific, precise instant on the universal timeline. It
represents a count of time units (e.g. milliseconds or nanoseconds) since the
Unix Epoch (1970-01-01 00:00:00 UTC). It has no concept of "Tuesday" or "New
York"; it is just a number on a physics timeline.

- **Use for:** System logs, git commits, database `created_at` fields, event
  ordering.
- **Never use for:** Future events (meetings), birthdays, or anything dependent
  on human laws (time zones).

## 2. Civil Date

**Definition:** A generic date on a wall calendar (Year-Month-Day), specifically
in the **ISO 8601 / Gregorian** calendar system. It is not a specific moment in
time because it begins at different instants depending on where you are on
Earth.

- **Use for:** Birthdays, holidays, "Due Today" tasks, tax deadlines.
- **Never use for:** Timestamps or precise coordination between geographies.

## 3. Civil DateTime

**Definition:** A date and time on a wall clock (ISO 8601 / Gregorian), detached
from any location. It represents the abstract idea of "9:00 AM" but not "9:00 AM
_here_."

- **Use for:** Floating alarms ("Wake up at 7:00 AM"), store opening hours
  ("Opens at 9:00 AM").
- **Never use for:** Converting to a timestamp without user input (requires
  asking "Where are you?").

## 4. Zoned Instant (Physics Dominant)

**Definition:** A Timestamp coupled with a Timezone. The timestamp is the source
of truth.

- **Structure:** `Timestamp + Timezone`
- **Truth:** The _Timestamp_ is fixed.
- **Behavior:** If timezone rules change (politics), the _civil_ display
  changes, but the instant remains the same.
- **Use for:** Historical logs ("The server crashed at `<timestamp>` in NYC").

## 5. Zoned Civil (Civil Dominant)

**Definition:** A Civil DateTime coupled with a Timezone. The wall-clock time is
the source of truth.

- **Structure:** `Civil DateTime + Timezone`
- **Truth:** The _Civil Time_ is fixed.
- **Behavior:** If timezone rules change, the underlying _timestamp_ is
  recalculated to preserve the "9:00 AM" civil intent.
- **Use for:** Future appointments, flights.

## Leap Seconds & Atomic Time

All "Absolute Timestamps" discussed here are based on **Unix Time** (POSIX),
which assumes a uniform day length of 86,400 seconds.

- **Leap Seconds are ignored:** When an atomic leap second occurs (e.g.
  23:59:60), Unix time protocols typically "smear" the extra second or repeat
  the last second.
- **Application View:** Software sees a continuous, monotonic timeline. You will
  never encounter `23:59:60` in these data types.

## Ambiguity Scenarios (Civil → Zoned)

When converting a **Civil DateTime** to a **Zoned Instant**, the wall clock time
may not exist or may happen twice due to **Offset Changes** (e.g. daylight
savings time transitions or political time zone changes).

- **Overlaps (Fall Back):** The clock jumps backward (e.g. 1:59 -> 1:00),
  causing 1:30 AM to occur **twice**.
  - _Resolutions:_ **Earlier** (First pass), **Later** (Second pass), **Reject**
    (Error).
- **Gaps (Spring Forward):** The clock jumps forward (e.g. 1:59 -> 3:00), so
  2:30 AM **never happens**.
  - _Resolutions:_ **Shift Forward** (Advance civil time by gap duration, e.g.
    2:30 becomes 3:30), **Reject** (Error). There is also **Advance to Next
    Valid** (e.g. 2:30 becomes 3:00), which is a valid strategy that is used by
    some systems such as Unix `cron` but is not explicitly supported by most
    libraries.
- **"Compatible" Behavior:** The industry standard default (Java, Temporal,
  etc).
  - _Overlaps:_ Picks **Earlier**.
  - _Gaps:_ Picks **Shift Forward**.

## Implementation Guide

### Common Encodings (Examples)

_These are common formats found in the ecosystem, not a strict prescription._

- **Absolute Timestamp**:
  - **ISO 8601 (UTC)** `2026-01-12T17:00:00Z`
  - **Unix Seconds** `1768237200`
  - **Unix Millis** `1768237200000`
- **Civil Date**: **ISO 8601 (Date)** `2026-01-12`
- **Civil DateTime**: **ISO 8601 (No Offset)** `2026-01-12T17:00:00`
- **Zoned Instant**:
  - **RFC 9557** `2026-01-12T17:00:00-05:00[New_York]`
    > **Trap:** This format introduces **Semantic Ambiguity**. Because different
    > parsers prioritize different parts of the string (Offset vs Zone), the
    > _same string_ can yield _different times_ in different systems.
    >
    > - **Standard Parsers (Physics):** Trust Offset. `09:00` becomes `08:00`.
    > - **Smart Parsers (Civil):** Trust Zone. `09:00` stays `09:00`.
    > - **Result:** Loss of interoperability, potential for bugs.
  - **ISO 8601 (Offset Only)** `2026-01-12T17:00:00-05:00` (Lossy: Missing IANA
    Zone ID)
  - **Separate Fields**
    `{ timestamp: "2026-01-12T22:00:00Z", zone: "America/New_York" }`
- **Zoned Civil**: **Separate Fields** `Civil ISO` + `Zone ID`

### JavaScript (Temporal)

- **Absolute Timestamp**: `Temporal.Instant`
- **Civil Date**: `Temporal.PlainDate`
- **Civil DateTime**: `Temporal.PlainDateTime`
- **Zoned Instant**: `Temporal.ZonedDateTime`
- **Zoned Civil**: _Composite_ `PlainDateTime` + `TimeZone`
  > **Ambiguity Handling (Civil → Zoned):** Fully specified.
  >
  > - `disambiguation: 'compatible'` (Default): Uses "Compatible" behavior.
  > - `'earlier'` / `'later'` / `'reject'`: Explicit control.

### JavaScript (Legacy Date)

- **Absolute Timestamp**: `Date` (Acts as timestamp, but API mimics Civil)
- **Civil Date**: `String` (ISO 8601 `YYYY-MM-DD`)
- **Civil DateTime**: `String` (ISO 8601 `YYYY-MM-DDTHH:mm:ss`)
- **Zoned Instant**: _N/A_ (Requires library)
- **Zoned Civil**: _Composite_ `String` (ISO 8601) + `String` (IANA Zone ID)
  > **Ambiguity Handling (Civil → Zoned):** Implicitly **Compatible**.
  >
  > - Modern engines (V8, SpiderMonkey) default to "Compatible" behavior.
  > - Legacy/Older engines: Implementation defined.
  > - **Control:** To avoid reliance on implicit defaults, use a library (e.g.
  >   `date-fns-tz`).

### Rust (Jiff)

- **Absolute Timestamp**: `jiff::Timestamp`
- **Civil Date**: `jiff::civil::Date`
- **Civil DateTime**: `jiff::civil::DateTime`
- **Zoned Instant**: `jiff::Zoned`
- **Zoned Civil**: _Composite_ `civil::DateTime` + `TimeZone`
  > **Ambiguity Handling (Civil → Zoned):** Explicit. `to_zoned()` accepts a
  > `Disambiguation` strategy (`Compatible`, `Earlier`, `Later`, `Reject`).

### Rust (Chrono)

- **Absolute Timestamp**: `DateTime<Utc>`
- **Civil Date**: `NaiveDate`
- **Civil DateTime**: `NaiveDateTime`
- **Zoned Instant**: `DateTime<Tz>`
- **Zoned Civil**: _Composite_ `NaiveDateTime` + `Tz`
  > **Encoding Violated:** Defaults to ISO 8601 Offset-only (e.g. `-05:00`).
  > Lacks the `[Zone/ID]` bracket syntax required by RFC 9557. **Ambiguity
  > Handling (Civil → Zoned):** Manual. `from_local_datetime()` returns a
  > `LocalResult` enum (`Single`, `Ambiguous`, `None`). You must manually map
  > these cases to your desired resolution.

### C++ (CCTZ)

- **Absolute Timestamp**: `std::chrono::time_point` (CCTZ extends this)
- **Civil Date**: `cctz::civil_day`
- **Civil DateTime**: `cctz::civil_second`
- **Zoned Instant**: _Composite_ `time_point` + `time_zone`
- **Zoned Civil**: _Composite_ `civil_second` + `time_zone`
  > **Encoding:** No default serialization. Must use `cctz::format()` with
  > explicit format strings. **Note:** CCTZ refuses to provide a "Zoned
  > DateTime" type. It forces you to choose: `time_point` (Physics) or
  > `civil_second` (Civil). You convert using a `time_zone` object (e.g.,
  > `cctz::convert(tp, nyc)`). **Ambiguity Handling (Civil → Zoned):** Manual.
  > `cctz::lookup()` returns a struct with `kind` (`UNIQUE`, `SKIPPED`,
  > `REPEATED`) and both candidates (`pre`, `post`). You must manually choose
  > one.

### Google Cloud API (Protobuf)

- **Absolute Timestamp**: `google.protobuf.Timestamp` (Always UTC)
- **Civil Date**: `google.type.Date`
  > **Encoding Violated:** Standard Protobuf JSON mapping serializes as a JSON
  > Object `{year: Y, month: M, day: D}`, not an ISO 8601 string.
- **Civil DateTime**: `String` (ISO 8601 `YYYY-MM-DDTHH:mm:ss`)
- **Zoned Instant**: _Composite_ `Timestamp` + `TimeZone` string
- **Zoned Civil**: _Composite_ `Date` + `Time` + `TimeZone`
  > **Note:** Google APIs enforce a strict separation. Zoned logic requires
  > storing `Timestamp` and `TimeZone` string in separate fields. **Ambiguity
  > Handling (Civil → Zoned):** N/A. This is a data protocol. Ambiguity must be
  > resolved by the client's language (e.g. Temporal/Jiff) _before_
  > serialization.

# Post-Mortem: Playwright Browser Snapshot Setup

**Date:** 2026-02-12
**PR:** [#317](https://github.com/matta/mydoo/pull/317)
**Status:** No measurable improvement -- PR should be reverted or closed.

## Hypothesis

Starting each E2E test with a fresh browser context is slow because every
test must:

1. Fetch all static assets (WASM bundle, CSS, HTML, JS) over HTTP.
2. Compile the WASM module.
3. Initialize the app and wait for `[data-app-state="ready"]`.
4. Seed sample data via `tasklensSeedSampleData()` (for tests that need it).

We hypothesized that pre-generating browser snapshots (IndexedDB state via
`storageState` and static assets via HAR recording) would eliminate steps
1, 2, and 4, producing a significant speedup.

## What We Built

A `snapshot-setup` Playwright project that runs before all test projects,
generating three artifacts:

- `empty-db.json` -- IndexedDB snapshot of a freshly initialized app.
- `sample-db.json` -- IndexedDB snapshot with sample data pre-seeded.
- `app-assets.har` -- HAR recording of all static assets for replay.

Tests declared their database profile via `test.use({ db: "sample" })`.
The fixture system restored the matching IndexedDB snapshot and replayed
static assets from the HAR file using `routeFromHAR`.

### Playwright APIs Used

- `browserContext.storageState({ indexedDB: true })` (v1.51+) to snapshot
  and restore cookies, localStorage, and IndexedDB.
- `browserContext.routeFromHAR()` to record and replay HTTP responses.
- Project dependencies to orchestrate setup before test execution.

## Results

Three runs of each configuration, `e2e-desktop` project only:

| Scenario                         | Run 1 | Run 2 | Run 3 | Avg   |
| -------------------------------- | ----- | ----- | ----- | ----- |
| Baseline (no snapshots)          | 43.9s | 43.8s | 44.0s | 43.9s |
| Optimized cold (generate + use)  | 43.1s | 43.6s | 51.3s | 46.0s |
| Optimized warm (reuse snapshots) | 43.7s | 43.8s | 43.6s | 43.7s |

Per-test times for sample-data specs were also identical across both
configurations (300--800ms range).

**No measurable speedup.**

## Root Cause Analysis

### Why `storageState` didn't help

`storageState({ indexedDB: true })` restores the database contents, but
the app still must:

1. Navigate to `/` and load the HTML page.
2. Fetch (or receive via HAR) the WASM bundle.
3. **Compile the WASM module from bytes.** This is the dominant cost per
   test and cannot be cached by any Playwright API.
4. Initialize the Dioxus app, hydrate state from IndexedDB, and signal
   `[data-app-state="ready"]`.

Steps 3 and 4 are irreducible with the current architecture. The IndexedDB
restore saves only the `tasklensSeedSampleData()` call, which was already
~300ms -- negligible in a 44s suite.

### Why `routeFromHAR` didn't help

HAR replay serves responses from disk instead of the local `serve` process,
but:

- The local `serve` process is already serving from disk on localhost.
  The network hop is negligible (~1ms per request).
- HAR replay actually **disables the browser HTTP cache** (Playwright
  documents this: "Enabling routing disables http cache"), so we lose
  the in-memory cache that the browser would normally use.
- The WASM bundle (~3MB) must still be compiled after delivery regardless
  of transport.

### What actually dominates test time

Each test spends most of its time on:

1. **WASM compilation**: ~100--300ms per page load. The browser's compiled
   module cache is per-context and cannot be persisted by Playwright.
2. **App initialization**: The Dioxus framework boots, reads IndexedDB,
   and renders the initial DOM. This is CPU-bound Rust/WASM work.
3. **Test actions**: Clicking, typing, waiting for DOM updates. These are
   the actual test assertions and cannot be optimized away.

## Lessons Learned

1. **Profile before optimizing.** The sample data seeding (~300ms) was
   assumed to be expensive but was actually a tiny fraction of total test
   time. The real bottleneck (WASM compilation + app init) was not
   addressable by the proposed solution.

2. **`storageState` is for auth, not app state.** It works well for
   skipping login flows (cookies, tokens). For WASM apps where the
   expensive part is module compilation and framework initialization,
   restoring IndexedDB alone doesn't skip the slow path.

3. **`routeFromHAR` is not a cache -- it's a mock.** It replaces real
   network calls with recorded ones, but disables the browser's own
   HTTP cache in the process. For a localhost static file server, this
   is a lateral move at best.

4. **`launchPersistentContext` is the only API that preserves compiled
   WASM**, since it reuses a real browser profile directory with the
   actual disk cache. However, it is incompatible with Playwright's
   parallel test isolation model and would require a fundamentally
   different test architecture.

## What Would Actually Help

If E2E test speed becomes a real problem, these approaches would have
more impact:

- **Parallelism**: Increase `workers` from 1 (requires ensuring test
  isolation via separate IndexedDB databases per worker).
- **Smaller WASM bundle**: Reduce the compiled WASM size to decrease
  compilation time. Code splitting or lazy-loading modules could help.
- **Selective test runs**: Use Playwright's `--grep` or `--last-failed`
  to run only relevant tests during development.
- **`launchPersistentContext`**: Reuse a real browser profile with
  compiled WASM cache. Trades isolation for speed; only suitable for
  serial test execution.

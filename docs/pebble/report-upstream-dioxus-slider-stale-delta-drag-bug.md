+++
id = "issue-u9cGaU"
title = "Report upstream Dioxus slider stale-delta drag bug"
status = "todo"
created_at = 2026-02-23T17:42:00.017857+00:00
needs = []
tags = []
+++

# Bug Report: `dioxus_primitives::slider` can replay stale pointer delta, causing value drift and pointer desync

## Summary

The upstream `dioxus_primitives::slider::Slider` drag algorithm has a correctness bug: it integrates pointer **delta** inside a reactive effect, but does not consume that delta after application. When the component re-renders while dragging, the same delta can be applied again, so one physical movement is multiplied into several logical value updates.

This causes the exact behavior observed downstream:

- handle does not track pointer movement faithfully,
- value can accelerate toward min/max,
- sliders can appear "pinned" or snap unexpectedly.

This is not just a performance/event-storm problem; it is a semantic input-processing bug.

## Affected Upstream Revision

- Repo: `DioxusLabs/components`
- Crate: `dioxus-primitives`
- Revision used downstream: `8d65778356dfd5415cf4b1b7261f185551840261`

## Primary Defect (Correctness)

### Location

`primitives/src/slider.rs`

- Drag effect: lines ~220-251
- Delta source: `pointer.delta()` based on `position - last_position`
- Value update: `ctx.set_value.call(SliderValue::Single(stepped))`

### Problem

The drag effect computes:

1. `delta = pointer.position - pointer.last_position`
2. `new = granular_value + delta`
3. emits `set_value(new)`

But the effect is reactive and may run multiple times between pointermove events (for example due to controlled re-renders triggered by `on_value_change`). Because `pointer.last_position` and `pointer.position` are unchanged between those runs, the same delta is reapplied, causing drift.

In other words: pointer delta is treated like an event payload, but stored like persistent state and re-consumed on each effect pass.

### Why this produces incorrect UX

A small real pointer movement can become multiple step updates:

- expected: one move event -> one value step (or few)
- actual: one move event -> repeated integration of same delta -> rapid movement toward bounds

This explains sudden snapping/pinning and poor pointer-handle tracking.

## Secondary Issues (Amplifiers / Related)

### 1) Unconditional callback emission

`primitives/src/lib.rs` (`use_controlled`, lines ~97-111) always calls `on_change` from `set_value`, even when value did not change. This increases re-render pressure and makes stale-delta replay easier to trigger/faster.

### 2) Drag lifecycle not explicitly reset

In `slider.rs`, `dragging` is set `true` on pointer down (~326), but no explicit `dragging.set(false)` was found on pointer up/cancel in this file.

### 3) Coordinate-space inconsistency and duplicate pointer insertion

`slider.rs` mixes global `pageX/pageY` pointer tracking (~49-58) with local `client_coordinates` insertion on pointerdown (~291), and both paths insert pointer records for the same pointer id. This is another potential source of jumpiness in some layouts/scroll states.

## Downstream Evidence

From `/Users/matt/src/mydoo/context/console.log` during reproduction:

- long runs of repeated value processing while dragging,
- value marching toward bound and then repeated bound-value callbacks,
- downstream redistribution remained numerically stable (`sum=1.0`), indicating the upstream slider emitted problematic value sequences.

Observed symptoms matched stale-delta replay:

- over-reaction to small movements,
- snapping/pinning at extremes,
- handle/pointer desynchronization.

## Minimal Repro Shape

1. Create a controlled `Slider` (`value` prop bound to parent state).
2. In `on_value_change`, update parent state.
3. Drag slowly near a bound.
4. Observe value can advance more than pointer movement would justify, and may snap/pin.

## Suggested Upstream Fix (Preferred)

Use absolute pointer position to derive value during drag, not incremental delta integration.

Instead of:

- `new = granular_value + delta`

Use:

- `new = min + ((pointer_pos - track_start) / track_size) * range`
- clamp + step
- emit only if changed

Absolute mapping is idempotent under re-renders: repeated evaluations with same pointer position yield the same value.

## Additional Hardening

1. Emit `on_value_change` only when stepped value changes.
2. Explicitly clear drag state (`dragging=false`, clear active pointer id) on pointer up/cancel.
3. Use one coordinate space consistently (`client` or `page`, not mixed).
4. Avoid duplicate pointer records for the same pointer id.

## Downstream Workaround Used

Balance view in this repo was switched to native `input[type="range"]` to avoid this upstream drag-path behavior until upstream fix is available.

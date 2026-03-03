---
id: issue-8tmrh59m79f
title: "Fix plan view layout: expand button overlap and checkbox spacing"
status: done
created_at: 2026-03-03T03:31:53.945223556+00:00
modified_at: 2026-03-03T21:19:47.134712382+00:00
resolved_at: 2026-03-03T20:04:48.290578714+00:00
---

The layout in plan view is wrong. The button that expands/contracts the outline has a button area that extends out beyond the row for each item, and impinges upon the row's checkbox. There is not enough space around each item's completion checkbox.

Fix implemented:

1. Updated TaskRow layout to use `--app_touch_target` for chevron container/placeholder sizing and row min-height.
2. Added and applied `ButtonVariant::Icon` for the expand button, with an `expand_button` class for centering.
3. Increased checkbox interactive area using padding in `checkbox_custom`.
4. Wrapped rendered task rows in a dedicated `.task_list` container in `PlanPage`.

Verified with `just check-rust`.

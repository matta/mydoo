---
id: issue-o7q31dy0tfi
title: >-
  Use dioxus-primitives Badge component for due date display and urgency
  indicators in task UI
status: todo
priority: 20
created_at: 2026-03-02T14:59:35.949369190+00:00
tags:
  - task
  - deferred
---
## Notes

dioxus-primitives does NOT have a Badge component. Available components: accordion, alert_dialog, aspect_ratio, avatar, calendar, checkbox, collapsible, context_menu, date_picker, dialog, dropdown_menu, hover_card, label, menubar, navbar, popover, progress, radio_group, scroll_area, select, separator, slider, switch, tabs, toast, toggle, toggle_group, toolbar, tooltip. Options: (1) Create a custom Badge component, (2) Close this task as not applicable, (3) Wait for dioxus-primitives to add a Badge component.

## Comments

**Matt Armstrong** (2026-02-03T02:50:17Z):
Blocked: Badge component not yet published to dioxus-components registry. It exists only in the preview crate at https://github.com/DioxusLabs/components/tree/main/preview/src/components/badge. Deferred 30 days to re-check availability. Run 'dx components list' to check if badge appears.

## Related Issues

- mydoo-e48
- mydoo-63z

## Deferred

Deferred until: 2026-03-04T18:50:11-08:00

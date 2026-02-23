## 2024-05-24 - Priority Task Row Accessibility
**Learning:** `PriorityTaskRow` (and possibly other custom list items) often use `span` or `div` for interactivity without semantic roles, making them inaccessible.
**Action:** Always verify interactive elements have `role="button"`, `tabindex="0"`, `aria-label`, and `onkeydown` handlers for Enter/Space.

## 2024-05-24 - Dioxus 0.7 Key Handling
**Learning:** Dioxus 0.7 `Key` enum does not have a `Space` variant. Use `Key::Character(" ".to_string())` to detect Space bar presses.
**Action:** Use this pattern consistently and be aware it might look like a bug to those familiar with other frameworks or versions.

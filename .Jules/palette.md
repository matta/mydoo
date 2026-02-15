## 2024-05-22 - Missing ARIA Labels on Icon Buttons
**Learning:** Icon-only buttons (like "Add Subtask" and "Delete") are often overlooked for accessibility, relying solely on `title` attributes which are insufficient for screen readers.
**Action:** Always check `Button` components that wrap SVG icons for `aria-label` attributes, especially if they lack visible text.

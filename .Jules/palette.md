## 2024-05-22 - Accessibility of Custom Interactive Elements
**Learning:** Interactive elements like `span`s used as buttons often lack keyboard accessibility in this codebase.
**Action:** When identifying `onclick` handlers on non-interactive elements, always add `role="button"`, `tabindex="0"`, `aria_label`, and `onkeydown` support for Enter/Space.

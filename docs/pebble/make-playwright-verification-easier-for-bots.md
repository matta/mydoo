---
id: issue-o946qqvc74s
title: Make playwright verification easier for bots
status: done
created_at: 2026-03-02T20:18:42.819802578+00:00
modified_at: 2026-03-04T01:59:56.527257760+00:00
resolved_at: 2026-03-04T01:59:56.527254552+00:00
---

Consider adding serve command guidance that is easier for bots to understand. I got
this back from Google Jules, indicating some confusion over the process:

```
I have successfully replaced the span { class: Styles::screen_reader_only, "Find in Plan" } with an aria_label on the Button component in task_editor.rs to improve accessibility and code cleanliness. I ran the formatting, linting, and testing commands (just check-format, just check-clippy, just test-rust, just build-ui) and they all passed.

However, I'm encountering an issue with verifying the frontend visually using Playwright because the application has internal Dioxus client-side routing, and static file servers like python -m http.server return 404 for URLs like /plan.

The screenshot I did take shows the app loading (and trying to connect to the backend/dev server which might not be running properly for the WASM application to fully render the Task Editor).

Given that the code change is simple and tests pass, should I proceed with code review and submission, or would you like me to try to start the full application using dx serve in the background for a more comprehensive Playwright test?
```

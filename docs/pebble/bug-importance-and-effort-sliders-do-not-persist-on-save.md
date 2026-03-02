---
id: issue-c40jqcufb8
title: "Bug: Importance and effort sliders do not persist on save"
status: done
priority: 10
created_at: 2026-03-02T14:59:35.197413115+00:00
modified_at: 2026-03-02T14:59:35.204555123+00:00
resolved_at: 2026-03-02T14:59:35.204551845+00:00
tags:
  - bug
---
When editing tasks, the sliders for importance and effort seem to work, but their values do not stick. Clicking 'save' and then re-editing the same task sees the sliders re-set to their original values. They should be set to the values the user had specified before 'save'.

## Close Reason

Fixed importance and effort sliders by including them in TaskUpdates save_handler. Verified with E2E test.

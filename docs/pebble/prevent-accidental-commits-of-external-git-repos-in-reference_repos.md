---
id: issue-k3v8gb6x9s3
title: Prevent accidental commits of external git repos in reference_repos
status: done
priority: 10
created_at: 2026-03-02T14:59:35.702452312+00:00
modified_at: 2026-03-02T14:59:35.711690233+00:00
resolved_at: 2026-03-02T14:59:35.711686069+00:00
tags:
  - task
---
Commit 5dad53f accidentally committed a whole git repo under reference_repos. We should add a lint check to prevent this. Suggestion: a whitelist of approved subdirectories in reference_repos or a check for .git files that aren't expected.

---
*Imported from beads issue mydoo-mu1*

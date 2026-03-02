---
id: issue-684w8i65bn
title: Partition formatting tasks to be disjoint between Biome and Prettier
status: done
priority: 20
created_at: 2026-03-02T14:59:35.291496840+00:00
modified_at: 2026-03-02T14:59:35.298780661+00:00
resolved_at: 2026-03-02T14:59:35.298776976+00:00
tags:
  - task
---

The project currently has overlapping responsibilities between Biome and Prettier, which can lead to conflicting changes and wasted cycles. To ensure a clean and authoritative formatting pipeline, the tasks should be partitioned as follows:\n\n1. **Biome** should be the authoritative formatter for all languages it supports (JavaScript, TypeScript, JSON, JSONC, CSS, etc.).\n2. **Prettier** should be restricted to only the formats Biome does not yet support: YAML, HTML, and Markdown.\n\nThis will involve updating the recipes and any relevant configuration files to ensure neither tool attempts to format the other's territory.

## Close Reason

obsolete

---
id: issue-vsh012ck
title: Implement DocIndex Shallow Parse Type
status: done
priority: 20
created_at: 2026-03-02T14:59:34.577590413+00:00
modified_at: 2026-03-02T14:59:34.581220400+00:00
resolved_at: 2026-03-02T14:59:34.581215320+00:00
tags:
  - task
---

Create a new DocIndex type that provides shallow parsing of the Automerge doc root structure. This separates navigation (finding where things are) from content (deserializing actual data).

Key design:

- DocIndex contains HashMap<TaskID, automerge::ObjId> for tasks
- DocIndex contains HashMap<PlaceID, automerge::ObjId> for places
- DocIndex contains root_task_ids: Vec<TaskID>
- DocIndex contains other root-level metadata

Benefits:

- O(1) task/place lookup by ID returns ObjId pointer
- On-demand full hydration: PersistedTask::hydrate(doc, &obj_id)
- Structural integrity without deserializing content

TunnelState remains for legacy/UI rendering as a 'fully materialized view' computed when needed.

Depends on: mydoo-01z.1 (Manual TunnelState Hydration Module)

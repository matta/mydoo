import type { DocHandle } from "@automerge/automerge-repo";
import { z } from "zod";
import type { TaskID, TunnelState } from "../types/persistence";

// Define strict runtime schema for legacy tasks
// We specifically look for tasks that have the OLD 'Recurring' type.
const LegacyRecurringTaskSchema = z.object({
  id: z.string(),
  status: z.string(),
  lastCompletedAt: z.number().optional(),
  schedule: z.object({
    type: z.literal("Recurring"),
  }),
});

/**
 * Handles system-initiated changes (Schema Migrations).
 * @returns true if a mutation occurred, false otherwise.
 */
export function runReconciler(handle: DocHandle<TunnelState>): boolean {
  const doc = handle.doc();
  if (!doc) return false;

  // 1. Schema Migration: Recurring -> Routinely
  // Filter for legacy tasks first to avoid iterating the whole mutable doc later
  const legacyTasks = Object.values(doc.tasks).filter(
    (t) => LegacyRecurringTaskSchema.safeParse(t).success,
  );

  // 2. Schema Migration: Backfill metadata.automerge_url
  const needsMetadata = !doc.metadata?.automerge_url;

  // If we have any legacy tasks or missing metadata, we will mutate the document.
  const willMutate = legacyTasks.length > 0 || needsMetadata;

  if (willMutate) {
    handle.change((d) => {
      // 1. Fix Legacy Tasks
      for (const legacyTask of legacyTasks) {
        // Look up the mutable task by ID to avoid race conditions with Object.values(d.tasks)
        const task = d.tasks[legacyTask.id as TaskID];
        if (!task) continue;

        // Mutate the Automerge object.
        task.schedule.type = "Routinely";

        // Backfill lastCompletedAt
        if (legacyTask.status === "Done" && !legacyTask.lastCompletedAt) {
          task.lastCompletedAt = Date.now();
        }
      }

      // 2. Fix Metadata
      if (needsMetadata) {
        if (!d.metadata) d.metadata = {};
        if (!d.metadata.automerge_url) {
          d.metadata.automerge_url = handle.url;
        }
      }
    });
  }

  return willMutate;
}

import type {DocHandle} from '@automerge/automerge-repo';
import {z} from 'zod';
import type {TaskID, TunnelState} from '../types';

// Define strict runtime schema for legacy tasks
// We specifically look for tasks that have the OLD 'Recurring' type.
const LegacyRecurringTaskSchema = z.object({
  id: z.string(),
  status: z.string(),
  lastCompletedAt: z.number().optional(),
  schedule: z.object({
    type: z.literal('Recurring'),
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

  // If we have any legacy tasks, we will mutate the document. This is a subtle
  // invariant.
  const willMutate = legacyTasks.length > 0;

  if (willMutate) {
    handle.change((d) => {
      for (const legacyTask of legacyTasks) {
        // Look up the mutable task by ID to avoid race conditions with Object.values(d.tasks)
        const task = d.tasks[legacyTask.id as TaskID];
        if (!task) continue;

        // Validation gives us type safety at runtime
        // We re-validate briefly to ensure the mutable task is indeed the one we want
        // or just use the existence check since we filtered by ID from a recent snapshot.
        // For safety, let's just cast since we know the ID existed in the snapshot match.

        // Mutate the Automerge object.
        task.schedule.type = 'Routinely';

        // Backfill lastCompletedAt
        // We can use the legacyTask snapshot data for logic checks
        // Zod parsing guarantees these properties exist
        if (legacyTask.status === 'Done' && !legacyTask.lastCompletedAt) {
          task.lastCompletedAt = Date.now();
        }
      }
    });
  }

  return willMutate;
}

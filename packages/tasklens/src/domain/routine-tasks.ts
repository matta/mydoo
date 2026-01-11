import type { DocHandle } from "@automerge/automerge-repo";
import type { TunnelState } from "../types/persistence";
import { getIntervalMs } from "../utils/time";

/**
 * Wake up "Routinely" tasks that are due for their next cycle.
 *
 * This function is triggered manually by the user (e.g. via "Refresh").
 * It checks all "Done" + "Routinely" tasks to see if their wake-up window has arrived.
 * If so, it resets them to "Pending" and updates their due date to the next interval.
 *
 * @param handle - A DocHandle for any document. The function internally casts to
 *                 TunnelState to maintain type safety while allowing callers to
 *                 remain decoupled from the schema details.
 */
// biome-ignore lint/suspicious/noExplicitAny: Intentional type erasure for client decoupling
export function wakeUpRoutineTasks(handle: DocHandle<any>) {
  handle.change((d) => {
    const doc = d as TunnelState;
    const now = Date.now();

    for (const task of Object.values(doc.tasks)) {
      if (task.status === "Done" && task.schedule?.type === "Routinely") {
        const repeatConfig = task.repeatConfig;

        // Safety check: Routinely tasks must have a repeat config
        if (!repeatConfig) {
          continue;
        }

        const lastCompletedAt = task.lastCompletedAt ?? 0;

        // Calculate the next theoretical due date based on completion time + interval
        const intervalMs = getIntervalMs(
          repeatConfig.frequency,
          repeatConfig.interval,
        );
        const nextDueDate = lastCompletedAt + intervalMs;

        // Lead Time defines how early the task appears before it's due
        const wakeUpTime = nextDueDate - task.schedule.leadTime;

        if (now >= wakeUpTime) {
          // Wake up the task!
          task.status = "Pending";
          task.isAcknowledged = false;

          // Update the schedule for the new cycle
          task.schedule.dueDate = nextDueDate;
        }
      }
    }
  });
}

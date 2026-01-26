import type { DocHandle } from "@automerge/automerge-repo";
import {
  TaskStatus,
  type TunnelState,
  toTaskStatusScalar,
  unwrapScalar,
} from "../types/persistence";
import { getIntervalMs } from "../utils/time";

/**
 * Wake up "Routinely" tasks that are due for their next cycle.
 *
 * This function is triggered manually by the user (e.g. via "Refresh").
 * It checks all "Done" + "Routinely" tasks to see if their wake-up window has arrived.
 * If so, it resets them to "Pending" and updates their due date to the next interval.
 *
 * @param handle - A DocHandle for the TunnelState document.
 */
export function wakeUpRoutineTasks(handle: DocHandle<TunnelState>) {
  handle.change((doc) => {
    const now = Date.now();

    for (const task of Object.values(doc.tasks)) {
      if (
        unwrapScalar(task.status) === "Done" &&
        unwrapScalar(task.schedule?.type) === "Routinely"
      ) {
        const repeatConfig = task.repeatConfig;

        // Safety check: Routinely tasks must have a repeat config
        if (!repeatConfig) {
          continue;
        }

        const lastCompletedAt = task.lastCompletedAt ?? 0;

        // Calculate the next theoretical due date based on completion time + interval
        // We use a safe check for interval since WritableTask's index signature can pollute inference
        const interval =
          typeof repeatConfig.interval === "number" ? repeatConfig.interval : 1;

        const intervalMs = getIntervalMs(
          unwrapScalar(repeatConfig.frequency),
          interval,
        );
        const nextDueDate = lastCompletedAt + intervalMs;

        // Lead Time defines how early the task appears before it's due
        const wakeUpTime = nextDueDate - Number(task.schedule.leadTime);

        if (now >= wakeUpTime) {
          // Wake up the task!
          task.status = toTaskStatusScalar(TaskStatus.Pending);
          task.isAcknowledged = false;

          // Update the schedule for the new cycle
          task.schedule.lastDone = lastCompletedAt;
          // We can clear the explicit due date since priority.ts will calculate it from lastDone
          delete task.schedule.dueDate;
        }
      }
    }
  });
}

export type UrgencyStatus =
  | "Overdue"
  | "Urgent"
  | "Active"
  | "Upcoming"
  | "None";

export const UrgencyStatus = {
  Overdue: "Overdue",
  Urgent: "Urgent",
  Active: "Active",
  Upcoming: "Upcoming",
  None: "None",
} as const;

/**
 * Determines the urgency status of a task based on its effective due date and lead time.
 *
 * @param effectiveDueDate - The timestamp when the task is due.
 * @param effectiveLeadTime - The duration (ms) before due date when task becomes active.
 * @param currentTime - The current timestamp (ms).
 * @returns The urgency classification.
 */
export function getUrgencyStatus(
  effectiveDueDate: number | undefined,
  effectiveLeadTime: number | undefined,
  currentTime: number,
): UrgencyStatus {
  if (effectiveDueDate === undefined || effectiveLeadTime === undefined) {
    return UrgencyStatus.None;
  }

  // Check Overdue: Now > Due Date
  // Note: We use strictly greater than, so if it is exactly due date it might fall into Urgent (Today).
  // However, usually timestamps are specific.
  if (currentTime > effectiveDueDate) {
    // Spec says: "Overdue: The due date has passed."
    // But "Urgent" is "Due Today". Today overrides Overdue in many systems (or implies "Due by end of today").
    // If it's strictly past the timestamp (e.g. 00:00 UTC), it's technically passed.
    // However, if it's the SAME DAY in UTC, we count it as "Urgent" (Today), not "Overdue".

    if (isSameDayUTC(effectiveDueDate, currentTime)) {
      return UrgencyStatus.Urgent;
    }

    return UrgencyStatus.Overdue;
  }

  // Check Urgent: Final 25% of lead time window OR Due Today
  if (isSameDayUTC(effectiveDueDate, currentTime)) {
    return UrgencyStatus.Urgent;
  }

  const timeBuffer = effectiveDueDate - currentTime;
  const leadTime = effectiveLeadTime; // e.g. 7 days

  // Active Window: [DueDate - LeadTime, DueDate]
  // We are currently BEFORE DueDate (checked above).

  // If we are strictly outside the window (TimeBuffer > LeadTime)
  // Check "Upcoming": within 25% of lead time BEFORE the window starts.
  // Window Starts at: DueDate - LeadTime.
  // Upcoming Window: [DueDate - LeadTime - (0.25 * LeadTime), DueDate - LeadTime]

  if (timeBuffer > leadTime) {
    const upcomingThreshold = leadTime + leadTime * 0.25;
    if (timeBuffer <= upcomingThreshold) {
      return UrgencyStatus.Upcoming;
    }
    return UrgencyStatus.None;
  }

  // We are IN the active window (timeBuffer <= leadTime)

  // Urgent: "Final 25% of its lead time window".
  // The window duration is `leadTime`.
  // Final 25% means the timeBuffer is small (close to 0).
  // Buffer <= 25% of LeadTime.
  if (timeBuffer <= leadTime * 0.25) {
    return UrgencyStatus.Urgent;
  }

  // Otherwise, just Active
  return UrgencyStatus.Active;
}

/**
 * Checks if two timestamps represent the same day in UTC.
 *
 * @param t1 - The first timestamp in milliseconds since the Unix Epoch.
 * @param t2 - The second timestamp in milliseconds since the Unix Epoch.
 */
function isSameDayUTC(t1: number, t2: number): boolean {
  const d1 = new Date(t1);
  const d2 = new Date(t2);
  return (
    d1.getUTCFullYear() === d2.getUTCFullYear() &&
    d1.getUTCMonth() === d2.getUTCMonth() &&
    d1.getUTCDate() === d2.getUTCDate()
  );
}

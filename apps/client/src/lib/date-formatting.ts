/**
 * Formats a due date into a human-friendly string.
 *
 * Rules:
 * - Distance < 2 days: "Today", "Tomorrow", "Yesterday"
 * - Same year: "Oct 27"
 * - Different year: "Oct 27, 2025"
 *
 * @param dateTimestamp - The due date as a unix timestamp (ms).
 * @param nowTimestamp - (Optional) Current time, defaults to Date.now().
 * @returns Humanized date string.
 */
export function formatDueDate(
  dateTimestamp: number,
  nowTimestamp: number = Date.now(),
): string {
  // Use UTC dates for comparison
  const date = new Date(dateTimestamp);
  const now = new Date(nowTimestamp);

  /**
   * Intentionally asymmetric: compares the UTC-stored date against the user's LOCAL "today".
   * This ensures a due date stored as "2024-06-01 UTC" shows as "Today" when the user's
   * local date is June 1st, regardless of their timezone offset.
   */
  const isSameDayLocal = (utcDate: Date, nowDate: Date) => {
    return (
      utcDate.getUTCFullYear() === nowDate.getFullYear() &&
      utcDate.getUTCMonth() === nowDate.getMonth() &&
      utcDate.getUTCDate() === nowDate.getDate()
    );
  };

  // Check Today
  // Compare UTC Date (from DB) with Local Date (Now)
  if (isSameDayLocal(date, now)) {
    return "Today";
  }

  // Check Tomorrow
  // Construct a "Tomorrow" date relative to LOCAL time
  const tomorrowLocal = new Date(now);
  tomorrowLocal.setDate(now.getDate() + 1);

  if (isSameDayLocal(date, tomorrowLocal)) {
    return "Tomorrow";
  }

  // Check Yesterday
  const yesterdayLocal = new Date(now);
  yesterdayLocal.setDate(now.getDate() - 1);

  if (isSameDayLocal(date, yesterdayLocal)) {
    return "Yesterday";
  }

  // Format: "Oct 27" or "Oct 27, 2025"
  const months = [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec",
  ];
  const monthStr = months[date.getUTCMonth()];
  const dayStr = date.getUTCDate();

  if (date.getUTCFullYear() === now.getFullYear()) {
    return `${monthStr} ${dayStr}`;
  }

  return `${monthStr} ${dayStr}, ${date.getUTCFullYear()}`;
}

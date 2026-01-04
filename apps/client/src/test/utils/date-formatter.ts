/**
 * Formats a JavaScript Date object as an ISO date string (YYYY-MM-DD).
 *
 * @example
 * formatDateAsISO(new Date('2026-01-04')) // "2026-01-04"
 * formatDateAsISO(new Date('2026-01-04T12:30:00')) // "2026-01-04"
 */
export function formatDateAsISO(date: Date): string {
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, '0');
  const dd = String(date.getDate()).padStart(2, '0');
  return `${yyyy}-${mm}-${dd}`;
}

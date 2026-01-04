import {describe, expect, it} from 'vitest';
import {formatDateAsISO} from './date-formatter';

describe('formatDateAsISO', () => {
  it('should format a date at midnight', () => {
    const date = new Date(2026, 0, 4, 0, 0, 0); // Jan 4, 2026 in local time
    expect(formatDateAsISO(date)).toBe('2026-01-04');
  });

  it('should format a date with time component', () => {
    const date = new Date(2026, 0, 4, 12, 30, 45); // Jan 4, 2026 12:30:45 PM in local time
    expect(formatDateAsISO(date)).toBe('2026-01-04');
  });

  it('should handle single-digit months', () => {
    const date = new Date(2026, 2, 15, 0, 0, 0); // March 15, 2026 in local time
    expect(formatDateAsISO(date)).toBe('2026-03-15');
  });

  it('should handle single-digit days', () => {
    const date = new Date(2026, 11, 5, 0, 0, 0); // December 5, 2026 in local time
    expect(formatDateAsISO(date)).toBe('2026-12-05');
  });

  it('should handle year boundaries', () => {
    const date = new Date(2025, 11, 31, 23, 59, 59); // Dec 31, 2025 23:59:59 in local time
    expect(formatDateAsISO(date)).toBe('2025-12-31');
  });

  it('should handle leap year dates', () => {
    const date = new Date(2024, 1, 29, 0, 0, 0); // Feb 29, 2024 in local time
    expect(formatDateAsISO(date)).toBe('2024-02-29');
  });
});

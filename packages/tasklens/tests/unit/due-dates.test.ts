import { describe, expect, it } from 'vitest';
import { calculateLeadTimeFactor } from '../../src/domain/readiness';
import { createTask, updateTask } from '../../src/persistence/ops';
import { TunnelStore } from '../../src/persistence/store';
import { daysToMilliseconds } from '../../src/utils/time';

// A stable reference timestamp for testing (September 2001)
const REFERENCE_TIME = new Date('2001-09-09T01:46:40.000Z').getTime();

describe('Due Date Logic', () => {
  describe('Persistence (ops.ts)', () => {
    it('should persist schedule.dueDate when created', () => {
      const store = new TunnelStore();
      const dueDate = Date.now() + daysToMilliseconds(3);
      const leadTime = daysToMilliseconds(1);

      const task = createTask(store.state, {
        title: 'Due Date Task',
        schedule: {
          type: 'Once',
          dueDate,
          leadTime,
        },
      });

      const persisted = store.state.tasks[task.id];
      expect(persisted?.schedule.dueDate).toBe(dueDate);
      expect(persisted?.schedule.leadTime).toBe(leadTime);
    });

    it('should update schedule.dueDate', () => {
      const store = new TunnelStore();
      const task = createTask(store.state, { title: 'Update Test' });
      const newDueDate = Date.now() + daysToMilliseconds(5);

      updateTask(store.state, task.id, {
        schedule: {
          type: 'Once',
          leadTime: daysToMilliseconds(1),
          dueDate: newDueDate,
        },
      });

      expect(store.state.tasks[task.id]?.schedule.dueDate).toBe(newDueDate);
    });
  });

  describe('Readiness (Lead Time Factor)', () => {
    it('should be hidden (0.0) when > 2x lead time away', () => {
      const now = REFERENCE_TIME;
      const leadTime = daysToMilliseconds(1); // 1 day
      const dueDate = now + daysToMilliseconds(3); // 3 days away

      // Time remaining = 3 days. 2x lead time = 2 days.
      // 3 days > 2 days -> Should be 0.0

      const schedule = { type: 'Once' as const, leadTime, dueDate };
      const factor = calculateLeadTimeFactor(schedule, now);

      expect(factor).toBe(0.0);
    });

    it('should be ramping up when between 1x and 2x lead time away', () => {
      const now = REFERENCE_TIME;
      const leadTime = daysToMilliseconds(1); // 1 day
      const dueDate = now + daysToMilliseconds(1.5); // 1.5 days away

      // Time remaining = 1.5 days.
      // 2x lead time = 2 days.
      // Formula: (2*L - R) / L = (2 - 1.5) / 1 = 0.5

      const schedule = { type: 'Once' as const, leadTime, dueDate };
      const factor = calculateLeadTimeFactor(schedule, now);

      expect(factor).toBeCloseTo(0.5);
    });

    it('should be fully visible (1.0) when <= 1x lead time away', () => {
      const now = REFERENCE_TIME;
      const leadTime = daysToMilliseconds(1); // 1 day
      const dueDate = now + daysToMilliseconds(0.5); // 0.5 days away (12 hours)

      // Time remaining = 0.5 days.
      // 2*L - R = 2 - 0.5 = 1.5.
      // 1.5 / 1 = 1.5 -> Clamped to 1.0

      const schedule = { type: 'Once' as const, leadTime, dueDate };
      const factor = calculateLeadTimeFactor(schedule, now);

      expect(factor).toBe(1.0);
    });

    it('should be fully visible if passed due date', () => {
      const now = REFERENCE_TIME;
      const leadTime = daysToMilliseconds(1);
      const dueDate = now - daysToMilliseconds(1); // Overdue

      const schedule = { type: 'Once' as const, leadTime, dueDate };
      const factor = calculateLeadTimeFactor(schedule, now);

      expect(factor).toBe(1.0);
    });
  });
});

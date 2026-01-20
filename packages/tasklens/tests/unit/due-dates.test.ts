import { describe, expect, it } from "vitest";
import { getUrgencyStatus, UrgencyStatus } from "../../src/domain/dates";
import { calculateLeadTimeFactor } from "../../src/domain/readiness";
import { createTask, updateTask } from "../../src/persistence/ops";
import { TunnelStore } from "../../src/persistence/store";
import { daysToMilliseconds } from "../../src/utils/time";

// A stable reference timestamp for testing (September 2001)
const REFERENCE_TIME = new Date("2001-09-09T01:46:40.000Z").getTime();

describe("Due Date Logic", () => {
  describe("Persistence (ops.ts)", () => {
    it("should persist schedule.dueDate when created", () => {
      const store = new TunnelStore();
      const dueDate = Date.now() + daysToMilliseconds(3);
      const leadTime = daysToMilliseconds(1);

      const task = createTask(store.state, {
        title: "Due Date Task",
        schedule: {
          type: "Once",
          dueDate,
          leadTime,
        },
      });

      const persisted = store.state.tasks[task.id];
      expect(persisted?.schedule.dueDate).toBe(dueDate);
      expect(persisted?.schedule.leadTime).toBe(leadTime);
    });

    it("should update schedule.dueDate", () => {
      const store = new TunnelStore();
      const task = createTask(store.state, { title: "Update Test" });
      const newDueDate = Date.now() + daysToMilliseconds(5);

      updateTask(store.state, task.id, {
        schedule: {
          type: "Once",
          leadTime: daysToMilliseconds(1),
          dueDate: newDueDate,
        },
      });

      expect(store.state.tasks[task.id]?.schedule.dueDate).toBe(newDueDate);
    });
  });

  describe("Readiness (Lead Time Factor)", () => {
    it("should be hidden (0.0) when > 2x lead time away", () => {
      const now = REFERENCE_TIME;
      const leadTime = daysToMilliseconds(1); // 1 day
      const dueDate = now + daysToMilliseconds(3); // 3 days away

      // Time remaining = 3 days. 2x lead time = 2 days.
      // 3 days > 2 days -> Should be 0.0

      const factor = calculateLeadTimeFactor(dueDate, leadTime, now);

      expect(factor).toBe(0.0);
    });

    it("should be ramping up when between 1x and 2x lead time away", () => {
      const now = REFERENCE_TIME;
      const leadTime = daysToMilliseconds(1); // 1 day
      const dueDate = now + daysToMilliseconds(1.5); // 1.5 days away

      // Time remaining = 1.5 days.
      // 2x lead time = 2 days.
      // Formula: (2*L - R) / L = (2 - 1.5) / 1 = 0.5

      const factor = calculateLeadTimeFactor(dueDate, leadTime, now);

      expect(factor).toBeCloseTo(0.5);
    });

    it("should be fully visible (1.0) when <= 1x lead time away", () => {
      const now = REFERENCE_TIME;
      const leadTime = daysToMilliseconds(1); // 1 day
      const dueDate = now + daysToMilliseconds(0.5); // 0.5 days away (12 hours)

      // Time remaining = 0.5 days.
      // 2*L - R = 2 - 0.5 = 1.5.
      // 1.5 / 1 = 1.5 -> Clamped to 1.0

      const factor = calculateLeadTimeFactor(dueDate, leadTime, now);

      expect(factor).toBe(1.0);
    });

    it("should be fully visible if passed due date", () => {
      const now = REFERENCE_TIME;
      const leadTime = daysToMilliseconds(1);
      const dueDate = now - daysToMilliseconds(1); // Overdue

      const factor = calculateLeadTimeFactor(dueDate, leadTime, now);

      expect(factor).toBe(1.0);
    });
  });

  describe("Urgency Status (getUrgencyStatus)", () => {
    const now = REFERENCE_TIME; // 2001-09-09T01:46:40.000Z

    it.each([
      {
        scenario: "due date is in the past (Overdue)",
        daysDue: -1,
        leadTimeDays: 7,
        expected: UrgencyStatus.Overdue,
      },
      {
        scenario: "due date is today (Urgent)",
        daysDue: 0.5,
        leadTimeDays: 7,
        expected: UrgencyStatus.Urgent,
      },
      {
        scenario: "due date is strictly overdue but same UTC day (Urgent)",
        daysDue: -(46 / 60 / 24), // -46 minutes
        leadTimeDays: 7,
        expected: UrgencyStatus.Urgent,
      },
      {
        scenario: "due soon within lead time (Active)",
        daysDue: 3,
        leadTimeDays: 7,
        expected: UrgencyStatus.Active,
      },
      {
        scenario: "due shortly outside lead time within buffer (Upcoming)",
        daysDue: 8,
        leadTimeDays: 7,
        expected: UrgencyStatus.Upcoming,
      },
      {
        scenario: "due far in the future (None)",
        daysDue: 10,
        leadTimeDays: 7,
        expected: UrgencyStatus.None,
      },
      {
        // Urgent threshold: timeBuffer <= leadTime * 0.25
        // With leadTime=7 days, threshold is 1.75 days; 1 day remaining qualifies.
        scenario: "due in final 25% of lead time (Urgent)",
        daysDue: 1,
        leadTimeDays: 7,
        expected: UrgencyStatus.Urgent,
      },
    ])(
      "should return $expected when $scenario",
      ({ daysDue, leadTimeDays, expected }) => {
        const dueDate = now + daysToMilliseconds(daysDue);
        const leadTime = daysToMilliseconds(leadTimeDays);
        expect(getUrgencyStatus(dueDate, leadTime, now)).toBe(expected);
      },
    );

    it("should return None if dates are undefined", () => {
      expect(getUrgencyStatus(undefined, 1000, now)).toBe(UrgencyStatus.None);
      expect(getUrgencyStatus(1000, undefined, now)).toBe(UrgencyStatus.None);
    });
  });
});

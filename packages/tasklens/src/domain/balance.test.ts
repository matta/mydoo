import { describe, expect, it } from "vitest";
import { createMockTask } from "../test/test-utils";
import type { TaskID } from "../types/ui";
import { calculateBalanceData } from "./balance";

describe("calculateBalanceData", () => {
  it("should return empty array when no tasks are provided", () => {
    expect(calculateBalanceData([])).toEqual([]);
  });

  it("should exclude non-root tasks", () => {
    const tasks = [
      createMockTask({ id: "root-1" as TaskID, parentId: undefined }),
      createMockTask({ id: "child-1" as TaskID, parentId: "root-1" as TaskID }),
    ];
    const data = calculateBalanceData(tasks);
    expect(data).toHaveLength(1);
    const [root] = data;
    expect(root?.id).toBe("root-1");
  });

  it("should calculate percentages correctly", () => {
    const tasks = [
      createMockTask({
        id: "goal-1" as TaskID,
        desiredCredits: 1,
        effectiveCredits: 50,
      }),
      createMockTask({
        id: "goal-2" as TaskID,
        desiredCredits: 3,
        effectiveCredits: 50,
      }),
    ];
    const data = calculateBalanceData(tasks);

    // Total Desired: 4. Goal 1: 1/4 = 25%. Goal 2: 3/4 = 75%.
    // Total Actual: 100. Goal 1: 50/100 = 50%. Goal 2: 50/100 = 50%.

    const [goal1, goal2] = data;
    expect(goal1?.targetPercent).toBe(25);
    expect(goal1?.actualPercent).toBe(50);
    expect(goal2?.targetPercent).toBe(75);
    expect(goal2?.actualPercent).toBe(50);
  });

  it("should identify starving goals", () => {
    const tasks = [
      createMockTask({
        id: "goal-1" as TaskID,
        desiredCredits: 10,
        effectiveCredits: 10,
      }),
      createMockTask({
        id: "goal-2" as TaskID,
        desiredCredits: 10,
        effectiveCredits: 1,
      }),
    ];
    const data = calculateBalanceData(tasks);

    // Total Desired: 20. Target: 50% each.
    // Total Actual: 11.
    // Goal 1 Actual: 10/11 = ~90.9%. (90.9 > 50 * 0.9 = 45). Not starving.
    // Goal 2 Actual: 1/11 = ~9.1%. (9.1 < 50 * 0.9 = 45). Starving!

    const [goal1, goal2] = data;
    expect(goal1?.isStarving).toBe(false);
    expect(goal2?.isStarving).toBe(true);
  });

  it("should handle zero total credits gracefully", () => {
    const tasks = [createMockTask({ desiredCredits: 0, effectiveCredits: 0 })];
    const data = calculateBalanceData(tasks);
    const [goal] = data;
    expect(goal?.targetPercent).toBe(0);
    expect(goal?.actualPercent).toBe(0);
    expect(goal?.isStarving).toBe(false);
  });
});

import { describe, expect, it } from "vitest";
import { completeTask, createTask } from "../../src/persistence/ops";
import type { TaskID, TunnelState } from "../../src/types/persistence";
import { TaskStatus } from "../../src/types/persistence";
import {
  daysToMilliseconds,
  mockCurrentTimestamp,
  resetCurrentTimestampMock,
} from "../../src/utils/time";

function createBaseState(): TunnelState {
  return {
    tasks: {},
    rootTaskIds: [],
    places: {},
  };
}

describe("Credit Attribution & Decay", () => {
  const getTask = (state: TunnelState, id: TaskID) => {
    const task = state.tasks[id];
    if (!task) throw new Error(`Task ${id} not found`);
    return task;
  };

  it("should attribute credits to ancestors when a task is completed", () => {
    const state = createBaseState();

    // Setup: Goal G (root) -> Task T (child)
    const goal = createTask(state, {
      title: "Goal G",
      creditIncrement: 1.0,
    });
    const task = createTask(state, {
      title: "Task T",
      parentId: goal.id,
      creditIncrement: 0.5,
    });

    expect(getTask(state, goal.id).credits).toBe(0);
    expect(getTask(state, task.id).credits).toBe(0);

    // Action: Complete Task T
    completeTask(state, task.id);

    // Assert: T should have credits added, and G should inherit them
    expect(getTask(state, task.id).status).toBe(TaskStatus.Done);
    expect(getTask(state, task.id).credits).toBe(0.5);
    expect(getTask(state, goal.id).credits).toBe(0.5);
  });

  it("should decay credits based on time before adding new ones", () => {
    const state = createBaseState();
    const T0 = 1000000;
    mockCurrentTimestamp(T0);

    const goal = createTask(state, {
      title: "Goal G",
      credits: 100,
      creditsTimestamp: T0,
    });
    const task = createTask(state, {
      title: "Task T",
      parentId: goal.id,
      creditIncrement: 10,
    });

    // Move time forward by 7 days (one half-life)
    const T1 = T0 + daysToMilliseconds(7);
    mockCurrentTimestamp(T1);

    // Action: Complete Task T
    completeTask(state, task.id);

    // Assert:
    // Goal credits should be: (100 * 0.5) + 10 = 60
    // Task credits should be: 0 + 10 = 10
    expect(getTask(state, goal.id).credits).toBe(60);
    expect(getTask(state, task.id).credits).toBe(10);
    expect(getTask(state, goal.id).creditsTimestamp).toBe(T1);
    expect(getTask(state, task.id).creditsTimestamp).toBe(T1);

    resetCurrentTimestampMock();
  });
});

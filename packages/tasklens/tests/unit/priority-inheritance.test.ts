import { describe, expect, it } from "vitest";
import { recalculatePriorities } from "../../src/domain/priority";
import {
  ANYWHERE_PLACE_ID,
  DEFAULT_CREDIT_INCREMENT,
  type EnrichedTask,
  type PlaceID,
  type TaskID,
  TaskStatus,
  type TunnelState,
} from "../../src/types";

describe("Inheritance Logic", () => {
  it("should not overwrite child schedule if child has explicit due date", () => {
    const parentId = "parent" as TaskID;
    const childId = "child" as TaskID;

    const parent: EnrichedTask = {
      ...baseTask(parentId),
      childTaskIds: [childId],
      schedule: {
        type: "Once",
        dueDate: 1000, // Parent due at T=1000
        leadTime: 100,
      },
      isContainer: true,
    };

    const child: EnrichedTask = {
      ...baseTask(childId),
      parentId: parentId,
      schedule: {
        type: "Once",
        dueDate: 2000, // Child due at T=2000 (Later)
        leadTime: 0,
      },
      outlineIndex: 1,
    };

    const state = {
      rootTaskIds: [parentId],
      places: {},
      tasks: {
        [parentId]: parent,
        [childId]: child,
      },
    } as TunnelState;

    const enrichedTasks = [parent, child];

    // Test context: T=1500
    // Parent is overdue (1000). Child is future (2000).
    // If child is overwritten, it becomes overdue (1000) and shows up.
    // If child is preserved, it is hidden (Lead time 0, T=1500 > 2000 - 0? No. 2000 - 1500 = 500. 500 > 0. Hidden).

    // Wait, leadTimeFactor logic:
    // timeRemaining = dueDate - currentTime
    // Factor = 0 if timeRemaining > 2 * leadTime

    // Case 1: Child Overwritten to 1000
    // timeRemaining = 1000 - 1500 = -500.
    // -500 > 0? False.
    // Factor calculated as ... overdue tasks are ready.

    // Case 2: Child Preserved at 2000
    // timeRemaining = 2000 - 1500 = 500.
    // LeadTime = 0.
    // 500 > 0? True.
    // Factor = 0 (Hidden).

    recalculatePriorities(state, enrichedTasks, {}, { currentTime: 1500 });

    const childResult = enrichedTasks.find((t) => t.id === childId);

    // Expect child schedule to NOT be mutated to parent's schedule
    expect(childResult?.schedule.dueDate).toBe(2000);

    // Expect child to be effectively hidden (LeadTimeFactor 0)
    expect(childResult?.leadTimeFactor).toBe(0);
  });

  it("should inherit schedule recursively (Grandparent -> Parent -> Child)", () => {
    const gpId = "gp" as TaskID;
    const pId = "p" as TaskID;
    const cId = "c" as TaskID;

    // Grandparent has Due Date
    const gp: EnrichedTask = {
      ...baseTask(gpId),
      childTaskIds: [pId],
      schedule: { type: "Once", dueDate: 1000, leadTime: 100 },
      isContainer: true,
      outlineIndex: 0,
    };

    // Parent has undefined schedule -> Inherits from GP
    const p: EnrichedTask = {
      ...baseTask(pId),
      parentId: gpId,
      childTaskIds: [cId],
      schedule: { type: "Once", dueDate: undefined, leadTime: 0 },
      isContainer: true,
      outlineIndex: 1,
    };

    // Child has undefined schedule -> Inherits from P (who inherited from GP)
    const c: EnrichedTask = {
      ...baseTask(cId),
      parentId: pId,
      schedule: { type: "Once", dueDate: undefined, leadTime: 0 },
      outlineIndex: 2,
    };

    const state = {
      rootTaskIds: [gpId],
      places: {},
      tasks: { [gpId]: gp, [pId]: p, [cId]: c },
    } as TunnelState;

    const enrichedTasks = [gp, p, c];
    recalculatePriorities(state, enrichedTasks, {}, { currentTime: 0 });

    const cResult = enrichedTasks.find((t) => t.id === cId);

    // Child should have GP's due date
    expect(cResult?.schedule.dueDate).toBe(1000);
    expect(cResult?.schedule.leadTime).toBe(100);
  });
});

function baseTask(id: TaskID): EnrichedTask {
  return {
    id,
    title: "Task",
    status: TaskStatus.Pending,
    childTaskIds: [],
    schedule: { type: "Once", dueDate: undefined, leadTime: 0 },
    parentId: undefined,
    importance: 0.5,
    credits: 0,
    creditsTimestamp: 0,
    desiredCredits: 1,
    creditIncrement: DEFAULT_CREDIT_INCREMENT,
    isAcknowledged: false,
    isSequential: false,
    notes: "",
    placeId: ANYWHERE_PLACE_ID as PlaceID,
    priorityTimestamp: 0,
    effectiveCredits: 0,
    feedbackFactor: 1,
    leadTimeFactor: 0,
    normalizedImportance: 0,
    priority: 0,
    visibility: true,
    isContainer: false,
    isPending: true,
    isReady: true,
    outlineIndex: 0,
  };
}

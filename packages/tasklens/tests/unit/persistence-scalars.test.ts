import {
  change,
  ImmutableString,
  init,
  load,
  save,
} from "@automerge/automerge";
import { describe, expect, it } from "vitest";
import { TunnelStore } from "../../src/persistence/store";
import {
  type TaskID,
  TaskStatus,
  type TunnelState,
} from "../../src/types/persistence";

describe("Persistence - Scalars & Mixed String Types", () => {
  it("should correctly load mixed string formats and verify scalar persistence", () => {
    /**
     * STAGE 1: Create a "corrupted" document with mixed types
     * simulating legacy data and different Automerge versions.
     */
    let doc = init<{
      tasks: Record<string, unknown>;
      rootTaskIds: string[];
      places: Record<string, unknown>;
    }>();

    const taskId1 = "task-primitive" as TaskID;
    const taskId2 = "task-scalar" as TaskID;

    doc = change(doc, (d) => {
      d.rootTaskIds = [taskId1, taskId2];
      d.places = {};
      d.tasks = {
        [taskId1]: {
          id: taskId1,
          title: "Primitive String Task",
          status: "Pending", // LEGACY: Plain string
          importance: 1.0,
          creditIncrement: 0.5,
          credits: 0,
          desiredCredits: 1,
          creditsTimestamp: Date.now(),
          priorityTimestamp: Date.now(),
          isSequential: false,
          isAcknowledged: false,
          childTaskIds: [],
          schedule: {
            type: "Once", // LEGACY: Plain string
            leadTime: 0,
          },
        },
        [taskId2]: {
          id: taskId2,
          title: "Scalar Task",
          status: new ImmutableString("Done"), // MODERN: Scalar
          importance: 1.0,
          creditIncrement: 0.5,
          credits: 0,
          desiredCredits: 1,
          creditsTimestamp: Date.now(),
          priorityTimestamp: Date.now(),
          isSequential: false,
          childTaskIds: [],
          isAcknowledged: false,
          schedule: {
            type: new ImmutableString("Routinely"), // MODERN: Scalar
            leadTime: 0,
          },
        },
      };
    });

    /**
     * STAGE 2: Load into TunnelStore and verify normalization
     */
    const store = new TunnelStore(doc as TunnelState);

    const task1 = store.getTask(taskId1);
    const task2 = store.getTask(taskId2);

    expect(task1).toBeDefined();
    expect(task2).toBeDefined();

    if (!task1 || !task2) throw new Error("Tasks not found");

    // Verify values match regardless of underlying storage format
    expect(String(task1.status)).toBe("Pending");
    expect(String(task2.status)).toBe("Done");
    expect(String(task1.schedule.type)).toBe("Once");
    expect(String(task2.schedule.type)).toBe("Routinely");

    /**
     * STAGE 3: Update and verify SCALAR enforcement
     */
    // Move from primitive status "Pending" to scalar "Done" via updateTask
    store.updateTask(taskId1, { status: TaskStatus.Done });

    // Move from primitive schedule type "Once" to scalar "Once" (identity update to trigger migration)
    store.updateTask(taskId1, {
      schedule: {
        ...task1.schedule,
        type: "Once",
      },
    });

    const bytes = save(store.doc);
    const reloadedDoc = load<{
      tasks: Record<string, { status: unknown; schedule: { type: unknown } }>;
    }>(bytes);

    // Check raw underlying data for taskId1 (which started as primitive)
    const rawTask1 = reloadedDoc.tasks[taskId1];
    expect(rawTask1).toBeDefined();
    if (!rawTask1) throw new Error("rawTask1 not found");

    // Verify it is NOW an ImmutableString (scalar)
    expect(rawTask1.status).toBeInstanceOf(ImmutableString);
    expect(rawTask1.schedule.type).toBeInstanceOf(ImmutableString);

    expect((rawTask1.status as ImmutableString).val).toBe("Done");
    expect((rawTask1.schedule.type as ImmutableString).val).toBe("Once");

    // Verify taskId2 (which started as scalar) remains a scalar
    const rawTask2 = reloadedDoc.tasks[taskId2];
    expect(rawTask2).toBeDefined();
    if (!rawTask2) throw new Error("rawTask2 not found");
    expect(rawTask2.status).toBeInstanceOf(ImmutableString);
    expect((rawTask2.status as ImmutableString).val).toBe("Done");
  });

  it("should ensure EVERY new task uses scalar serialization automatically", () => {
    const store = new TunnelStore();
    const task = store.createTask({ title: "New Task Scalar Test" });

    // Check the raw document proxy directly
    const rawTask = store.doc.tasks[task.id];

    expect(rawTask).toBeDefined();
    if (!rawTask) throw new Error("Task not found in raw doc");

    expect(rawTask.status).toBeInstanceOf(ImmutableString);
    expect((rawTask.status as ImmutableString).val).toBe("Pending");
    expect(rawTask.schedule.type).toBeInstanceOf(ImmutableString);
    expect((rawTask.schedule.type as ImmutableString).val).toBe("Once");
  });
});

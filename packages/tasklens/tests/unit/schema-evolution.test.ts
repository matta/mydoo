import { describe, expect, it } from "vitest";
import { TunnelStore } from "../../src/persistence/store";
import type {
  PersistedTask,
  PlaceID,
  TaskID,
  TunnelState,
} from "../../src/types/persistence";

describe("Tier 1: Round-Trip Fidelity (Schema Evolution)", () => {
  it("should preserve unknown task fields during update", () => {
    // 1. Setup: Create initial state with "future" data
    const taskId = "task-future" as TaskID;
    const initialState: TunnelState = {
      rootTaskIds: [taskId],
      tasks: {
        // eslint-disable-next-line no-restricted-syntax
        [taskId]: {
          id: taskId,
          title: "Legacy Title",
          status: "Pending",
          childTaskIds: [],
          credits: 0,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          isSequential: false,
          isAcknowledged: false,
          importance: 0.5,
          schedule: {
            type: "Once",
            leadTime: 60,
          },
          // Simulating future field
          futureUnknownFieldTask: "important meta",
        } as unknown as PersistedTask,
      },
      places: {},
    };

    // 2. Action: Load into store and perform update
    const store = new TunnelStore(initialState);

    // Perform a surgical update on a KNOWN field
    store.updateTask(taskId, { title: "New Title" });

    // 3. Assertion: Verify unknown field persisted
    const task = store.getTask(taskId);
    expect(task).toBeDefined();
    expect(task?.title).toBe("New Title");
    // biome-ignore lint/suspicious/noExplicitAny: testing unknown fields
    expect((task as any).futureUnknownFieldTask).toBe("important meta");
  });

  it("should propagate unknown fields from updateTask props", () => {
    // This test covers the Object.assign(task, rest) path in ops.ts
    // 1. Setup: Create a task without unknown fields
    const taskId = "task-update-props" as TaskID;
    const initialState: TunnelState = {
      rootTaskIds: [taskId],
      tasks: {
        // eslint-disable-next-line no-restricted-syntax
        [taskId]: {
          id: taskId,
          title: "Original Title",
          status: "Pending",
          childTaskIds: [],
          credits: 0,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          isSequential: false,
          isAcknowledged: false,
          importance: 0.5,
          schedule: {
            type: "Once",
            leadTime: 60,
          },
        } as unknown as PersistedTask,
      },
      places: {},
    };

    const store = new TunnelStore(initialState);

    // 2. Action: Update with unknown field in props
    // eslint-disable-next-line no-restricted-syntax
    store.updateTask(taskId, {
      title: "Updated Title",
      futureUnknownFieldFromUpdate: "propagated value",
    } as unknown as Partial<PersistedTask>);

    // 3. Assertion: Verify unknown field was propagated
    const task = store.getTask(taskId);
    expect(task).toBeDefined();
    expect(task?.title).toBe("Updated Title");
    // biome-ignore lint/suspicious/noExplicitAny: testing unknown fields
    expect((task as any).futureUnknownFieldFromUpdate).toBe("propagated value");
  });

  it("should preserve unknown fields in nested objects (Schedule)", () => {
    // 1. Setup
    const taskId = "task-deep" as TaskID;
    const initialState: TunnelState = {
      rootTaskIds: [taskId],
      tasks: {
        // eslint-disable-next-line no-restricted-syntax
        [taskId]: {
          id: taskId,
          title: "Deep Field",
          status: "Pending",
          childTaskIds: [],
          credits: 0,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          isSequential: false,
          isAcknowledged: false,
          importance: 0.5,
          schedule: {
            type: "Once",
            leadTime: 60,
            // Simulating future nested field
            futureUnknownFieldSchedule: "mars",
          },
        } as unknown as PersistedTask,
      },
      places: {},
    };

    const store = new TunnelStore(initialState);

    // 2. Action: Update the nested object
    const task = store.getTask(taskId);
    if (!task) throw new Error("Task not found");

    store.updateTask(taskId, {
      schedule: {
        ...task.schedule,
        leadTime: 120, // Modifying known field
      },
    });

    // 3. Assertion
    const updatedTask = store.getTask(taskId);
    if (!updatedTask) throw new Error("Task not found");

    expect(updatedTask.schedule.leadTime).toBe(120);
    // biome-ignore lint/suspicious/noExplicitAny: testing unknown fields
    expect((updatedTask.schedule as any).futureUnknownFieldSchedule).toBe(
      "mars",
    );
  });

  it("should preserve unknown fields in Root State", () => {
    // 1. Setup: Explicitly create state with unknown root property
    // We treat TunnelState as a plain record first to inject the field
    const rawState = {
      rootTaskIds: [],
      tasks: {},
      places: {},
      futureUnknownFieldTunnelState: "metadata",
    };

    // 2. Action: Initialize store
    // eslint-disable-next-line no-restricted-syntax
    const store = new TunnelStore(rawState as unknown as TunnelState);

    // Perform some operation to trigger a change
    store.createTask({ title: "Trigger Change" });

    // 3. Assertion
    // biome-ignore lint/suspicious/noExplicitAny: testing unknown fields
    expect((store.state as any).futureUnknownFieldTunnelState).toBe("metadata");
  });

  it("should preserve unknown fields through Save/Load cycle", () => {
    // 1. Setup
    const taskId = "task-persist" as TaskID;
    const initialState: TunnelState = {
      rootTaskIds: [taskId],
      tasks: {
        // eslint-disable-next-line no-restricted-syntax
        [taskId]: {
          id: taskId,
          title: "Persist Me",
          status: "Pending",
          childTaskIds: [],
          credits: 0,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          isSequential: false,
          isAcknowledged: false,
          importance: 0.5,
          schedule: {
            type: "Once",
            leadTime: 60,
          },

          futureUnknownFieldTask: "saved data",
        } as unknown as PersistedTask,
      },
      places: {},
    };
    const store = new TunnelStore(initialState);

    // 2. Action: Save and Load
    const binary = store.save();
    const loadedStore = TunnelStore.load(binary);

    // 3. Assertion
    const loadedTask = loadedStore.getTask(taskId);
    // biome-ignore lint/suspicious/noExplicitAny: testing unknown fields
    expect((loadedTask as any).futureUnknownFieldTask).toBe("saved data");
  });

  it("should preserve unknown fields in Place", () => {
    // 1. Setup
    const placeId = "place-future" as PlaceID;
    const initialState: TunnelState = {
      rootTaskIds: [],
      tasks: {},
      places: {
        [placeId]: {
          id: placeId,
          hours: "",
          includedPlaces: [],
          // Simulating future unknown field
          futureUnknownFieldPlace: "lat,long",
        },
      },
    };

    // 2. Action: Load -> Save -> Load (Round trip)
    const store = new TunnelStore(initialState);
    const binary = store.save();
    const loadedStore = TunnelStore.load(binary);

    // 3. Assertion
    const loadedPlace = loadedStore.state.places[placeId];
    // biome-ignore lint/suspicious/noExplicitAny: testing unknown fields
    expect((loadedPlace as any).futureUnknownFieldPlace).toBe("lat,long");
  });

  it("should preserve unknown fields in RepeatConfig", () => {
    // 1. Setup
    const taskId = "task-repeat" as TaskID;
    const initialState: TunnelState = {
      rootTaskIds: [taskId],
      tasks: {
        // eslint-disable-next-line no-restricted-syntax
        [taskId]: {
          id: taskId,
          title: "Repeating Task",
          status: "Pending",
          childTaskIds: [],
          credits: 0,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          isSequential: false,
          isAcknowledged: false,
          importance: 0.5,
          schedule: {
            type: "Routinely",
            leadTime: 60,
          },
          repeatConfig: {
            frequency: "daily",
            interval: 1,

            futureUnknownFieldRepeatConfig: "daily-extra",
          },
        } as unknown as PersistedTask,
      },
      places: {},
    };

    // 2. Action: Load -> Save -> Load
    const store = new TunnelStore(initialState);
    const binary = store.save();
    const loadedStore = TunnelStore.load(binary);

    // 3. Assertion
    const loadedTask = loadedStore.getTask(taskId);
    if (!loadedTask || !loadedTask.repeatConfig)
      throw new Error("Task or RepeatConfig not found");

    // biome-ignore lint/suspicious/noExplicitAny: testing unknown fields
    const unknownField = (loadedTask.repeatConfig as any)
      .futureUnknownFieldRepeatConfig;
    expect(unknownField).toBe("daily-extra");
  });
});

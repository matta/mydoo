import { configureStore } from "@reduxjs/toolkit";
import { beforeEach, describe, expect, it } from "vitest";
import tasksReducer, {
  syncDoc,
  type TasksState,
} from "../../src/store/slices/tasks-slice";
import type {
  PersistedTask,
  TaskID as PersistenceTaskID,
  TunnelState,
} from "../../src/types/persistence";
import { type TaskID, TaskStatus } from "../../src/types/ui";

/**
 * Helper to create a SyncDocPayload for tests.
 * In tests, raw and parsed are the same (no Automerge Proxy distinction).
 */
function toPayload(doc: TunnelState) {
  return { proxyDoc: doc, parsedDoc: doc };
}

describe("tasksSlice - syncDoc", () => {
  let store: ReturnType<typeof configureStore<{ tasks: TasksState }>>;

  beforeEach(() => {
    store = configureStore({
      reducer: {
        tasks: tasksReducer,
      },
      middleware: (getDefaultMiddleware) =>
        getDefaultMiddleware({
          serializableCheck: false,
        }),
    });
  });

  const createMockTask = (id: string, title: string): PersistedTask => ({
    id: id as TaskID,
    title,
    status: TaskStatus.Pending,
    importance: 1.0,
    creditIncrement: 0.5,
    credits: 0.0,
    desiredCredits: 0.0,
    creditsTimestamp: 0,
    priorityTimestamp: 0,
    schedule: { type: "Once", leadTime: 0 },
    isSequential: false,
    childTaskIds: [] as PersistenceTaskID[],
    notes: "",
    isAcknowledged: false,
  });

  it("should initialize state and preserve stable references", async () => {
    const task1 = createMockTask("1", "Task 1");
    const task2 = createMockTask("2", "Task 2");

    const doc1: TunnelState = {
      tasks: {
        ["1" as TaskID]: task1,
        ["2" as TaskID]: task2,
      },
      rootTaskIds: ["1" as TaskID, "2" as TaskID],
      places: {},
      nextTaskId: 3,
      nextPlaceId: 1,
    };

    // First sync
    await store.dispatch(syncDoc(toPayload(doc1)));

    const state1 = store.getState().tasks;
    const computedTask1_v1 = state1.entities["1" as TaskID];
    const computedTask2_v1 = state1.entities["2" as TaskID];

    expect(computedTask1_v1).toBeDefined();
    expect(computedTask2_v1).toBeDefined();

    // Second sync with updated task 2, but task 1 remains SAME REFERENCE
    const updatedTask2 = { ...task2, title: "Updated Task 2" };
    const doc2: TunnelState = {
      ...doc1,
      tasks: {
        ["1" as TaskID]: task1,
        ["2" as TaskID]: updatedTask2,
      },
    };

    await store.dispatch(syncDoc(toPayload(doc2)));

    const state2 = store.getState().tasks;
    const computedTask1_v2 = state2.entities["1" as TaskID];
    const computedTask2_v2 = state2.entities["2" as TaskID];

    // ASSERTIONS
    // Task 1 should have preserved its reference because doc1.tasks['1'] === doc2.tasks['1']
    expect(computedTask1_v2).toBe(computedTask1_v1);

    // Task 2 should have a NEW reference because it changed in the doc
    expect(computedTask2_v2).not.toBe(computedTask2_v1);
    expect(computedTask2_v2?.title).toBe("Updated Task 2");
  });

  it("should handle adding and removing tasks", async () => {
    const task1 = createMockTask("1", "Task 1");
    const doc1: TunnelState = {
      tasks: { ["1" as TaskID]: task1 },
      rootTaskIds: ["1" as TaskID],
      places: {},
      nextTaskId: 2,
      nextPlaceId: 1,
    };

    await store.dispatch(syncDoc(toPayload(doc1)));
    expect(store.getState().tasks.todoListIds).toEqual(["1"]);

    const task2 = createMockTask("2", "Task 2");
    const doc2: TunnelState = {
      tasks: {
        ["1" as TaskID]: task1,
        ["2" as TaskID]: task2,
      },
      rootTaskIds: ["1" as TaskID, "2" as TaskID],
      places: {},
      nextTaskId: 3,
      nextPlaceId: 1,
    };

    await store.dispatch(syncDoc(toPayload(doc2)));
    expect(store.getState().tasks.todoListIds).toContain("1");
    expect(store.getState().tasks.todoListIds).toContain("2");
    expect(Object.keys(store.getState().tasks.entities)).toHaveLength(2);

    const doc3: TunnelState = {
      tasks: { ["2" as TaskID]: task2 },
      rootTaskIds: ["2" as TaskID],
      places: {},
      nextTaskId: 3,
      nextPlaceId: 1,
    };

    await store.dispatch(syncDoc(toPayload(doc3)));
    expect(store.getState().tasks.todoListIds).toEqual(["2"]);
    expect(store.getState().tasks.entities["1" as TaskID]).toBeUndefined();
  });

  it("should handle first sync when lastProxyDoc is null", async () => {
    // Verify initial state
    expect(store.getState().tasks.lastProxyDoc).toBeNull();
    expect(store.getState().tasks.todoListIds).toEqual([]);

    const task1 = createMockTask("1", "Task 1");
    const doc1: TunnelState = {
      tasks: { ["1" as TaskID]: task1 },
      rootTaskIds: ["1" as TaskID],
      places: {},
      nextTaskId: 2,
      nextPlaceId: 1,
    };

    await store.dispatch(syncDoc(toPayload(doc1)));

    // After first sync, state should be populated
    expect(store.getState().tasks.lastProxyDoc).not.toBeNull();
    expect(store.getState().tasks.todoListIds).toEqual(["1"]);
    expect(store.getState().tasks.entities["1" as TaskID]).toBeDefined();
  });

  it("should handle empty document state", async () => {
    const emptyDoc: TunnelState = {
      tasks: {},
      rootTaskIds: [],
      places: {},
      nextTaskId: 1,
      nextPlaceId: 1,
    };

    await store.dispatch(syncDoc(toPayload(emptyDoc)));

    expect(store.getState().tasks.todoListIds).toEqual([]);
    expect(Object.keys(store.getState().tasks.entities)).toHaveLength(0);
    expect(store.getState().tasks.lastProxyDoc).not.toBeNull();
  });
});

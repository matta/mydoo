import type { AutomergeUrl, DocHandle } from "@automerge/automerge-repo";
import { describe, expect, it, vi } from "vitest";
import { runReconciler } from "../../src/domain/reconciler";
import { strictMock } from "../../src/test/test-utils";
import type {
  PersistedTask,
  TaskID,
  TunnelState,
} from "../../src/types/persistence";

// Helper to create a minimal TunnelState with the given tasks
function createTestState(
  tasks: Record<string, Partial<PersistedTask>>,
): TunnelState {
  const typedTasks: Record<TaskID, PersistedTask> = {};
  for (const [id, task] of Object.entries(tasks)) {
    typedTasks[id as TaskID] = {
      id: id as TaskID,
      title: "",
      status: "Pending",
      importance: 1.0,
      creditIncrement: 0.5,
      credits: 0.0,
      desiredCredits: 0.0,
      creditsTimestamp: 0,
      priorityTimestamp: 0,
      schedule: { type: "Once", leadTime: 0 },
      isSequential: false,
      childTaskIds: [],
      notes: "",
      isAcknowledged: false,
      ...task,
    } as PersistedTask;
  }
  return {
    tasks: typedTasks,
    places: {},
    rootTaskIds: Object.keys(typedTasks) as TaskID[],
    metadata: { automerge_url: "automerge:123" }, // Default to migrated state
  };
}

// Helper to create a mock DocHandle for testing
function createMockDocHandle(state: TunnelState): DocHandle<TunnelState> {
  return strictMock<DocHandle<TunnelState>>("DocHandle", {
    doc: () => state,
    url: "automerge:123" as AutomergeUrl, // Mock URL
    change: (callback: (doc: TunnelState) => void) => {
      callback(state);
    },
  });
}

describe("runReconciler", () => {
  it("should migrate Recurring tasks to Routinely", () => {
    const state = createTestState({
      "task-1": {
        title: "Old Task",
        status: "Pending",
        schedule: {
          type: "Recurring" as "Once" | "Routinely", // Old type (cast to satisfy TS)
          leadTime: 1000,
        },
      },
      "task-2": {
        title: "Normal Task",
        status: "Pending",
        schedule: {
          type: "Once",
          leadTime: 0,
        },
      },
    });

    const handle = createMockDocHandle(state);
    const result = runReconciler(handle);

    expect(result).toBe(true); // Should return true on mutation

    const updatedTask = state.tasks["task-1" as TaskID];
    expect(updatedTask).toBeDefined();
    expect(updatedTask?.schedule.type).toBe("Routinely");

    const normalTask = state.tasks["task-2" as TaskID];
    expect(normalTask).toBeDefined();
    expect(normalTask?.schedule.type).toBe("Once");
  });

  it("should backfill lastCompletedAt for Done Routinely tasks", () => {
    const state = createTestState({
      "task-done": {
        title: "Done Old Task",
        status: "Done",
        // Missing lastCompletedAt
        schedule: {
          type: "Recurring" as "Once" | "Routinely",
          leadTime: 1000,
        },
      },
    });

    const handle = createMockDocHandle(state);
    const result = runReconciler(handle);

    expect(result).toBe(true); // Should return true on mutation

    const task = state.tasks["task-done" as TaskID];
    expect(task).toBeDefined();
    expect(task?.schedule.type).toBe("Routinely");
    expect(task?.lastCompletedAt).toBeDefined();
    // Should be close to now
    expect(task?.lastCompletedAt).toBeGreaterThan(Date.now() - 1000);
  });

  it("should backfill metadata.automerge_url if missing", () => {
    const state = createTestState({});
    // Explicitly remove metadata to simulate legacy doc
    state.metadata = undefined;

    const handle = createMockDocHandle(state);
    const result = runReconciler(handle);

    expect(result).toBe(true);
    expect(state.metadata).toEqual({ automerge_url: "automerge:123" });
  });

  it("should be idempotent (not change anything if already migrated)", () => {
    const state = createTestState({
      "task-1": {
        title: "New Task",
        status: "Pending",
        schedule: {
          type: "Routinely",
          leadTime: 1000,
        },
      },
    });

    const handle = createMockDocHandle(state);
    // Spy on change to ensure it's not called
    const changeSpy = vi.spyOn(handle, "change");

    const result = runReconciler(handle);

    expect(result).toBe(false); // Should return false when no mutation
    expect(changeSpy).not.toHaveBeenCalled();
  });
});

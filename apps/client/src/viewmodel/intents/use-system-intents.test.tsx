import {
  type AutomergeUrl,
  type DocHandle,
  Repo,
} from "@automerge/automerge-repo";
import {
  createMockTask as createSharedMockTask,
  createTaskLensStore,
  type PersistedTask,
  type TaskID,
  TaskStatus,
  type TunnelState,
} from "@mydoo/tasklens";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { createTestWrapper } from "../../test/setup";
import { useSystemIntents } from "./use-system-intents";

const createMockTask = (
  id: string,
  title: string,
  status: TaskStatus,
  isAcknowledged: boolean,
): PersistedTask => {
  return createSharedMockTask({
    id: id as TaskID,
    title,
    status,
    isAcknowledged,
    isPending: status === TaskStatus.Pending,
  });
};

describe("useSystemIntents", () => {
  let repo: Repo;
  let handle: DocHandle<TunnelState>;
  let docUrl: AutomergeUrl;

  beforeEach(() => {
    repo = new Repo({ network: [] });
    window.location.hash = "";
    handle = repo.create({ tasks: {}, rootTaskIds: [], places: {} });
    docUrl = handle.url;
  });

  afterEach(() => {
    window.location.hash = "";
  });

  describe("refreshTaskList", () => {
    it("should acknowledge completed tasks", async () => {
      // 1. Seed Data
      handle.change((d) => {
        d.tasks["task1" as TaskID] = createMockTask(
          "task1",
          "Pending",
          TaskStatus.Pending,
          false,
        );
        d.tasks["task2" as TaskID] = createMockTask(
          "task2",
          "Done Unacked",
          TaskStatus.Done,
          false,
        );
        d.tasks["task3" as TaskID] = createMockTask(
          "task3",
          "Done Acked",
          TaskStatus.Done,
          true,
        );
        d.rootTaskIds = [
          "task1" as TaskID,
          "task2" as TaskID,
          "task3" as TaskID,
        ];
      });

      // 2. Setup Hook
      const store = createTaskLensStore();
      const wrapper = createTestWrapper(repo, store, docUrl);
      const { result } = renderHook(() => useSystemIntents(), { wrapper });

      // Wait for Redux to have the tasks (to avoid race conditions in intents)
      await waitFor(() => {
        const state = store.getState();
        if (!state.tasks.entities["task1" as TaskID])
          throw new Error("Task1 not in store");
      });

      // 3. Act
      act(() => {
        result.current.refreshTaskList();
      });

      // 4. Verify in Doc
      await waitFor(() => {
        const doc = handle.doc();
        const t1 = doc.tasks["task1" as TaskID];
        const t2 = doc.tasks["task2" as TaskID];
        const t3 = doc.tasks["task3" as TaskID];

        if (!t1 || !t2 || !t3) throw new Error("Tasks missing in final doc");
        expect(t1.isAcknowledged).toBe(false);
        expect(t2.isAcknowledged).toBe(true); // Changed!
        expect(t3.isAcknowledged).toBe(true);
      });
    });

    it("should wake up routine tasks", async () => {
      // 1. Seed Data with a routine task ready to wake up
      handle.change((d) => {
        const routineTaskId = "routine-task" as TaskID;
        d.tasks[routineTaskId] = createSharedMockTask({
          id: routineTaskId,
          title: "Morning Routine",
          status: TaskStatus.Done,
          isAcknowledged: true,
          schedule: {
            type: "Routinely",
            leadTime: 3600000,
            dueDate: 1000,
          },
          repeatConfig: {
            frequency: "daily",
            interval: 1,
          },
          lastCompletedAt: Date.now() - 1000 * 60 * 60 * 25, // 25 hours ago
        });
        d.rootTaskIds = [routineTaskId];
      });

      // 2. Setup Hook
      const store = createTaskLensStore();
      const wrapper = createTestWrapper(repo, store, docUrl);
      const { result } = renderHook(() => useSystemIntents(), { wrapper });

      // Wait for Redux
      await waitFor(() => {
        const state = store.getState();
        if (!state.tasks.entities["routine-task" as TaskID])
          throw new Error("Task not in store");
      });

      // 3. Act
      act(() => {
        result.current.refreshTaskList();
      });

      // 4. Verify in Doc
      await waitFor(() => {
        const doc = handle.doc();
        const t = doc.tasks["routine-task" as TaskID];
        if (!t) throw new Error("Task missing");

        // Should be woken up (Pending and Unacknowledged)
        expect(t.status).toBe(TaskStatus.Pending);
        expect(t.isAcknowledged).toBe(false);
      });
    });
  });
});

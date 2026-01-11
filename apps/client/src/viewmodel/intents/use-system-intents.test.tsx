import {
  type AutomergeUrl,
  type DocHandle,
  Repo,
} from "@automerge/automerge-repo";

import { type TaskID, TaskStatus } from "@mydoo/tasklens";
import type { TunnelState } from "@mydoo/tasklens/persistence";
import {
  createEmptyTunnelState,
  mockCurrentTimestamp,
  resetCurrentTimestampMock,
} from "@mydoo/tasklens/test";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createClientStore } from "../../store";
import { createTestWrapper } from "../../test/setup";
import { useSystemIntents } from "./use-system-intents";
import { useTaskIntents } from "./use-task-intents";

describe("useSystemIntents", () => {
  let repo: Repo;
  let handle: DocHandle<TunnelState>;
  let docUrl: AutomergeUrl;

  beforeEach(() => {
    repo = new Repo({ network: [] });
    window.location.hash = "";
    handle = repo.create(createEmptyTunnelState());
    docUrl = handle.url;
    vi.useRealTimers();
  });

  afterEach(() => {
    window.location.hash = "";
    vi.useRealTimers();
  });

  describe("refreshTaskList", () => {
    it("should acknowledge completed tasks", async () => {
      // 1. Setup Wrapper
      const store = createClientStore(docUrl, repo);
      const wrapper = createTestWrapper(repo, docUrl, store);

      // 2. Seed Data using public API via useTaskIntents
      const { result: intents } = renderHook(() => useTaskIntents(), {
        wrapper,
      });

      // Wait for initial Redux sync
      await waitFor(() => {
        expect(store.getState().tasks.lastProxyDoc).toBeDefined();
      });

      let t1: TaskID;
      let t2: TaskID;
      let t3: TaskID;

      await act(async () => {
        // Task 1: Pending (should remain unacked)
        t1 = intents.current.createTask({ title: "task1" });

        // Task 2: Done (default unacked) -> Should be auto-acked
        t2 = intents.current.createTask({ title: "task2" });
        intents.current.updateTask(t2, { status: TaskStatus.Done });

        // Task 3: Done + Acked (should remain acked)
        t3 = intents.current.createTask({ title: "task3" });
        intents.current.updateTask(t3, {
          status: TaskStatus.Done,
          isAcknowledged: true,
        });
      });

      // Wait for Redux to have the tasks
      await waitFor(() => {
        const state = store.getState();
        if (!state.tasks.entities[t1]) throw new Error("Task1 not in store");
      });

      // 3. Act - Refresh
      const { result: systemIntents } = renderHook(
        () => useSystemIntents(docUrl),
        {
          wrapper,
        },
      );

      await act(async () => {
        systemIntents.current.refreshTaskList();
      });

      // 4. Verify in Store
      await waitFor(() => {
        const doc = store.getState().tasks.entities;
        const task1 = doc[t1];
        const task2 = doc[t2];
        const task3 = doc[t3];

        if (!task1 || !task2 || !task3)
          throw new Error("Tasks missing in store");

        expect(task1.isAcknowledged).toBe(false);
        expect(task2.isAcknowledged).toBe(true); // Changed!
        expect(task3.isAcknowledged).toBe(true);
      });
    });

    it("should wake up routine tasks", async () => {
      // 1. Setup Wrapper and Time
      const store = createClientStore(docUrl, repo);
      const wrapper = createTestWrapper(repo, docUrl, store);

      // Use internal mocking helper instead of vi.useFakeTimers
      // This ensures code using getCurrentTimestamp() sees the mock,
      // while preserving native async behavior (setTimeout, Promise, etc.)
      const realNow = Date.now();
      const yesterday = realNow - 1000 * 60 * 60 * 25;

      try {
        mockCurrentTimestamp(yesterday);

        // 2. Seed Data using public API
        const { result: intents } = renderHook(() => useTaskIntents(), {
          wrapper,
        });

        // Wait for initial Redux sync
        await waitFor(() => {
          expect(store.getState().tasks.lastProxyDoc).toBeDefined();
        });

        let routineTaskId: TaskID;

        await act(async () => {
          routineTaskId = intents.current.createTask({
            title: "Morning Routine",
            schedule: {
              type: "Routinely",
              leadTime: 3600000,
            },
            repeatConfig: {
              frequency: "daily",
              interval: 1,
            },
          });

          // Complete it "yesterday"
          intents.current.updateTask(routineTaskId, {
            status: TaskStatus.Done,
            isAcknowledged: true,
          });
        });

        // Advance time to "now"
        mockCurrentTimestamp(realNow);

        // Wait for Redux to have the task
        await waitFor(() => {
          const state = store.getState();
          if (!state.tasks.entities[routineTaskId])
            throw new Error("Task not in store");
        });

        // 3. Act - Refresh
        const { result: systemIntents } = renderHook(
          () => useSystemIntents(docUrl),
          {
            wrapper,
          },
        );

        // FIXME: Replace with robust wait strategy instead of hardcoded delay
        await act(async () => {
          await new Promise((r) => setTimeout(r, 100));
          systemIntents.current.refreshTaskList();
        });

        // 4. Verify in Store
        await waitFor(() => {
          const task = store.getState().tasks.entities[routineTaskId];
          if (!task) throw new Error("Task missing");

          // Should be woken up (Pending and Unacknowledged)
          expect(task.status).toBe(TaskStatus.Pending);
          expect(task.isAcknowledged).toBe(false);
        });
      } finally {
        resetCurrentTimestampMock();
      }
    });
  });
});

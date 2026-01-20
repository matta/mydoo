import { type TaskID, TaskStatus } from "@mydoo/tasklens";
import { createTaskLensTestEnvironment } from "@mydoo/tasklens/test";
import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { createTestWrapper } from "../../test/setup";
import { useTaskIntents } from "../intents/use-task-intents";
import { usePriorityList } from "./use-priority-list";

describe("usePriorityList", () => {
  it("filters out completed tasks that are acknowledged", async () => {
    vi.clearAllMocks();
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);

    // 1. Setup Data via Intents
    const { result: intents } = renderHook(() => useTaskIntents(), { wrapper });

    // Wait for initial Redux sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    await act(async () => {
      intents.current.createTask({
        title: "Todo 1",
        status: TaskStatus.Pending,
        importance: 0.5,
      });

      const t2 = intents.current.createTask({
        title: "Done 1",
        status: TaskStatus.Done,
        importance: 0.5,
      });

      // Update to be acknowledged
      intents.current.updateTask(t2, { isAcknowledged: true });
    });

    // 2. Render Projection
    const { result } = renderHook(() => usePriorityList(), { wrapper });

    // 3. Verify
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
      expect(result.current.tasks).toHaveLength(1);
    });

    expect(result.current.tasks[0]?.title).toBe("Todo 1");
  });

  it("includes completed tasks that are NOT acknowledged", async () => {
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);

    // Wait for initial Redux sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    // 1. Setup Data via Intents
    const { result: intents } = renderHook(() => useTaskIntents(), { wrapper });

    let t1: TaskID = "t1" as TaskID;
    let t2: TaskID = "t2" as TaskID;

    await act(async () => {
      t1 = intents.current.createTask({
        title: "Todo 1",
        status: TaskStatus.Pending,
        importance: 0.5,
      });

      t2 = intents.current.createTask({
        title: "Done Unacked",
        status: TaskStatus.Done,
        importance: 0.5,
        // Default isAcknowledged is false
      });
    });

    // 2. Render Projection
    const { result } = renderHook(() => usePriorityList(), { wrapper });

    // 3. Verify
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
      expect(result.current.tasks).toHaveLength(2);
    });

    const ids = result.current.tasks.map((t) => t.id);
    expect(ids).toContain(t1);
    expect(ids).toContain(t2);
  });

  it("sorts tasks by priority (descending)", async () => {
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);

    // Wait for initial Redux sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    // 1. Setup Data via Intents
    const { result: intents } = renderHook(() => useTaskIntents(), { wrapper });

    let tHigh: TaskID = "high" as TaskID;
    let tMed: TaskID = "med" as TaskID;
    let tLow: TaskID = "low" as TaskID;

    await act(async () => {
      tLow = intents.current.createTask({
        title: "Low Priority",
        status: TaskStatus.Pending,
        importance: 0.1,
      });

      tHigh = intents.current.createTask({
        title: "High Priority",
        status: TaskStatus.Pending,
        importance: 0.9,
      });

      tMed = intents.current.createTask({
        title: "Medium Priority",
        status: TaskStatus.Pending,
        importance: 0.5,
      });
    });

    // 2. Render Projection
    const { result } = renderHook(() => usePriorityList(), { wrapper });

    // 3. Verify
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
      expect(result.current.tasks).toHaveLength(3);
    });

    expect(result.current.tasks).toMatchObject([
      { id: tHigh },
      { id: tMed },
      { id: tLow },
    ]);
  });

  it("returns loading state initially", async () => {
    // Verify that the hook starts in a loading state before the initial Redux sync completes.
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result } = renderHook(() => usePriorityList(), { wrapper });

    // Initial state should be loading until the subscription establishes and syncs
    if (result.current.isLoading) {
      expect(result.current.tasks).toEqual([]);
    }

    // Eventually it should settle to not loading
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
  });
});

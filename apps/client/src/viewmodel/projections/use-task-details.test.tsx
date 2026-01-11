import type { TaskID } from "@mydoo/tasklens";
import { createTaskLensTestEnvironment } from "@mydoo/tasklens/test";

import { act, renderHook, waitFor } from "@testing-library/react";

import { describe, expect, it, vi } from "vitest";

import { createTestWrapper } from "../../test/setup";
import { useTaskIntents } from "../intents/use-task-intents";
import { useTaskDetails } from "./use-task-details";

describe("useTaskDetails", () => {
  it("returns task details correctly", async () => {
    vi.clearAllMocks();
    const { repo, docUrl: url, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, url, store);
    const { result: intents } = renderHook(() => useTaskIntents(), { wrapper });

    // Wait for initial Redux sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    let parentId: TaskID = "p" as TaskID;
    let childId: TaskID = "c" as TaskID;

    await act(async () => {
      parentId = intents.current.createTask({ title: "Parent Goal" });
      childId = intents.current.createTask({ title: "Child Task", parentId });
      intents.current.createTask({ title: "Grandchild", parentId: childId });
    });

    const { result } = renderHook(() => useTaskDetails(childId), {
      wrapper,
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
        expect(result.current.task?.title).toBe("Child Task");
        expect(result.current.parentTitle).toBe("Parent Goal");
        expect(result.current.descendantCount).toBe(1); // One grandchild
      },
      { timeout: 2000 },
    );
  });

  it("handles root tasks (no parent)", async () => {
    const { repo, docUrl: url, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, url, store);
    const { result: intents } = renderHook(() => useTaskIntents(), { wrapper });

    // Wait for initial Redux sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    let rootId: TaskID = "r" as TaskID;

    await act(async () => {
      rootId = intents.current.createTask({ title: "Root Task" });
    });

    const { result } = renderHook(() => useTaskDetails(rootId), {
      wrapper,
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
        expect(result.current.task?.title).toBe("Root Task");
        expect(result.current.parentTitle).toBeUndefined();
        expect(result.current.descendantCount).toBe(0);
      },
      { timeout: 2000 },
    );
  });

  it("returns null when task not found", async () => {
    const { repo, docUrl: url, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, url, store);
    const { result } = renderHook(
      () => useTaskDetails("non-existent" as TaskID),
      {
        wrapper,
      },
    );

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
        expect(result.current.task).toBeUndefined();
      },
      { timeout: 2000 },
    );
  });

  it("returns loading state initially", async () => {
    const { repo, docUrl: url, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, url, store);
    const { result } = renderHook(() => useTaskDetails("any-task" as TaskID), {
      wrapper,
    });

    // Initial state should be loading
    expect(result.current.isLoading).toBe(true);
    expect(result.current.task).toBeUndefined();

    // Wait for Redux sync to complete to avoid act() warning
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });
  });
});

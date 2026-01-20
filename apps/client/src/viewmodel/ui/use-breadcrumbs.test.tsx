import type { TaskID } from "@mydoo/tasklens";
import { createTaskLensTestEnvironment } from "@mydoo/tasklens/test";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createTestWrapper } from "../../test/setup";
import { useTaskIntents } from "../intents/use-task-intents";
import { useBreadcrumbs } from "./use-breadcrumbs";

describe("useBreadcrumbs", () => {
  afterEach(() => {
    window.location.hash = "";
  });

  it("should return empty array for root view", async () => {
    vi.clearAllMocks();
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result } = renderHook(() => useBreadcrumbs(undefined), {
      wrapper,
    });

    // Wait for initial Redux sync to complete
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    expect(result.current).toEqual([]);
  });

  it("should return path for nested task", async () => {
    // 1. Setup Data: Root -> Parent -> Child
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result: intents } = renderHook(() => useTaskIntents(), {
      wrapper,
    });

    // Wait for initial sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    let parentId: TaskID;
    let childId: TaskID;

    await act(async () => {
      parentId = intents.current.createTask({ title: "Parent" });
      childId = intents.current.createTask({
        title: "Child",
        parentId: parentId,
      });
    });

    // 2. Test Breadcrumbs when focused on Child
    const { result } = renderHook(() => useBreadcrumbs(childId), {
      wrapper,
    });

    // Should be [Parent, Child]
    await waitFor(() => {
      expect(result.current).toHaveLength(2);
      expect(result.current[0]?.title).toBe("Parent");
      expect(result.current[1]?.title).toBe("Child");
      expect(result.current[0]?.id).toBe(parentId);
      expect(result.current[1]?.id).toBe(childId);
    });
  });
});

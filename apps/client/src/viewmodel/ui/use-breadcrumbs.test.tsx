import { type AutomergeUrl, Repo } from "@automerge/automerge-repo";
import {
  createTaskLensDoc,
  createTaskLensStore,
  type TaskID,
} from "@mydoo/tasklens";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createTestWrapper } from "../../test/setup";
import { useTaskIntents } from "../intents/use-task-intents";
import { useBreadcrumbs } from "./use-breadcrumbs";

describe("useBreadcrumbs", () => {
  let repo: Repo;
  let docUrl: AutomergeUrl;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({ network: [] });
    window.location.hash = "";
    docUrl = createTaskLensDoc(repo);
  });

  afterEach(() => {
    window.location.hash = "";
  });

  it("should return empty array for root view", async () => {
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, docUrl);
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
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, docUrl);
    const { result: intents } = renderHook(() => useTaskIntents(), {
      wrapper,
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

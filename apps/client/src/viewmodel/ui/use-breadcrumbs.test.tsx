import {
  type AutomergeUrl,
  type DocHandle,
  Repo,
} from "@automerge/automerge-repo";
import type { TaskID } from "@mydoo/tasklens";
import type { TunnelState } from "@mydoo/tasklens/persistence";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { createClientStore } from "../../store";
import { createTestWrapper } from "../../test/setup";
import { useTaskIntents } from "../intents/use-task-intents";
import { useBreadcrumbs } from "./use-breadcrumbs";

describe("useBreadcrumbs", () => {
  let repo: Repo;
  let handle: DocHandle<TunnelState>;
  let docUrl: AutomergeUrl;

  beforeEach(() => {
    repo = new Repo({ network: [] });
    window.location.hash = "";

    handle = repo.create<TunnelState>({
      tasks: {},
      rootTaskIds: [],
      places: {},
    });
    docUrl = handle.url;
  });

  afterEach(() => {
    window.location.hash = "";
  });

  it("should return empty array for root view", async () => {
    const store = createClientStore(docUrl, repo);
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
    const store = createClientStore(docUrl, repo);
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result: intents } = renderHook(() => useTaskIntents(), {
      wrapper,
    });

    // Wait for initial sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    let parentId: TaskID;
    act(() => {
      parentId = intents.current.createTask({ title: "Parent" });
    });

    // Wait for parent in Redux
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[parentId])
        throw new Error("Parent not in store");
    });

    let childId: TaskID;
    act(() => {
      childId = intents.current.createTask({
        title: "Child",
        parentId: parentId,
      });
    });

    // Wait for child in Redux
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[childId]) throw new Error("Child not in store");
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

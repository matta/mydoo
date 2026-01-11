import {
  type AutomergeUrl,
  type DocHandle,
  Repo,
} from "@automerge/automerge-repo";
import { createTaskLensStore, type TaskID } from "@mydoo/tasklens";
import { seedTask } from "@mydoo/tasklens/test";
import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { createClientStore } from "../../store";
import { createTestWrapper } from "../../test/setup";
import { useTaskDetails } from "./use-task-details";

describe("useTaskDetails", () => {
  // biome-ignore lint/suspicious/noExplicitAny: test handle
  let handle: DocHandle<any>;
  let repo: Repo;
  let url: AutomergeUrl;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({ network: [] });
    handle = repo.create({ tasks: {}, rootTaskIds: [], places: {} });
    url = handle.url;
  });

  it("returns task details correctly", async () => {
    seedTask(handle, { id: "parent-id", title: "Parent Goal" });
    seedTask(handle, {
      id: "child-id",
      title: "Child Task",
      parentId: "parent-id" as TaskID,
    });
    seedTask(handle, {
      id: "grandchild-id",
      title: "Grandchild",
      parentId: "child-id" as TaskID,
    });

    const store = createClientStore(url, repo);
    const wrapper = createTestWrapper(repo, url, store);
    const { result } = renderHook(() => useTaskDetails("child-id" as TaskID), {
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
    seedTask(handle, { id: "root-id", title: "Root Task" });

    const store = createClientStore(url, repo);
    const wrapper = createTestWrapper(repo, url, store);
    const { result } = renderHook(() => useTaskDetails("root-id" as TaskID), {
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
    const store = createClientStore(url, repo);
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
    const store = createTaskLensStore();
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

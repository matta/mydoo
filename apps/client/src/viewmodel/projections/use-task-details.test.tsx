import { type AutomergeUrl, Repo } from "@automerge/automerge-repo";
import {
  createTaskLensDoc,
  createTaskLensStore,
  type TaskID,
} from "@mydoo/tasklens";

import { act, renderHook, waitFor } from "@testing-library/react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import { createTestWrapper } from "../../test/setup";
import { useTaskIntents } from "../intents/use-task-intents";
import { useTaskDetails } from "./use-task-details";

describe("useTaskDetails", () => {
  let repo: Repo;
  let url: AutomergeUrl;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({ network: [] });
    url = createTaskLensDoc(repo);
  });

  it("returns task details correctly", async () => {
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, url);
    const { result: intents } = renderHook(() => useTaskIntents(), { wrapper });

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
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, url);
    const { result: intents } = renderHook(() => useTaskIntents(), { wrapper });

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
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, url);
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
    const wrapper = createTestWrapper(repo, store, url);
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

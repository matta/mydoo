import {
  type AutomergeUrl,
  type DocHandle,
  Repo,
} from "@automerge/automerge-repo";

import type { TaskID } from "@mydoo/tasklens";
import { seedTask } from "@mydoo/tasklens/test";

import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createClientStore } from "../../store";
import { createTestWrapper } from "../../test/setup";
import { useTaskTree } from "./use-task-tree";

describe("useTaskTree", () => {
  // biome-ignore lint/suspicious/noExplicitAny: test handle
  let handle: DocHandle<any>;
  let repo: Repo;
  let docUrl: AutomergeUrl;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({ network: [] });
    handle = repo.create({ tasks: {}, rootTaskIds: [], places: {} });
    docUrl = handle.url;
  });

  it("builds a task tree from rootTaskIds", async () => {
    seedTask(handle, { id: "root1", title: "Root 1" });
    seedTask(handle, { id: "root2", title: "Root 2" });
    seedTask(handle, {
      id: "child1",
      title: "Child 1",
      parentId: "root1" as TaskID,
    });

    const store = createClientStore(docUrl, repo);
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result } = renderHook(() => useTaskTree(), {
      wrapper,
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
        expect(result.current.roots).toHaveLength(2);
      },
      { timeout: 2000 },
    );

    expect(result.current.roots[0]?.id).toBe("root1");
    expect(result.current.roots[1]?.id).toBe("root2");

    // Verify recursion
    expect(result.current.roots[0]?.children).toHaveLength(1);
    expect(result.current.roots[0]?.children[0]?.id).toBe("child1");
    expect(result.current.roots[1]?.children).toHaveLength(0);
  });

  it("handles loading state initially", async () => {
    const store = createClientStore(docUrl, repo);
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result } = renderHook(() => useTaskTree(), {
      wrapper,
    });

    // Initial state might be loading or empty depending on speed,
    // so we don't strictly assert true/false here to avoid flakes.
    expect(result.current.roots).toEqual([]);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
  });
});

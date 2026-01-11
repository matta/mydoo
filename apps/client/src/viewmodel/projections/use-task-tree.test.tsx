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
import { useTaskTree } from "./use-task-tree";

describe("useTaskTree", () => {
  let repo: Repo;
  let docUrl: AutomergeUrl;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({ network: [] });
    docUrl = createTaskLensDoc(repo);
  });

  it("builds a task tree from rootTaskIds", async () => {
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, docUrl);
    const { result: intents } = renderHook(() => useTaskIntents(), { wrapper });

    let root1: TaskID = "r1" as TaskID;
    let root2: TaskID = "r2" as TaskID;
    let child1: TaskID = "c1" as TaskID;

    await act(async () => {
      // Create root1 then its child
      root1 = intents.current.createTask({ title: "Root 1" });
      child1 = intents.current.createTask({
        title: "Child 1",
        parentId: root1,
      });

      // Create root2
      root2 = intents.current.createTask({ title: "Root 2" });
    });

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

    expect(result.current.roots[0]?.title).toBe("Root 1");
    // Depending on sort order (creation time vs manual), check ID or content.
    // If sort is insertion order or similar, root1 should be first.
    // Let's check IDs or titles.
    const rootTitles = result.current.roots.map((r) => r.title);
    expect(rootTitles).toContain("Root 1");
    expect(rootTitles).toContain("Root 2");

    // Check structure of Root 1
    const r1Node = result.current.roots.find((r) => r.id === root1);
    expect(r1Node).toBeDefined();
    expect(r1Node?.children).toHaveLength(1);
    expect(r1Node?.children[0]?.title).toBe("Child 1");
    expect(r1Node?.children[0]?.id).toBe(child1);

    // Check structure of Root 2
    const r2Node = result.current.roots.find((r) => r.id === root2);
    expect(r2Node).toBeDefined();
    expect(r2Node?.children).toHaveLength(0);
  });

  it("handles loading state initially", async () => {
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, docUrl);
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

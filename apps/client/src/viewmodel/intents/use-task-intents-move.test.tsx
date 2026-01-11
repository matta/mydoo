import type { TaskID } from "@mydoo/tasklens";
import { createTaskLensTestEnvironment } from "@mydoo/tasklens/test";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { createTestWrapper } from "../../test/setup";
import { useTaskIntents } from "./use-task-intents";

describe("useTaskIntents (Move Interactions)", () => {
  afterEach(() => {
    window.location.hash = "";
  });

  it("should indent a task (become child of previous sibling)", async () => {
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result } = renderHook(() => useTaskIntents(), { wrapper });

    // Setup: Root -> [Sibling, Target]
    // Wait for initial sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    let siblingId: TaskID;
    let targetId: TaskID;
    act(() => {
      siblingId = result.current.createTask({ title: "Sibling" });
      targetId = result.current.createTask({ title: "Target" });
    });

    // Wait for Redux to sync the tasks
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[siblingId])
        throw new Error("Sibling not in store");
      if (!state.tasks.entities[targetId])
        throw new Error("Target not in store");
    });

    // Indent target to be child of sibling
    act(() => {
      result.current.indentTask(targetId);
    });

    // Validate structure via Redux Store
    await waitFor(() => {
      const state = store.getState();
      const sibling = state.tasks.entities[siblingId];
      const target = state.tasks.entities[targetId];

      if (!sibling) throw new Error("Sibling task not found");
      if (!target) throw new Error("Target task not found");

      expect(sibling.childTaskIds).toContain(targetId);
      expect(target.parentId).toBe(siblingId);
    });
  });

  it("should outdent a task (become sibling of parent)", async () => {
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result } = renderHook(() => useTaskIntents(), { wrapper });

    // Setup: Root -> Parent -> Child
    // Wait for initial sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    let parentId: TaskID;
    let childId: TaskID;
    act(() => {
      parentId = result.current.createTask({ title: "Parent" });
    });

    // Wait for parent
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[parentId])
        throw new Error("Parent not in store");
    });

    act(() => {
      childId = result.current.createTask({
        title: "Child",
        parentId: parentId,
      });
    });

    // Wait for child
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[childId]) throw new Error("Child not in store");
    });

    act(() => {
      result.current.outdentTask(childId);
    });

    // Validate via Redux Store
    await waitFor(() => {
      const state = store.getState();
      const child = state.tasks.entities[childId];
      if (!child) throw new Error("Child task not found");

      expect(child.parentId).toBeUndefined();
      expect(state.tasks.rootTaskIds).toContain(childId);
    });
  });

  it("should not indent if no previous sibling", async () => {
    const { repo, docUrl, store } = createTaskLensTestEnvironment();
    const wrapper = createTestWrapper(repo, docUrl, store);
    const { result } = renderHook(() => useTaskIntents(), { wrapper });

    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    let id: TaskID;
    act(() => {
      id = result.current.createTask({ title: "Solo" });
    });

    // Wait for solo task
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[id]) throw new Error("Solo task not in store");
    });

    act(() => {
      result.current.indentTask(id);
    });

    // Validate via Redux Store
    await waitFor(() => {
      const state = store.getState();
      const task = state.tasks.entities[id];
      if (!task) throw new Error("Task not found");

      expect(task.parentId).toBeUndefined();
    });
  });
});

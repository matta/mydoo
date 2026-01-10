import { Repo } from "@automerge/automerge-repo";
import { createTaskLensStore, type TaskID } from "@mydoo/tasklens";
import type { TunnelState } from "@mydoo/tasklens/persistence";
import { createEmptyTunnelState } from "@mydoo/tasklens/test";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { createTestWrapper } from "../../test/setup";
import { useTaskDetails } from "../projections/use-task-details";
import { useTaskIntents } from "./use-task-intents";

describe("useTaskIntents", () => {
  let repo: Repo;

  beforeEach(() => {
    repo = new Repo({ network: [] });
    window.location.hash = "";
  });

  afterEach(() => {
    window.location.hash = "";
  });

  it("should create a task", async () => {
    // 1. Setup Document
    const handle = repo.create<TunnelState>(createEmptyTunnelState());
    const docUrl = handle.url;
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, docUrl);

    // 2. Setup Intents Hook
    const { result } = renderHook(() => useTaskIntents(), { wrapper });

    // Wait for initial Redux sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    // 3. Create Task
    let taskId: TaskID;
    act(() => {
      taskId = result.current.createTask({ title: "Buy Milk" });
    });

    // Wait for Redux to sync the new task
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[taskId])
        throw new Error("Task not in Redux yet");
    });

    // 4. Verify in Repo - use doc() for immediate verification after act()
    const doc = handle.doc();
    if (!doc) throw new Error("Doc missing");
    const tasks = Object.values(doc.tasks);

    expect(tasks).toHaveLength(1);
    const createdTask = tasks[0];

    if (!createdTask) throw new Error("Task missing");

    expect(createdTask.title).toBe("Buy Milk");
    expect(createdTask.status).toBe("Pending");
  });

  it("should toggle task completion", async () => {
    // 1. Setup Document
    const handle = repo.create<TunnelState>(createEmptyTunnelState());
    const docUrl = handle.url;
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, docUrl);

    // 2. Setup observer hook to wait for reactive state
    const useObserver = () => {
      const intents = useTaskIntents();
      // This will cause re-render when ANY task in the store updates
      const details = useTaskDetails(undefined);
      return { intents, details };
    };

    const { result } = renderHook(() => useObserver(), { wrapper });

    // 3. Create Task
    let taskId: TaskID;
    act(() => {
      taskId = result.current.intents.createTask({ title: "Walk Dog" });
    });

    // Wait until the hook sees the task in Redux entities
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[taskId])
        throw new Error("Task not in store yet");
    });

    // 4. Toggle Completion
    act(() => {
      result.current.intents.toggleTask(taskId);
    });

    // 5. Verify
    await waitFor(() => {
      // Verify the Automerge document reflects the update.
      const docAfter = handle.doc();
      if (!docAfter) throw new Error("Doc missing in update");
      const taskAfter = docAfter.tasks[taskId];
      if (!taskAfter) throw new Error("Task missing in update");
      expect(taskAfter.status).toBe("Done");
    });

    // 6. Toggle Back
    act(() => {
      result.current.intents.toggleTask(taskId);
    });

    await waitFor(() => {
      const docFinal = handle.doc();
      if (!docFinal) throw new Error("Doc missing final");
      const taskFinal = docFinal.tasks[taskId];
      if (!taskFinal) throw new Error("Task missing in final");
      expect(taskFinal.status).toBe("Pending");
    });
  });

  it("should create a child task with parentId", async () => {
    // 1. Setup Document
    const handle = repo.create<TunnelState>(createEmptyTunnelState());
    const docUrl = handle.url;
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, docUrl);

    // 2. Setup Intents Hook
    const { result } = renderHook(() => useTaskIntents(), { wrapper });

    // Wait for initial Redux sync
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });

    // 3. Create Parent Task
    let parentId: TaskID;
    act(() => {
      parentId = result.current.createTask({ title: "Parent Task" });
    });

    // Wait for parent task to sync to Redux
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[parentId])
        throw new Error("Parent not in Redux yet");
    });

    const docAfterParent = handle.doc();
    if (!docAfterParent) throw new Error("Doc missing after parent creation");
    const parentTask = Object.values(docAfterParent.tasks)[0];
    if (!parentTask) throw new Error("Parent task not found");

    // 4. Create Child Task
    let childId: TaskID;
    act(() => {
      childId = result.current.createTask({
        title: "Child Task",
        parentId: parentTask.id,
      });
    });

    // Wait for child task to sync to Redux
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[childId])
        throw new Error("Child not in Redux yet");
    });

    // 5. Verify Child Task
    const docFinal = handle.doc();
    if (!docFinal) throw new Error("Doc missing final");
    const tasks = Object.values(docFinal.tasks);
    expect(tasks).toHaveLength(2);

    const childTask = tasks.find((t) => t.title === "Child Task");
    if (!childTask) throw new Error("Child task not found");

    // Get fresh parent from final doc
    const parentTaskFinal = docFinal.tasks[parentTask.id];
    if (!parentTaskFinal) throw new Error("Parent task not found in final doc");

    expect(childTask.parentId).toBe(parentTask.id);
    expect(parentTaskFinal.childTaskIds).toContain(childTask.id);
  });
});

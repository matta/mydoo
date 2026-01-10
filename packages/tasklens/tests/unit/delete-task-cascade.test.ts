import { beforeEach, describe, expect, it } from "vitest";

import { TunnelStore } from "../../src/persistence/store";
import type { TaskID } from "../../src/types/persistence"; // Keep TaskID as it's used
import { TaskStatus } from "../../src/types/ui";

describe("deleteTask cascade (hard-delete)", () => {
  let store: TunnelStore;

  beforeEach(() => {
    store = new TunnelStore();
  });

  it("should hard-delete a single task (no children)", () => {
    const task = store.createTask({ title: "Solo Task" });

    const deletedCount = store.deleteTask(task.id);

    expect(deletedCount).toBe(1);
    expect(store.getTask(task.id)).toBeUndefined();
    expect(store.state.rootTaskIds).not.toContain(task.id);
  });

  it("should cascade delete all descendants", () => {
    const parent = store.createTask({ title: "Parent" });
    const child1 = store.createTask({ title: "Child 1", parentId: parent.id });
    const child2 = store.createTask({ title: "Child 2", parentId: parent.id });
    const grandchild = store.createTask({
      title: "Grandchild",
      parentId: child1.id,
    });

    const deletedCount = store.deleteTask(parent.id);

    expect(deletedCount).toBe(4);
    expect(store.getTask(parent.id)).toBeUndefined();
    expect(store.getTask(child1.id)).toBeUndefined();
    expect(store.getTask(child2.id)).toBeUndefined();
    expect(store.getTask(grandchild.id)).toBeUndefined();
    expect(store.state.rootTaskIds).not.toContain(parent.id);
  });

  it("should remove child from parent childTaskIds when deleting a child", () => {
    const parent = store.createTask({ title: "Parent" });
    const child = store.createTask({ title: "Child", parentId: parent.id });

    const deletedCount = store.deleteTask(child.id);

    expect(deletedCount).toBe(1);
    expect(store.getTask(child.id)).toBeUndefined();
    // Child should be removed from parent's childTaskIds
    expect(store.getTask(parent.id)?.childTaskIds).not.toContain(child.id);
    // Parent should still exist and be Pending
    expect(store.getTask(parent.id)?.status).toBe(TaskStatus.Pending);
  });

  it("should return 0 for non-existent task", () => {
    const deletedCount = store.deleteTask("nonexistent" as TaskID);

    expect(deletedCount).toBe(0);
  });
});

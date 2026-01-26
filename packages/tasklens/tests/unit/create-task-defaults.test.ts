import { describe, expect, it } from "vitest";
import { createTask } from "../../src/persistence/ops";
import { TunnelStore } from "../../src/persistence/store";
import {
  ANYWHERE_PLACE_ID,
  type PlaceID,
  type TaskID,
  unwrapScalar,
} from "../../src/types/persistence";

describe("createTask - Default Values and Inheritance", () => {
  it("should set placeId to ANYWHERE_PLACE_ID for root tasks", () => {
    const store = new TunnelStore();
    const task = createTask(store.state, { title: "Root Task" });

    expect(task.placeId).toBe(ANYWHERE_PLACE_ID);
  });

  it("should inherit placeId from parent task", () => {
    const store = new TunnelStore();
    const customPlaceId = "Home" as PlaceID;

    // Create parent with custom placeId
    const parent = createTask(store.state, {
      title: "Parent Task",
      placeId: customPlaceId,
    });

    // Create child without specifying placeId
    const child = createTask(store.state, {
      title: "Child Task",
      parentId: parent.id,
    });

    expect(child.placeId).toBe(customPlaceId);
  });

  it("should allow explicit placeId override even with parent", () => {
    const store = new TunnelStore();
    const parentPlaceId = "Home" as PlaceID;
    const childPlaceId = "Office" as PlaceID;

    const parent = createTask(store.state, {
      title: "Parent Task",
      placeId: parentPlaceId,
    });

    const child = createTask(store.state, {
      title: "Child Task",
      parentId: parent.id,
      placeId: childPlaceId,
    });

    expect(child.placeId).toBe(childPlaceId);
  });

  it("should default to ANYWHERE_PLACE_ID if parent has no placeId", () => {
    const store = new TunnelStore();

    // Create parent without placeId (will get ANYWHERE_PLACE_ID)
    const parent = createTask(store.state, { title: "Parent Task" });

    // Create child
    const child = createTask(store.state, {
      title: "Child Task",
      parentId: parent.id,
    });

    expect(child.placeId).toBe(ANYWHERE_PLACE_ID);
  });

  it("should set importance to 1.0 by default", () => {
    const store = new TunnelStore();
    const task = createTask(store.state, { title: "Test Task" });

    expect(task.importance).toBe(1.0);
  });

  it("should set status to Pending by default", () => {
    const store = new TunnelStore();
    const task = createTask(store.state, { title: "Test Task" });

    expect(unwrapScalar(task.status)).toBe("Pending");
  });

  it("should generate a valid UUID for task id", () => {
    const store = new TunnelStore();
    const task = createTask(store.state, { title: "Test Task" });

    expect(task.id).toBeTruthy();
    expect(typeof task.id).toBe("string");
    expect(task.id.length).toBeGreaterThanOrEqual(32); // UUIDs are 36 chars
  });

  it("should initialize childTaskIds as empty array", () => {
    const store = new TunnelStore();
    const task = createTask(store.state, { title: "Test Task" });

    expect(task.childTaskIds).toEqual([]);
  });

  it("should add child to parent childTaskIds array", () => {
    const store = new TunnelStore();

    const parent = createTask(store.state, { title: "Parent" });
    const child = createTask(store.state, {
      title: "Child",
      parentId: parent.id,
    });

    const updatedParent = store.state.tasks[parent.id];
    expect(updatedParent?.childTaskIds).toContain(child.id);
  });

  it('should default dueDate to Now for "Routinely" tasks if not provided', () => {
    const store = new TunnelStore();
    const task = createTask(store.state, {
      title: "Routine Task",
      schedule: {
        type: "Routinely",
        leadTime: 1000,
      },
    });

    expect(task.schedule.dueDate).toBeDefined();
    expect(task.schedule.dueDate).toBeLessThanOrEqual(Date.now());
  });

  describe("Positioning", () => {
    it("should add child to end by default", () => {
      const store = new TunnelStore();
      const parent = createTask(store.state, { title: "Parent" });
      createTask(store.state, { title: "First", parentId: parent.id });
      createTask(store.state, { title: "Second", parentId: parent.id });

      const updatedParent = store.state.tasks[parent.id];
      const childIds = updatedParent?.childTaskIds ?? [];
      expect(childIds.length).toBe(2);
      if (childIds[0])
        expect(store.state.tasks[childIds[0]]?.title).toBe("First");
      if (childIds[1])
        expect(store.state.tasks[childIds[1]]?.title).toBe("Second");
    });

    it("should add child to start when specified", () => {
      const store = new TunnelStore();
      const parent = createTask(store.state, { title: "Parent" });
      createTask(store.state, { title: "First", parentId: parent.id });
      createTask(
        store.state,
        { title: "New First", parentId: parent.id },
        { position: "start" },
      );

      const updatedParent = store.state.tasks[parent.id];
      const childIds = updatedParent?.childTaskIds ?? [];
      expect(childIds[0]).toBeDefined();
      if (childIds[0])
        expect(store.state.tasks[childIds[0]]?.title).toBe("New First");
    });

    it("should add child after specific task", () => {
      const store = new TunnelStore();
      const parent = createTask(store.state, { title: "Parent" });
      const first = createTask(store.state, {
        title: "First",
        parentId: parent.id,
      });
      createTask(store.state, { title: "Second", parentId: parent.id });

      createTask(
        store.state,
        { title: "Inserted", parentId: parent.id },
        { position: "after", afterTaskId: first.id },
      );

      const updatedParent = store.state.tasks[parent.id];
      const childIds = updatedParent?.childTaskIds ?? [];
      if (childIds[1])
        expect(store.state.tasks[childIds[1]]?.title).toBe("Inserted");
    });

    it("should fallback to end if afterTaskId is not found", () => {
      const store = new TunnelStore();
      const parent = createTask(store.state, { title: "Parent" });
      createTask(store.state, { title: "First", parentId: parent.id });

      createTask(
        store.state,
        { title: "Fallback", parentId: parent.id },
        { position: "after", afterTaskId: "non-existent" as TaskID },
      );

      const updatedParent = store.state.tasks[parent.id];
      const childTaskIds = updatedParent?.childTaskIds ?? [];
      const lastId = childTaskIds[childTaskIds.length - 1];
      if (lastId) {
        expect(store.state.tasks[lastId]?.title).toBe("Fallback");
      }
    });
  });
});

/* eslint-disable no-restricted-syntax */
import { describe, expect, it } from "vitest";

import type { TaskLensState } from "../../src/store/index";
import {
  selectRootTaskIds,
  selectStoreReady,
  selectTaskById,
  selectTaskEntities,
} from "../../src/store/selectors";
import type { ComputedTask } from "../../src/types";
import { type TaskID, TaskStatus } from "../../src/types";

describe("Redux Selectors", () => {
  const createMockState = (
    overrides?: Partial<TaskLensState["tasks"]>,
  ): TaskLensState => ({
    tasks: {
      entities: {},
      rootTaskIds: [],
      todoListIds: [],
      lastProxyDoc: {} as unknown as TaskLensState["tasks"]["lastProxyDoc"], // Mocking the proxy doc as we shouldn't access it
      ...overrides,
    },
  });

  describe("selectTaskEntities", () => {
    it("should return empty object when entities are empty", () => {
      const state = createMockState({ entities: {} });
      const result = selectTaskEntities(state);
      expect(result).toEqual({});
    });

    it("should return the entities map", () => {
      const mockTask = {
        id: "t1" as TaskID,
        title: "Test",
        status: TaskStatus.Pending,
      } as unknown as ComputedTask;

      const state = createMockState({
        entities: { ["t1" as TaskID]: mockTask },
      });

      const result = selectTaskEntities(state);
      expect(result).toEqual({ t1: mockTask });
    });
  });

  describe("selectTaskById", () => {
    it("should return undefined if ID is undefined", () => {
      const state = createMockState();
      const selector = selectTaskById(undefined);
      expect(selector(state)).toBeUndefined();
    });

    it("should return undefined if task does not exist", () => {
      const state = createMockState();
      const selector = selectTaskById("missing-id" as TaskID);
      expect(selector(state)).toBeUndefined();
    });

    it("should return the task if it exists", () => {
      const mockTask = {
        id: "t1" as TaskID,
        title: "Found me",
      } as unknown as ComputedTask;

      const state = createMockState({
        entities: { ["t1" as TaskID]: mockTask },
      });

      const selector = selectTaskById("t1" as TaskID);
      expect(selector(state)).toEqual(mockTask);
    });
  });

  describe("selectRootTaskIds", () => {
    it("should return empty array when no root tasks exist", () => {
      const state = createMockState({ rootTaskIds: [] });
      const result = selectRootTaskIds(state);
      expect(result).toEqual([]);
    });

    it("should return the rootTaskIds array", () => {
      const rootIds = ["r1" as TaskID, "r2" as TaskID, "r3" as TaskID];
      const state = createMockState({ rootTaskIds: rootIds });
      const result = selectRootTaskIds(state);
      expect(result).toEqual(rootIds);
    });
  });

  describe("selectStoreReady", () => {
    it("should return false when lastProxyDoc is null", () => {
      const state = createMockState({ lastProxyDoc: null });
      const result = selectStoreReady(state);
      expect(result).toBe(false);
    });

    it("should return true when lastProxyDoc is set", () => {
      const state = createMockState({
        lastProxyDoc: {} as unknown as TaskLensState["tasks"]["lastProxyDoc"],
      });
      const result = selectStoreReady(state);
      expect(result).toBe(true);
    });
  });
});

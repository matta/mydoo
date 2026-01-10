import {
  type DocHandle,
  Repo,
  type StorageAdapterInterface,
  type StorageKey,
} from "@automerge/automerge-repo";
import { createTaskLensStore, syncDoc, TaskStatus } from "@mydoo/tasklens";
import type { TunnelState } from "@mydoo/tasklens/persistence";
import { seedTask } from "@mydoo/tasklens/test";
import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTestWrapper } from "../../test/setup";
import { usePriorityList } from "./use-priority-list";

// Dummy Storage Adapter
class DummyStorageAdapter implements StorageAdapterInterface {
  async load(_key: StorageKey): Promise<Uint8Array | undefined> {
    return undefined;
  }
  async save(_key: StorageKey, _data: Uint8Array): Promise<void> {}
  async remove(_key: StorageKey): Promise<void> {}
  async loadRange(
    _keyPrefix: StorageKey,
  ): Promise<{ data: Uint8Array; key: StorageKey }[]> {
    return [];
  }
  async removeRange(_keyPrefix: StorageKey): Promise<void> {}
}

describe("usePriorityList", () => {
  let handle: DocHandle<TunnelState>;
  let repo: Repo;
  let store: ReturnType<typeof createTaskLensStore>;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({ network: [], storage: new DummyStorageAdapter() });
    handle = repo.create({ tasks: {}, rootTaskIds: [], places: {} });
    store = createTaskLensStore();
  });

  const renderWithSync = async () => {
    // Sync the current doc state to Redux before rendering
    const doc = handle.doc();
    if (doc) {
      await store.dispatch(
        syncDoc({
          proxyDoc: doc as TunnelState,
          parsedDoc: doc as TunnelState,
        }),
      );
    }
    return renderHook(() => usePriorityList(), {
      wrapper: createTestWrapper(repo, store),
    });
  };

  it("filters out completed tasks that are acknowledged", async () => {
    seedTask(handle, {
      id: "1",
      title: "Todo 1",
      status: TaskStatus.Pending,
      importance: 0.5,
    });
    seedTask(handle, {
      id: "2",
      title: "Done 1",
      status: TaskStatus.Done,
      importance: 0.5,
      isAcknowledged: true,
    });

    const { result } = await renderWithSync();

    await waitFor(() => {
      expect(result.current.tasks).toHaveLength(1);
    });

    expect(result.current.tasks[0]?.id).toBe("1");
    expect(result.current.isLoading).toBe(false);
  });

  it("includes completed tasks that are NOT acknowledged", async () => {
    seedTask(handle, {
      id: "1",
      title: "Todo 1",
      status: TaskStatus.Pending,
      importance: 0.5,
    });
    seedTask(handle, {
      id: "2",
      title: "Done Unacked",
      status: TaskStatus.Done,
      importance: 0.5,
      isAcknowledged: false,
    });

    const { result } = await renderWithSync();

    await waitFor(() => {
      expect(result.current.tasks).toHaveLength(2);
    });

    expect(result.current.tasks.map((t) => t.id)).toEqual(
      expect.arrayContaining(["1", "2"]),
    );
  });

  it("sorts tasks by priority (descending)", async () => {
    seedTask(handle, {
      id: "1",
      title: "Low Priority",
      status: TaskStatus.Pending,
      importance: 0.1,
    });
    seedTask(handle, {
      id: "2",
      title: "High Priority",
      status: TaskStatus.Pending,
      importance: 0.9,
    });
    seedTask(handle, {
      id: "3",
      title: "Medium Priority",
      status: TaskStatus.Pending,
      importance: 0.5,
    });

    const { result } = await renderWithSync();

    await waitFor(() => {
      expect(result.current.tasks).toHaveLength(3);
    });

    expect(result.current.tasks).toMatchObject([
      { id: "2" }, // High
      { id: "3" }, // Medium
      { id: "1" }, // Low
    ]);
  });

  it("returns loading state initially", async () => {
    // Without syncing, Redux store is empty, so isLoading should be true
    const { result } = renderHook(() => usePriorityList(), {
      wrapper: createTestWrapper(repo, store),
    });
    expect(result.current.tasks).toEqual([]);
    expect(result.current.isLoading).toBe(true);
  });
});

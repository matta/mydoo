import {
  type DocHandle,
  Repo,
  type StorageAdapterInterface,
  type StorageKey,
} from '@automerge/automerge-repo';
import {
  createStore,
  syncDoc,
  type TaskID,
  TaskStatus,
  type TunnelNode,
  type TunnelState,
} from '@mydoo/tasklens';
import {renderHook, waitFor} from '@testing-library/react';
import {beforeEach, describe, expect, it, vi} from 'vitest';
import {createTestWrapper} from '../../test/setup';
import {usePriorityList} from './use-priority-list';

// Dummy Storage Adapter
class DummyStorageAdapter implements StorageAdapterInterface {
  async load(_key: StorageKey): Promise<Uint8Array | undefined> {
    return undefined;
  }
  async save(_key: StorageKey, _data: Uint8Array): Promise<void> {}
  async remove(_key: StorageKey): Promise<void> {}
  async loadRange(
    _keyPrefix: StorageKey,
  ): Promise<{data: Uint8Array; key: StorageKey}[]> {
    return [];
  }
  async removeRange(_keyPrefix: StorageKey): Promise<void> {}
}

const createMockTask = (
  id: string,
  title: string,
  status: TaskStatus,
  importance: number,
  isAcknowledged = false,
): TunnelNode => {
  // TODO: Move this to @mydoo/tasklens/test-utils (see ROLLING_CONTEXT.md)
  // biome-ignore lint/suspicious/noExplicitAny: Building mock object incrementally
  const node: any = {
    childTaskIds: [],
    children: [],
    creditIncrement: 1,
    credits: 0,
    creditsTimestamp: Date.now(),
    desiredCredits: 0,
    id: id as TaskID,
    importance,
    isContainer: false,
    isPending: status === TaskStatus.Pending,
    isReady: true,
    isSequential: false,
    priorityTimestamp: Date.now(),
    schedule: {
      leadTime: 0,
      type: 'Once' as const,
      dueDate: Date.now() + 86400000,
    },
    status,
    title,
    isAcknowledged,
    notes: '',
  };
  return node as TunnelNode;
};

describe('usePriorityList', () => {
  let handle: DocHandle<TunnelState>;
  let repo: Repo;
  let store: ReturnType<typeof createStore>;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({network: [], storage: new DummyStorageAdapter()});
    handle = repo.create({tasks: {}, rootTaskIds: [], places: {}});
    store = createStore();
  });

  const renderWithSync = async () => {
    // Sync the current doc state to Redux before rendering
    const doc = handle.docSync();
    if (doc) {
      await store.dispatch(syncDoc(doc));
    }
    return renderHook(() => usePriorityList(), {
      wrapper: createTestWrapper(repo, store),
    });
  };

  it('filters out completed tasks that are acknowledged', async () => {
    handle.change((doc: TunnelState) => {
      const task1 = createMockTask('1', 'Todo 1', TaskStatus.Pending, 0.5);
      const task2 = createMockTask('2', 'Done 1', TaskStatus.Done, 0.5, true);
      doc.tasks['1' as TaskID] = task1;
      doc.tasks['2' as TaskID] = task2;
      doc.rootTaskIds = ['1' as TaskID, '2' as TaskID];
    });

    const {result} = await renderWithSync();

    await waitFor(() => {
      expect(result.current.tasks).toHaveLength(1);
    });

    expect(result.current.tasks[0]?.id).toBe('1');
    expect(result.current.isLoading).toBe(false);
  });

  it('includes completed tasks that are NOT acknowledged', async () => {
    handle.change((doc: TunnelState) => {
      const task1 = createMockTask('1', 'Todo 1', TaskStatus.Pending, 0.5);
      const task2 = createMockTask(
        '2',
        'Done Unacked',
        TaskStatus.Done,
        0.5,
        false,
      );
      doc.tasks['1' as TaskID] = task1;
      doc.tasks['2' as TaskID] = task2;
      doc.rootTaskIds = ['1' as TaskID, '2' as TaskID];
    });

    const {result} = await renderWithSync();

    await waitFor(() => {
      expect(result.current.tasks).toHaveLength(2);
    });

    expect(result.current.tasks.map(t => t.id)).toEqual(
      expect.arrayContaining(['1', '2']),
    );
  });

  it('sorts tasks by priority (descending)', async () => {
    handle.change((doc: TunnelState) => {
      const task1 = createMockTask(
        '1',
        'Low Priority',
        TaskStatus.Pending,
        0.1,
      );
      const task2 = createMockTask(
        '2',
        'High Priority',
        TaskStatus.Pending,
        0.9,
      );
      const task3 = createMockTask(
        '3',
        'Medium Priority',
        TaskStatus.Pending,
        0.5,
      );
      doc.tasks['1' as TaskID] = task1;
      doc.tasks['2' as TaskID] = task2;
      doc.tasks['3' as TaskID] = task3;
      doc.rootTaskIds = ['1' as TaskID, '2' as TaskID, '3' as TaskID];
    });

    const {result} = await renderWithSync();

    await waitFor(() => {
      expect(result.current.tasks).toHaveLength(3);
    });

    expect(result.current.tasks).toMatchObject([
      {id: '2'}, // High
      {id: '3'}, // Medium
      {id: '1'}, // Low
    ]);
  });

  it('returns loading state initially', async () => {
    // Without syncing, Redux store is empty, so isLoading should be true
    const {result} = renderHook(() => usePriorityList(), {
      wrapper: createTestWrapper(repo, store),
    });
    expect(result.current.tasks).toEqual([]);
    expect(result.current.isLoading).toBe(true);
  });
});

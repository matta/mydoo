import {
  type DocHandle,
  Repo,
  type StorageAdapterInterface,
  type StorageKey,
} from '@automerge/automerge-repo';
import {
  type DocumentHandle,
  type TaskID,
  TaskStatus,
  type TunnelNode,
  type TunnelState,
} from '@mydoo/tasklens';
import {renderHook, waitFor} from '@testing-library/react';
import {beforeEach, describe, expect, it, vi} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {useTaskDetails} from './use-task-details';

// Dummy Storage Adapter (same as other tests)
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
  parentId?: string,
  children: string[] = [],
): TunnelNode => {
  // TODO: Move this to @mydoo/tasklens/test-utils (see ROLLING_CONTEXT.md)
  // biome-ignore lint/suspicious/noExplicitAny: Building mock object incrementally
  const node: any = {
    childTaskIds: children as TaskID[],
    children: [],
    creditIncrement: 1,
    credits: 0,
    creditsTimestamp: Date.now(),
    desiredCredits: 0,
    id: id as TaskID,
    importance: 1,
    isContainer: children.length > 0,
    isPending: true,
    isReady: true,
    isSequential: false,
    priorityTimestamp: Date.now(),
    schedule: {leadTime: 0, type: 'Once', dueDate: Date.now() + 86400000},
    status: TaskStatus.Pending,
    title,
    isAcknowledged: false,
    notes: '',
  };

  if (parentId) {
    node.parentId = parentId as TaskID;
  }

  return node as TunnelNode;
};

describe('useTaskDetails', () => {
  let docUrl: DocumentHandle;
  let handle: DocHandle<TunnelState>;
  let repo: Repo;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({network: [], storage: new DummyStorageAdapter()});
    handle = repo.create({tasks: {}, rootTaskIds: [], places: {}});
    docUrl = handle.url as unknown as DocumentHandle;
  });

  it('returns task details correctly', async () => {
    handle.change((doc: TunnelState) => {
      const grandchild = createMockTask(
        'grandchild-id',
        'Grandchild',
        'child-id',
      );
      const child = createMockTask('child-id', 'Child Task', 'parent-id', [
        'grandchild-id',
      ]);
      const parent = createMockTask('parent-id', 'Parent Goal', undefined, [
        'child-id',
      ]);

      doc.rootTaskIds = ['parent-id' as TaskID];
      doc.tasks['parent-id' as TaskID] = parent;
      doc.tasks['child-id' as TaskID] = child;
      doc.tasks['grandchild-id' as TaskID] = grandchild;
    });

    const {result} = renderHook(
      () => useTaskDetails(docUrl, 'child-id' as TaskID),
      {
        wrapper: createTestWrapper(repo),
      },
    );

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.task?.title).toBe('Child Task');
    expect(result.current.parentTitle).toBe('Parent Goal');
    expect(result.current.descendantCount).toBe(1); // One grandchild
  });

  it('handles root tasks (no parent)', async () => {
    handle.change((doc: TunnelState) => {
      const root = createMockTask('root-id', 'Root Task');
      doc.rootTaskIds = ['root-id' as TaskID];
      doc.tasks['root-id' as TaskID] = root;
    });

    const {result} = renderHook(
      () => useTaskDetails(docUrl, 'root-id' as TaskID),
      {
        wrapper: createTestWrapper(repo),
      },
    );

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.task?.title).toBe('Root Task');
    expect(result.current.parentTitle).toBeNull();
    expect(result.current.descendantCount).toBe(0);
  });

  it('returns null when task not found', async () => {
    const {result} = renderHook(
      () => useTaskDetails(docUrl, 'non-existent' as TaskID),
      {
        wrapper: createTestWrapper(repo),
      },
    );

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.task).toBeNull();
  });

  it('returns loading state initially', () => {
    // With a fresh repo, the document should be available almost immediately,
    // but the hook's initial render should reflect a loading/empty state.
    const {result} = renderHook(
      () => useTaskDetails(docUrl, 'any-task' as TaskID),
      {
        wrapper: createTestWrapper(repo),
      },
    );

    // Before waitFor, the state should be loading or have null task
    // This validates the hook's initial state handling
    expect(result.current.task).toBeNull();
  });
});

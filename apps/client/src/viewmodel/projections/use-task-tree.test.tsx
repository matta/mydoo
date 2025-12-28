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
import {useTaskTree} from './use-task-tree';

// Dummy Storage Adapter (same as PlanViewContainer)
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
    children: [], // TunnelNode extra prop, harmless in Automerge if just stored
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

describe('useTaskTree', () => {
  let docUrl: DocumentHandle;
  let handle: DocHandle<TunnelState>;
  let repo: Repo;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({network: [], storage: new DummyStorageAdapter()});
    handle = repo.create({tasks: {}, rootTaskIds: [], places: {}});
    docUrl = handle.url as unknown as DocumentHandle;
  });

  it('builds a task tree from rootTaskIds', async () => {
    // Populate the real doc
    handle.change((doc: TunnelState) => {
      doc.rootTaskIds = ['root1' as TaskID, 'root2' as TaskID];

      const root1 = createMockTask('root1', 'Root 1', undefined, [
        'child1' as TaskID,
      ]);
      doc.tasks['root1' as TaskID] = root1;
      const root2 = createMockTask('root2', 'Root 2', undefined, []);
      doc.tasks['root2' as TaskID] = root2;

      const child1 = createMockTask('child1', 'Child 1', 'root1', []);
      doc.tasks['child1' as TaskID] = child1;
    });

    const {result} = renderHook(() => useTaskTree(docUrl), {
      wrapper: createTestWrapper(repo),
    });

    // Wait for the hook to load the doc
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.roots).toHaveLength(2);
    expect(result.current.roots[0]?.id).toBe('root1');
    expect(result.current.roots[1]?.id).toBe('root2');

    // Verify recursion
    expect(result.current.roots[0]?.children).toHaveLength(1);
    expect(result.current.roots[0]?.children[0]?.id).toBe('child1');
    expect(result.current.roots[1]?.children).toHaveLength(0);
  });

  it('handles loading state initially', async () => {
    // We can't easily force "loading" forever with real repo,
    // but we can check initial state before wait
    const {result} = renderHook(() => useTaskTree(docUrl), {
      wrapper: createTestWrapper(repo),
    });

    // Initial state might be loading or empty depending on speed,
    // but typically useTunne starts loading.
    // However, with real repo, "doc" becomes available quickly.
    // Let's assert that eventually it settles.
    expect(result.current.roots).toEqual([]);

    // We check that it eventually works (sanity check)
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
  });
});

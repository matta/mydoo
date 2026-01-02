import {
  type AutomergeUrl,
  type DocHandle,
  Repo,
} from '@automerge/automerge-repo';
import {
  createMockTask as createSharedMockTask,
  createTaskLensStore,
  type TaskID,
  type TunnelNode,
  type TunnelState,
} from '@mydoo/tasklens';
import {renderHook, waitFor} from '@testing-library/react';
import {beforeEach, describe, expect, it, vi} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {useTaskDetails} from './use-task-details';

const createMockTask = (
  id: string,
  title: string,
  parentId?: string,
  children: string[] = [],
): TunnelNode => {
  return {
    ...createSharedMockTask({
      id: id as TaskID,
      title,
      parentId: parentId as TaskID | undefined,
      childTaskIds: children as TaskID[],
      isContainer: children.length > 0,
    }),
    children: [], // TunnelNode requires children array
  };
};

describe('useTaskDetails', () => {
  let handle: DocHandle<TunnelState>;
  let repo: Repo;
  let url: AutomergeUrl;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({network: []});
    handle = repo.create({tasks: {}, rootTaskIds: [], places: {}});
    url = handle.url;
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

    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, url);
    const {result} = renderHook(() => useTaskDetails('child-id' as TaskID), {
      wrapper,
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
        expect(result.current.task?.title).toBe('Child Task');
        expect(result.current.parentTitle).toBe('Parent Goal');
        expect(result.current.descendantCount).toBe(1); // One grandchild
      },
      {timeout: 2000},
    );
  });

  it('handles root tasks (no parent)', async () => {
    handle.change((doc: TunnelState) => {
      const root = createMockTask('root-id', 'Root Task');
      doc.rootTaskIds = ['root-id' as TaskID];
      doc.tasks['root-id' as TaskID] = root;
    });

    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, url);
    const {result} = renderHook(() => useTaskDetails('root-id' as TaskID), {
      wrapper,
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
        expect(result.current.task?.title).toBe('Root Task');
        expect(result.current.parentTitle).toBeUndefined();
        expect(result.current.descendantCount).toBe(0);
      },
      {timeout: 2000},
    );
  });

  it('returns null when task not found', async () => {
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, url);
    const {result} = renderHook(
      () => useTaskDetails('non-existent' as TaskID),
      {
        wrapper,
      },
    );

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
        expect(result.current.task).toBeUndefined();
      },
      {timeout: 2000},
    );
  });

  it('returns loading state initially', async () => {
    const store = createTaskLensStore();
    const wrapper = createTestWrapper(repo, store, url);
    const {result} = renderHook(() => useTaskDetails('any-task' as TaskID), {
      wrapper,
    });

    // Initial state should be loading
    expect(result.current.isLoading).toBe(true);
    expect(result.current.task).toBeUndefined();

    // Wait for Redux sync to complete to avoid act() warning
    await waitFor(() => {
      expect(store.getState().tasks.lastProxyDoc).toBeDefined();
    });
  });
});

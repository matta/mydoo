import {
  type AutomergeUrl,
  type DocHandle,
  Repo,
} from '@automerge/automerge-repo';
import {
  createMockTask as createSharedMockTask,
  createStore,
  type TaskID,
  type TunnelNode,
  type TunnelState,
} from '@mydoo/tasklens';
import {renderHook, waitFor} from '@testing-library/react';
import {beforeEach, describe, expect, it, vi} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {useTaskTree} from './use-task-tree';

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
    children: [],
  };
};

describe('useTaskTree', () => {
  let handle: DocHandle<TunnelState>;
  let repo: Repo;
  let docUrl: AutomergeUrl;

  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({network: []});
    handle = repo.create({tasks: {}, rootTaskIds: [], places: {}});
    docUrl = handle.url;
  });

  it('builds a task tree from rootTaskIds', async () => {
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

    const store = createStore();
    const wrapper = createTestWrapper(repo, store, docUrl);
    const {result} = renderHook(() => useTaskTree(), {
      wrapper,
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
        expect(result.current.roots).toHaveLength(2);
      },
      {timeout: 2000},
    );

    expect(result.current.roots[0]?.id).toBe('root1');
    expect(result.current.roots[1]?.id).toBe('root2');

    // Verify recursion
    expect(result.current.roots[0]?.children).toHaveLength(1);
    expect(result.current.roots[0]?.children[0]?.id).toBe('child1');
    expect(result.current.roots[1]?.children).toHaveLength(0);
  });

  it('handles loading state initially', async () => {
    const store = createStore();
    const wrapper = createTestWrapper(repo, store, docUrl);
    const {result} = renderHook(() => useTaskTree(), {
      wrapper,
    });

    // Initial state might be loading or empty depending on speed,
    // so we don't strictly assert true/false here to avoid flakes.
    expect(result.current.roots).toEqual([]);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
  });
});

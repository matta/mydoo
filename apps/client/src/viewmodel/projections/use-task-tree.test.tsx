import type {
  DocumentHandle,
  TaskID,
  TunnelNode,
  TunnelState,
} from '@mydoo/tasklens';
import {TaskStatus} from '@mydoo/tasklens';
import {renderHook} from '@testing-library/react';
import {beforeEach, describe, expect, it, vi} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {useTaskTree} from './use-task-tree';

// Mock @mydoo/tasklens
const mockUseTunnel = vi.fn();

vi.mock('@mydoo/tasklens', async importOriginal => {
  const actual = await importOriginal<typeof import('@mydoo/tasklens')>();
  return {
    ...actual,
    useTunnel: (url: DocumentHandle) => mockUseTunnel(url),
  };
});

// Mock helpers
const createMockTask = (
  id: string,
  title: string,
  parentId?: string,
  childTaskIds: string[] = [],
): TunnelNode => ({
  childTaskIds: childTaskIds as TaskID[],
  children: [],
  creditIncrement: 1,
  credits: 0,
  creditsTimestamp: 0,
  desiredCredits: 0,
  id: id as TaskID,
  importance: 1,
  isContainer: childTaskIds.length > 0,
  isPending: true,
  isReady: true,
  isSequential: false,
  priorityTimestamp: 0,
  schedule: {leadTime: 0, type: 'Once'},
  status: TaskStatus.Pending,
  title,
  parentId: parentId as TaskID | undefined,
  isAcknowledged: false,
  notes: '',
});

describe('useTaskTree', () => {
  const mockDocUrl = 'automerge:test' as DocumentHandle;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('builds a task tree from rootTaskIds', () => {
    // Setup: Root 1, Root 2
    // Root 1 has Child 1
    const taskRoot1 = createMockTask('root1', 'Root 1', undefined, ['child1']);
    const taskRoot2 = createMockTask('root2', 'Root 2', undefined, []);
    const taskChild1 = createMockTask('child1', 'Child 1', 'root1', []);

    const mockState: Partial<TunnelState> = {
      rootTaskIds: ['root1', 'root2'] as TaskID[],
      tasks: {
        root1: taskRoot1,
        root2: taskRoot2,
        child1: taskChild1,
      } as Record<TaskID, TunnelNode>,
    } as unknown as TunnelState;

    mockUseTunnel.mockReturnValue({doc: mockState});

    const {result} = renderHook(() => useTaskTree(mockDocUrl), {
      wrapper: createTestWrapper(),
    });

    expect(result.current.roots).toHaveLength(2);
    expect(result.current.roots[0]?.id).toBe('root1');
    expect(result.current.roots[1]?.id).toBe('root2');

    // Verify recursion
    expect(result.current.roots[0]?.children).toHaveLength(1);
    expect(result.current.roots[0]?.children[0]?.id).toBe('child1');
    expect(result.current.roots[1]?.children).toHaveLength(0);
  });

  it('handles loading state', () => {
    mockUseTunnel.mockReturnValue({doc: undefined});
    const {result} = renderHook(() => useTaskTree(mockDocUrl), {
      wrapper: createTestWrapper(),
    });
    expect(result.current.isLoading).toBe(true);
    expect(result.current.roots).toEqual([]);
  });
});

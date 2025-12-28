import {
  type DocumentHandle,
  type Task,
  type TaskID,
  TaskStatus,
  type TunnelState,
} from '@mydoo/tasklens';
import {renderHook} from '@testing-library/react';
import {beforeEach, describe, expect, it, vi} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {useTaskDetails} from './use-task-details';

// Mock @mydoo/tasklens
const mockUseTunnel = vi.fn();

vi.mock('@mydoo/tasklens', async importOriginal => {
  const actual = await importOriginal<typeof import('@mydoo/tasklens')>();
  return {
    ...actual,
    useTunnel: (url: DocumentHandle) =>
      mockUseTunnel(url) as ReturnType<
        typeof import('@mydoo/tasklens')['useTunnel']
      >,
  };
});

describe('useTaskDetails', () => {
  const mockDocUrl = 'automerge:123' as DocumentHandle;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  const createMockTask = (id: string, title: string, parentId?: string): Task =>
    ({
      id: id as TaskID,
      title,
      parentId: parentId ? (parentId as TaskID) : undefined,
      childTaskIds: [],
      status: TaskStatus.Pending,
      importance: 1.0,
      creditIncrement: 1.0,
      credits: 0,
      desiredCredits: 0,
      creditsTimestamp: 0,
      priorityTimestamp: 0,
      schedule: {leadTime: 0, type: 'Once'},
      isSequential: false,
      isAcknowledged: false,
      notes: '',
      isContainer: false,
      isPending: true,
      isReady: true,
    }) as Task;

  it('returns task details correctly', () => {
    const parentTask = createMockTask('parent-id', 'Parent Goal');
    const childTask = createMockTask('child-id', 'Child Task', 'parent-id');
    const grandchildTask = createMockTask(
      'grandchild-id',
      'Grandchild',
      'child-id',
    );

    parentTask.childTaskIds = [childTask.id];
    childTask.childTaskIds = [grandchildTask.id];

    const mockState: TunnelState = {
      nextPlaceId: 1,
      places: {},
      tasks: {
        [parentTask.id]: parentTask,
        [childTask.id]: childTask,
        [grandchildTask.id]: grandchildTask,
      },
      rootTaskIds: [parentTask.id],
    };

    mockUseTunnel.mockReturnValue({doc: mockState});

    const {result} = renderHook(
      () => useTaskDetails(mockDocUrl, childTask.id),
      {
        wrapper: createTestWrapper(),
      },
    );

    expect(result.current.task?.title).toBe('Child Task');
    expect(result.current.parentTitle).toBe('Parent Goal');
    expect(result.current.descendantCount).toBe(1); // One grandchild
    expect(result.current.isLoading).toBe(false);
  });

  it('handles root tasks (no parent)', () => {
    const rootTask = createMockTask('root-id', 'Root Task');
    const mockState: TunnelState = {
      nextPlaceId: 1,
      places: {},
      tasks: {
        [rootTask.id]: rootTask,
      },
      rootTaskIds: [rootTask.id],
    };

    mockUseTunnel.mockReturnValue({doc: mockState});

    const {result} = renderHook(() => useTaskDetails(mockDocUrl, rootTask.id), {
      wrapper: createTestWrapper(),
    });

    expect(result.current.task?.title).toBe('Root Task');
    expect(result.current.parentTitle).toBeNull();
    expect(result.current.descendantCount).toBe(0);
  });

  it('returns null when task not found', () => {
    const mockState: TunnelState = {
      nextPlaceId: 1,
      nextTaskId: 0,
      places: {},
      tasks: {},
      rootTaskIds: [],
    };

    mockUseTunnel.mockReturnValue({doc: mockState});

    const {result} = renderHook(
      () => useTaskDetails(mockDocUrl, 'non-existent' as TaskID),
      {
        wrapper: createTestWrapper(),
      },
    );

    expect(result.current.task).toBeNull();
    expect(result.current.isLoading).toBe(false);
  });

  it('returns loading state when doc is undefined', () => {
    mockUseTunnel.mockReturnValue({doc: undefined});

    const {result} = renderHook(
      () => useTaskDetails(mockDocUrl, 'any' as TaskID),
      {
        wrapper: createTestWrapper(),
      },
    );

    expect(result.current.isLoading).toBe(true);
    expect(result.current.task).toBeNull();
  });
});

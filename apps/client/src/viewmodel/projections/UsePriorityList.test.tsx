import type {
  DocumentHandle,
  TaskID,
  TunnelNode,
  TunnelState,
} from '@mydoo/tasklens';
import {TaskStatus} from '@mydoo/tasklens';
import {renderHook} from '@testing-library/react';
import {beforeEach, describe, expect, it, vi} from 'vitest';

import {usePriorityList} from './usePriorityList';

// Mock @mydoo/tasklens
const mockUseTunnel = vi.fn();
// We don't mock selectPriorityList because it's a pure function and we want to test integration with it,
// OR we mock it to test the HOOK only. Given the user wants logic separated, we should probably depend on the real function
// if it's imported from the package. But since we are mocking the package... we need to decide.
// Tests currently fail if we don't return the real implementation or mock it.
// Let's use the actual implementation for now since it's pure logic, but we need to ensure the mock of @mydoo/tasklens includes it.

vi.mock('@mydoo/tasklens', async importOriginal => {
  const actual = await importOriginal<typeof import('@mydoo/tasklens')>();
  return {
    ...actual,
    useTunnel: (url: DocumentHandle) =>
      mockUseTunnel(url) as ReturnType<
        (typeof import('@mydoo/tasklens'))['useTunnel']
      >,
  };
});

// Mock Data Helpers
const createMockTask = (
  id: string,
  title: string,
  status: TaskStatus,
  importance: number,
): TunnelNode => ({
  childTaskIds: [],
  children: [],
  creditIncrement: 1,
  credits: 0,
  creditsTimestamp: 0,
  desiredCredits: 0,
  id: id as TaskID,
  importance,
  isContainer: false,
  isPending: status === TaskStatus.Pending,
  isReady: true,
  isSequential: false,
  normalizedImportance: importance,
  priority: importance * 10,
  priorityTimestamp: 0,
  schedule: {leadTime: 0, type: 'Once' as const},
  status,
  title,
});

describe('usePriorityList', () => {
  const mockDocUrl = 'automerge:123' as DocumentHandle;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('filters out completed and deleted tasks', () => {
    const mockState: Partial<TunnelState> = {
      tasks: {
        '1': createMockTask('1', 'Todo 1', TaskStatus.Pending, 0.5),
        '2': createMockTask('2', 'Done 1', TaskStatus.Done, 0.5),
        '3': createMockTask('3', 'Deleted 1', TaskStatus.Deleted, 0.5),
      } as Record<TaskID, TunnelNode>,
    } as unknown as TunnelState;

    mockUseTunnel.mockReturnValue({doc: mockState});

    const {result} = renderHook(() => usePriorityList(mockDocUrl));

    expect(result.current.tasks).toMatchObject([{id: '1'}]);
  });

  it('sorts tasks by priority (descending)', () => {
    const mockState: Partial<TunnelState> = {
      tasks: {
        '1': createMockTask('1', 'Low Priority', TaskStatus.Pending, 0.1),
        '2': createMockTask('2', 'High Priority', TaskStatus.Pending, 0.9),
        '3': createMockTask('3', 'Medium Priority', TaskStatus.Pending, 0.5),
      } as Record<TaskID, TunnelNode>,
    } as unknown as TunnelState;

    mockUseTunnel.mockReturnValue({doc: mockState});

    const {result} = renderHook(() => usePriorityList(mockDocUrl));

    expect(result.current.tasks).toMatchObject([
      {id: '2'}, // High
      {id: '3'}, // Medium
      {id: '1'}, // Low
    ]);
  });

  it('returns empty list when doc is loading', () => {
    mockUseTunnel.mockReturnValue({doc: undefined});
    const {result} = renderHook(() => usePriorityList(mockDocUrl));
    expect(result.current.isLoading).toBe(true);
    expect(result.current.tasks).toEqual([]);
  });
});

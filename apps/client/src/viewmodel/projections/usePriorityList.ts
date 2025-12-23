import {
  type DocumentHandle,
  type Task,
  TaskStatus,
  type TunnelState,
  useTunnel,
} from '@mydoo/tasklens';
import {useMemo} from 'react';

/**
 * Filter and sort logic for priority list.
 */
export function selectPriorityList(state: TunnelState): Task[] {
  return Object.values(state.tasks)
    .filter(
      t =>
        t.status === TaskStatus.Pending ||
        (t.status === TaskStatus.Done && !t.isAcknowledged),
    )
    .sort(
      (a, b) => (b.priority ?? b.importance) - (a.priority ?? a.importance),
    );
}

/**
 * Hook to retrieve a prioritized list of pending tasks.
 *
 * Uses pure domain projection logic from the tasklens package.
 */
export function usePriorityList(docUrl: DocumentHandle) {
  const {doc} = useTunnel(docUrl);

  const tasks = useMemo(() => {
    if (!doc) return [];
    return selectPriorityList(doc);
  }, [doc]);

  return {
    tasks,
    isLoading: doc === undefined,
  };
}

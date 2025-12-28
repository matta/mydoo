import {
  type DocumentHandle,
  getPrioritizedTasks,
  useTunnel,
} from '@mydoo/tasklens';
import {useMemo} from 'react';

/**
 * Hook to retrieve a prioritized list of pending tasks.
 *
 * Uses the Automerge document directly via useTunnel for real-time updates.
 *
 * TODO: Migrate to Redux-based useTodoList once the RepoContext singleton
 * issue in production builds is resolved (see ROLLING_CONTEXT.md).
 */
export function usePriorityList(docUrl: DocumentHandle) {
  const {doc} = useTunnel(docUrl);

  const tasks = useMemo(() => {
    if (!doc) return [];
    return getPrioritizedTasks(doc);
  }, [doc]);

  return {
    tasks,
    isLoading: !doc,
  };
}

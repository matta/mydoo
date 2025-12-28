import {
  type DocumentHandle,
  getPrioritizedTasks,
  useTunnel,
} from '@mydoo/tasklens';
import {useMemo} from 'react';

/**
 * Hook to retrieve a prioritized list of pending tasks.
 *
 * Uses direct Automerge access via useTunnel for reliability until Redux sync is verified.
 *
 * @param docUrl - The document URL.
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

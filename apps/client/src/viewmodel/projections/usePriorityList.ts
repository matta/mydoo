import {
  type DocumentHandle,
  selectPriorityList,
  useTunnel,
} from '@mydoo/tasklens';
import {useMemo} from 'react';

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

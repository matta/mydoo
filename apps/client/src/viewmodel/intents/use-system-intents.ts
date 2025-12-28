import {useTaskActions} from '@mydoo/tasklens';
import {useCallback, useMemo} from 'react';

export interface SystemIntents {
  refreshTaskList: () => void;
}

export function useSystemIntents(): SystemIntents {
  const {acknowledgeAllDoneTasks} = useTaskActions();

  const refreshTaskList = useCallback(() => {
    acknowledgeAllDoneTasks();
  }, [acknowledgeAllDoneTasks]);

  return useMemo(
    () => ({
      refreshTaskList,
    }),
    [refreshTaskList],
  );
}

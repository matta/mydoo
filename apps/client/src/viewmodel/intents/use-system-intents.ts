import type { AutomergeUrl } from "@automerge/automerge-repo";
import { useDocHandle } from "@automerge/automerge-repo-react-hooks";
import { TaskActions, wakeUpRoutineTasks } from "@mydoo/tasklens";
import type { TunnelState } from "@mydoo/tasklens/persistence";
import { useCallback, useMemo } from "react";
import { useAppDispatch } from "../../store";

export interface SystemIntents {
  refreshTaskList: () => void;
}

export function useSystemIntents(docUrl: AutomergeUrl): SystemIntents {
  const dispatch = useAppDispatch();
  const handle = useDocHandle<TunnelState>(docUrl);

  const refreshTaskList = useCallback(() => {
    if (handle) {
      wakeUpRoutineTasks(handle);
    }
    dispatch(TaskActions.acknowledgeAllDoneTasks());
  }, [dispatch, handle]);

  return useMemo(
    () => ({
      refreshTaskList,
    }),
    [refreshTaskList],
  );
}

import { useDocHandle } from "@automerge/automerge-repo-react-hooks";
import {
  useTaskActions,
  useTaskLensDocUrl,
  wakeUpRoutineTasks,
} from "@mydoo/tasklens";
import { useCallback, useMemo } from "react";

export interface SystemIntents {
  refreshTaskList: () => void;
}

export function useSystemIntents(): SystemIntents {
  const { acknowledgeAllDoneTasks } = useTaskActions();
  const docUrl = useTaskLensDocUrl();
  const handle = useDocHandle(docUrl);

  const refreshTaskList = useCallback(() => {
    if (handle) {
      wakeUpRoutineTasks(handle);
    }
    acknowledgeAllDoneTasks();
  }, [acknowledgeAllDoneTasks, handle]);

  return useMemo(
    () => ({
      refreshTaskList,
    }),
    [refreshTaskList],
  );
}

export * from "./domain/index";
export * from "./domain/initialization";
export * from "./domain/projections";
export * from "./domain/routine-tasks";
export * from "./domain/tree";
export * as TunnelOps from "./persistence/ops";
export { TunnelStore } from "./persistence/store";
export type { ThunkExtra } from "./redux/middleware";
export * from "./redux/middleware";

import {
  acknowledgeAllDoneTasks,
  createTask,
  deleteTask,
  moveTask,
  updateTask,
} from "./redux/thunks";

export const TaskActions = {
  createTask,
  updateTask,
  deleteTask,
  moveTask,
  acknowledgeAllDoneTasks,
};
export {
  createTaskLensStore,
  type TaskLensDispatch,
  type TaskLensState,
  taskLensStore,
} from "./store/index";
export * from "./store/selectors";
export { default as tasksReducer, syncDoc } from "./store/slices/tasks-slice";
export * from "./types/ui";

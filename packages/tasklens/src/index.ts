import {
  acknowledgeAllDoneTasks,
  createTask,
  deleteTask,
  moveTask,
  updateTask,
} from "./redux/thunks";

// --- Domain: Balance ---
export { calculateBalanceData, STARVING_THRESHOLD } from "./domain/balance";
export {
  type BalanceItemSimple,
  type CreditUpdate,
  distributeCredits,
} from "./domain/balance-distribution";

// --- Domain: Constants ---
export {
  CREDITS_HALF_LIFE_MILLIS,
  DEFAULT_TASK_IMPORTANCE,
  DEFAULT_TASK_LEAD_TIME_HOURS,
  DEFAULT_TASK_LEAD_TIME_MS,
} from "./domain/constants";
// --- Domain: Initialization ---
export {
  createTaskLensDoc,
  initializeTunnelState,
  isDocInitialized,
} from "./domain/initialization";
export { getPrioritizedTasks } from "./domain/priority";
export {
  getDescendantCountFromEntities,
  toComputedTask,
} from "./domain/projections";
export { wakeUpRoutineTasks } from "./domain/routine-tasks";
export { buildTunnelTree } from "./domain/tree";
export * as TunnelOps from "./persistence/ops";
export { TunnelStore } from "./persistence/store";
export {
  createTaskLensMiddleware,
  type TaskLensMiddlewareResult,
  type ThunkExtra,
} from "./redux/middleware";
export const TaskActions = {
  createTask,
  updateTask,
  deleteTask,
  moveTask,
  acknowledgeAllDoneTasks,
};
export {
  createTaskLensStore,
  getTaskLensReduxConfig,
  type TaskLensDispatch,
  type TaskLensReduxConfig,
  type TaskLensState,
  taskLensStore,
} from "./store/index";
export {
  selectBalanceData,
  selectLastProxyDoc,
  selectRootTaskIds,
  selectStoreReady,
  selectTaskById,
  selectTaskEntities,
  selectTodoList,
  selectTodoListIds,
} from "./store/selectors";
export { default as tasksReducer, syncDoc } from "./store/slices/tasks-slice";
export {
  type BalanceItemData,
  type ComputedTask,
  type CreateTaskOptions,
  DEFAULT_CREDIT_INCREMENT,
  type PlaceID,
  type RepeatConfig,
  type RepeatConfigFields,
  type Schedule,
  type ScheduleFields,
  type TaskCreateInput,
  type TaskCreateProps,
  type TaskFields,
  type TaskID,
  TaskStatus,
  type TaskUpdateInput,
  type TunnelNode,
  type ViewFilter,
} from "./types/ui";

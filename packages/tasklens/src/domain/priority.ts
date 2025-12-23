import type {Context, Task, TaskID, TunnelState, ViewFilter} from '../types';
import {getCurrentTimestamp} from '../utils/time';
import {pass1ContextualVisibility} from './pass1Visibility';
import {pass2ScheduleInheritance} from './pass2Schedule';
import {pass3DeviationFeedback} from './pass3Thermostat';
import {pass4WeightNormalization} from './pass4Weights';
import {pass5LeadTimeRamp} from './pass5Leadtime';
import {pass6FinalPriority} from './pass6Priority';
import {pass7ContainerVisibility} from './pass7Container';

export function recalculatePriorities(
  state: TunnelState,
  viewFilter: ViewFilter,
  context?: Context,
): void {
  const currentTime = context?.currentTime ?? getCurrentTimestamp();
  const tasks = Object.values(state.tasks);

  // Pass 1: Contextual Visibility
  pass1ContextualVisibility(state, viewFilter, currentTime);

  // Helpers
  const getTaskFromDoc = (
    docState: TunnelState,
    id: TaskID,
  ): Task | undefined => docState.tasks[id];

  const getChildrenFromDoc = (
    docState: TunnelState,
    parentId: TaskID | undefined,
  ) => Object.values(docState.tasks).filter(task => task.parentId === parentId);

  const getAncestorsFromDoc = (docState: TunnelState, id: TaskID) => {
    const ancestors: Task[] = [];
    let currentTask = getTaskFromDoc(docState, id);
    while (currentTask?.parentId !== undefined) {
      const parent = getTaskFromDoc(docState, currentTask.parentId);
      if (parent) {
        ancestors.unshift(parent);
        currentTask = parent;
      } else {
        break;
      }
    }
    return ancestors;
  };

  // Pass 2: Schedule Inheritance
  pass2ScheduleInheritance(tasks);

  // Pass 3: Deviation Feedback
  pass3DeviationFeedback(state, tasks, getTaskFromDoc, getChildrenFromDoc);

  // Pass 4: Weight Normalization
  pass4WeightNormalization(state, tasks, getChildrenFromDoc);

  // Pass 5: Lead Time Ramp
  pass5LeadTimeRamp(tasks, currentTime);

  // Pass 6: Final Priority
  pass6FinalPriority(state, tasks, getAncestorsFromDoc);

  // Pass 7: Container Visibility
  pass7ContainerVisibility(state, tasks, getChildrenFromDoc);
}

/**
 * Derives the "Projected State" for the View Layer.
 *
 * computes transient properties (priority, visibility) that are not stored in the DB.
 *
 * @param state The raw Automerge state (Read-Only).
 * @param viewFilter Filter criteria (e.g. 'Pending' is implicit for priority list, but viewFilter handles Places).
 * @returns Sorted, filtered list of tasks ready for display.
 */
export function getPrioritizedTasks(
  state: TunnelState,
  viewFilter: ViewFilter = {},
): Task[] {
  // 1. Shallow clone tasks to allow mutation of computed properties (priority, visibility).
  // We must clone ALL tasks because the algorithm traverses the graph (parents/children).
  const clonedTasks: Record<string, Task> = {};
  for (const [id, task] of Object.entries(state.tasks)) {
    clonedTasks[id] = {...task};
  }

  // 2. Create a specific Projected State for the algo to work on.
  // We use "as TunnelState" because we are mocking the structure with cloned tasks.
  const projectedState: TunnelState = {
    ...state,
    tasks: clonedTasks,
  };

  // 3. Run the full algorithm on the cloned state.
  recalculatePriorities(projectedState, viewFilter);

  // 4. Return the filtered list (Pending + Visible + Sorted).
  // Note: recalculatePriorities applies Pass 1 & 7 which set 'visibility'.
  // We also explicitly filter for Status here as 'getPrioritizedList' implies 'Active' tasks.
  // Uses logic similar to what projections.ts had, but relying on the computed props.
  return Object.values(projectedState.tasks)
    .filter(t => {
      // Must be Visible (Pass 1 place/context + Pass 7 container visibility)
      if (!t.visibility) return false;

      // Must be Pending OR (Done + Unacknowledged)
      // Usually Pass 1 handles "Done" visibility?
      // Let's check Pass 1. Pass 1 usually hides Done tasks unless specific view settings.
      // But explicit Status check provides safety.
      return (
        t.status === 'Pending' || (t.status === 'Done' && !t.isAcknowledged)
      );
    })
    .sort((a, b) => {
      // Sort by Priority (Pass 6) -> Importance -> Creation (implicit in ID?)
      const pA = a.priority ?? 0;
      const pB = b.priority ?? 0;
      if (pA !== pB) return pB - pA;
      return (b.importance ?? 0) - (a.importance ?? 0);
    });
}

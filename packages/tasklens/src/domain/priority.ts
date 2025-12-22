import {
  type Context,
  type Task,
  type TaskID,
  type TunnelState,
  type ViewFilter,
} from '../types';
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

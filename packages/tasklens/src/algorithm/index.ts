import * as Automerge from "@automerge/automerge";
import { TunnelStore } from "../../src/store";
import {
  Task,
  TaskID,
  ViewFilter,
  Context,
  TunnelState,
} from "../../src/types";
import { getCurrentTimestamp } from "../../src/utils/time";

import { pass1ContextualVisibility } from "./pass1_visibility";
import { pass2ScheduleInheritance } from "./pass2_schedule";
import { pass3DeviationFeedback } from "./pass3_thermostat";
import { pass4WeightNormalization } from "./pass4_weights";
import { pass5LeadTimeRamp } from "./pass5_leadtime";
import { pass6FinalPriority } from "./pass6_priority";
import { pass7ContainerVisibility } from "./pass7_container";

export function recalculateScores(
  store: TunnelStore,
  viewFilter: ViewFilter,
  context?: Context,
): void {
  const currentTime = context?.currentTime ?? getCurrentTimestamp();

  store.doc = Automerge.change(store.doc, "Recalculate scores", (doc) => {
    const tasks = Object.values(doc.tasks);

    // Pass 1: Contextual Visibility
    pass1ContextualVisibility(doc, tasks, viewFilter, currentTime);

    // Helpers
    const getTaskFromDoc = (
      docState: TunnelState,
      id: TaskID,
    ): Task | undefined => docState.tasks[id];

    const getChildrenFromDoc = (
      docState: TunnelState,
      parentId: TaskID | null,
    ) =>
      Object.values(docState.tasks).filter(
        (task) => task.parentId === parentId,
      );

    const getAncestorsFromDoc = (docState: TunnelState, id: TaskID) => {
      const ancestors: Task[] = [];
      let currentTask = getTaskFromDoc(docState, id);
      while (currentTask && currentTask.parentId !== null) {
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
    pass3DeviationFeedback(doc, tasks, getTaskFromDoc, getChildrenFromDoc);

    // Pass 4: Weight Normalization
    pass4WeightNormalization(doc, tasks, getTaskFromDoc, getChildrenFromDoc);

    // Pass 5: Lead Time Ramp
    pass5LeadTimeRamp(doc, tasks, currentTime);

    // Pass 6: Final Priority
    pass6FinalPriority(doc, tasks, getAncestorsFromDoc);

    // Pass 7: Container Visibility
    pass7ContainerVisibility(doc, tasks, getChildrenFromDoc);
  });
}

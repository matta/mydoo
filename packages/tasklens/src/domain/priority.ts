import type {
  ComputedTask,
  Context,
  EnrichedTask,
  PriorityOptions,
  TaskID,
  TunnelState,
  ViewFilter,
} from '../types';
import {getCurrentTimestamp} from '../utils/time';
import {pass1ContextualVisibility} from './pass1-visibility';
import {pass2ScheduleInheritance} from './pass2-schedule';
import {pass3DeviationFeedback} from './pass3-thermostat';
import {pass4WeightNormalization} from './pass4-weights';
import {pass5LeadTimeRamp} from './pass5-leadtime';
import {pass6FinalPriority} from './pass6-priority';
import {pass7ContainerVisibility} from './pass7-container';

/**
 * Runs the prioritization algorithm on the mutable EnrichedTask objects.
 * (Stage 2 of the Pipeline)
 */
export function recalculatePriorities(
  state: TunnelState,
  enrichedTasks: EnrichedTask[],
  viewFilter: ViewFilter,
  context?: Context,
): void {
  const currentTime = context?.currentTime ?? getCurrentTimestamp();

  // Create Lookup Maps for Helpers
  // Performance: O(N) to build map, but enables O(1) lookups in passes.
  const taskMap = new Map<TaskID, EnrichedTask>();
  for (const t of enrichedTasks) {
    taskMap.set(t.id, t);
  }

  // Helpers
  const getTaskFromMap = (id: TaskID) => taskMap.get(id);

  // Note: Filter is O(N).
  // We can optimize by building a parent index if performance suffers for N > 2000.
  const getChildrenFromMap = (parentId: TaskID | undefined) =>
    enrichedTasks.filter(task => task.parentId === parentId);

  const getAncestorsFromMap = (id: TaskID) => {
    const ancestors: EnrichedTask[] = [];
    let currentTask = getTaskFromMap(id);
    while (currentTask?.parentId !== undefined) {
      const parent = getTaskFromMap(currentTask.parentId);
      if (parent) {
        ancestors.unshift(parent);
        currentTask = parent;
      } else {
        break;
      }
    }
    return ancestors;
  };

  pass1ContextualVisibility(state, enrichedTasks, viewFilter, currentTime);
  pass2ScheduleInheritance(enrichedTasks);
  pass3DeviationFeedback(
    state,
    enrichedTasks,
    getTaskFromMap,
    getChildrenFromMap,
  );
  pass4WeightNormalization(state, enrichedTasks, getChildrenFromMap);
  pass5LeadTimeRamp(enrichedTasks, currentTime);
  pass6FinalPriority(state, enrichedTasks, getAncestorsFromMap);
  pass7ContainerVisibility(state, enrichedTasks, getChildrenFromMap);
}

/**
 * Derives the "Projected State" for the View Layer.
 *
 * Implements the 3-Stage Pipeline:
 * 1. Hydrate (Persisted -> Enriched)
 * 2. Process (Run Algorithm)
 * 3. Sanitize (Enriched -> Computed)
 *
 * @param state The raw Automerge state. MUST BE A PLAIN OBJECT (POJO), NOT AN AUTOMERGE PROXY.
 *              Passes relying on spread syntax (...) or Object.keys will fail on Proxies.
 * @param viewFilter Filter criteria.
 * @returns Sorted, filtered list of tasks for the View.
 */
export function getPrioritizedTasks(
  state: TunnelState,
  viewFilter: ViewFilter = {},
  options: PriorityOptions = {},
): ComputedTask[] {
  // --- Stage 1: Hydrate & Initialize ---
  // Clone Persisted Tasks into Mutable Enriched Tasks
  const enrichedTasks: EnrichedTask[] = Object.values(state.tasks).map(
    persisted => {
      const isContainer = persisted.childTaskIds.length > 0;
      const isPending = persisted.status === 'Pending';

      return {
        ...persisted,
        effectiveCredits: 0,
        feedbackFactor: 1.0,
        leadTimeFactor: 0,
        normalizedImportance: 0,
        priority: 0,
        visibility: true,
        isContainer,
        isPending,
        isReady: false,
      };
    },
  );

  // --- Stage 2: Process ---
  recalculatePriorities(state, enrichedTasks, viewFilter);

  // --- Stage 3: Sanitize & Sort ---

  // Sort by Priority (Desc) -> Importance -> ID
  enrichedTasks.sort((a, b) => {
    const pA = a.priority ?? 0;
    const pB = b.priority ?? 0;
    if (pA !== pB) return pB - pA;
    return (b.importance ?? 0) - (a.importance ?? 0);
  });

  return enrichedTasks
    .filter(t => {
      if (!options.includeHidden && !t.visibility) return false;

      if (!options.includeDone) {
        return (
          t.status === 'Pending' || (t.status === 'Done' && !t.isAcknowledged)
        );
      }
      return true;
    })
    .map(enriched => {
      const isReady = enriched.isPending && enriched.leadTimeFactor > 0;

      // Note: We return the enriched object cast as ComputedTask for performance.
      // Extra properties (priority, visibility) are present at runtime but hidden by types.
      const computed: ComputedTask = {
        ...enriched,
        isContainer: enriched.isContainer,
        isPending: enriched.isPending,
        isReady,
      };

      return computed;
    });
}

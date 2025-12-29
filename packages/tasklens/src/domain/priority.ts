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
import {calculateFeedbackFactors} from './feedback';
import {calculateLeadTimeFactor} from './readiness';
import {calculateContextualVisibility} from './visibility';

/**
 * Half-life for credit decay calculation (7 days in milliseconds).
 * Credits decay exponentially with this half-life to prioritize recent work.
 */
const CREDITS_HALF_LIFE_MILLIS = 7 * 24 * 60 * 60 * 1000;

/**
 * Builds O(1) lookup indexes for tasks.
 * @returns taskMap (ID -> Task) and childrenIndex (ParentID -> Children[]).
 *          The 'undefined' key in childrenIndex holds root tasks.
 */
function buildIndexes(enrichedTasks: EnrichedTask[]): {
  taskMap: Map<TaskID, EnrichedTask>;
  childrenIndex: Map<TaskID | undefined, EnrichedTask[]>;
} {
  const taskMap = new Map<TaskID, EnrichedTask>();
  const childrenIndex = new Map<TaskID | undefined, EnrichedTask[]>();

  for (const task of enrichedTasks) {
    taskMap.set(task.id, task);

    const siblings = childrenIndex.get(task.parentId) ?? [];
    siblings.push(task);
    childrenIndex.set(task.parentId, siblings);
  }

  return {taskMap, childrenIndex};
}

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

  // --- Phase 0: Build Indexes ---
  const {childrenIndex} = buildIndexes(enrichedTasks);

  // --- Phase 1: Linear Local Computation ---
  calculateContextualVisibility(state, enrichedTasks, viewFilter, currentTime);

  // Compute LeadTimeFactor for all tasks
  for (const task of enrichedTasks) {
    task.leadTimeFactor = calculateLeadTimeFactor(task.schedule, currentTime);
  }

  // Compute EffectiveCredits for all tasks (required by feedback)
  for (const task of enrichedTasks) {
    const timeDelta = currentTime - task.creditsTimestamp;
    task.effectiveCredits =
      task.credits * 0.5 ** (timeDelta / CREDITS_HALF_LIFE_MILLIS);
  }

  // Compute Feedback Factors (affects root tasks only)
  calculateFeedbackFactors(enrichedTasks);

  // --- Phase 2: Unified DFS Traversal ---
  const roots = childrenIndex.get(undefined) ?? [];
  for (const root of roots) {
    root.normalizedImportance = 1.0;
    evaluateTaskRecursive(root, undefined, childrenIndex, currentTime);
  }
}

/**
 * Depth-First Search with Pre-Order (Top-Down) and Post-Order (Bottom-Up) logic.
 * @returns true if this task or any descendant is visible (for container pruning).
 */
function evaluateTaskRecursive(
  task: EnrichedTask,
  rootTask: EnrichedTask | undefined,
  childrenIndex: Map<TaskID | undefined, EnrichedTask[]>,
  currentTime: number,
): boolean {
  const children = childrenIndex.get(task.id) ?? [];
  const effectiveRoot = rootTask ?? task;

  // --- Pre-Order: Propagate from Parent to Children ---
  const siblingImportanceSum = children.reduce(
    (sum, c) => sum + c.importance,
    0,
  );

  let hasActiveChild = false; // Track sequential state

  for (const child of children) {
    // Inherit Schedule
    if (child.schedule.type === 'Once' && task.schedule.dueDate !== undefined) {
      child.schedule.dueDate = task.schedule.dueDate;
      child.schedule.leadTime = task.schedule.leadTime;
    }

    // Propagate Weights & Sequential Logic
    if (task.isSequential) {
      if (child.status === 'Pending') {
        if (hasActiveChild) {
          // Blocked: Subsequent pending tasks get no weight and no lead time
          child.normalizedImportance = 0;
          child.leadTimeFactor = 0;
          continue; // Skip further processing for this child (including leadTimeFactor below)
        }
        // Active: The first pending task becomes the active one
        hasActiveChild = true;

        // Active task gets full parent importance (Focus mode)
        child.normalizedImportance = task.normalizedImportance ?? 0;
      } else {
        // Done tasks in sequential lists also retain full importance
        child.normalizedImportance = task.normalizedImportance ?? 0;
      }
    } else {
      // Standard Proportional Distribution
      if (siblingImportanceSum === 0) {
        child.normalizedImportance =
          (task.normalizedImportance ?? 0) / children.length;
      } else {
        child.normalizedImportance =
          (child.importance / siblingImportanceSum) *
          (task.normalizedImportance ?? 0);
      }
    }

    // Compute LeadTimeFactor (Must occur after Schedule Inheritance)
    // (Note: Blocked sequential tasks already set to 0 and skipped via continue above)
    child.leadTimeFactor = calculateLeadTimeFactor(child.schedule, currentTime);
  }

  // --- Recurse ---
  let hasVisibleDescendant = false;
  for (const child of children) {
    hasVisibleDescendant =
      evaluateTaskRecursive(child, effectiveRoot, childrenIndex, currentTime) ||
      hasVisibleDescendant;
  }

  if (children.length > 0 && hasVisibleDescendant) {
    // --- Post-Order: Aggregate from Children ---
    // Container Visibility
    task.visibility = false;
    task.priority = 0;
  } else {
    // Compute Final Priority
    const visibilityFactor = task.visibility ? 1.0 : 0.0;
    const feedbackFactor = effectiveRoot.feedbackFactor ?? 1.0;
    task.priority =
      visibilityFactor *
      (task.normalizedImportance ?? 0) *
      feedbackFactor *
      (task.leadTimeFactor ?? 0);
  }

  return task.visibility || hasVisibleDescendant;
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

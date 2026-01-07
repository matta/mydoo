import {
  type ComputedTask,
  type Context,
  DEFAULT_CREDIT_INCREMENT,
  type EnrichedTask,
  type PriorityOptions,
  type TaskID,
  type TunnelState,
  type ViewFilter,
} from "../types";
import { getCurrentTimestamp, getIntervalMs } from "../utils/time";
import { CREDITS_HALF_LIFE_MILLIS } from "./constants";
import { calculateFeedbackFactors } from "./feedback";
import { calculateLeadTimeFactor } from "./readiness";
import { calculateContextualVisibility } from "./visibility";

/**
 * Builds O(1) lookup indexes for tasks and sorts children based on explicit order logic.
 * @returns taskMap (ID -> Task) and childrenIndex (ParentID -> Children[]).
 *          The 'undefined' key in childrenIndex holds root tasks.
 */
function buildIndexes(
  state: TunnelState,
  enrichedTasks: EnrichedTask[],
): {
  taskMap: Map<TaskID, EnrichedTask>;
  childrenIndex: Map<TaskID | undefined, EnrichedTask[]>;
} {
  const taskMap = new Map<TaskID, EnrichedTask>();
  const childrenIndex = new Map<TaskID | undefined, EnrichedTask[]>();

  for (const task of enrichedTasks) {
    taskMap.set(task.id, task);

    const parentId = task.parentId;
    const siblings = childrenIndex.get(parentId) ?? [];
    siblings.push(task);
    childrenIndex.set(parentId, siblings);
  }

  // --- Sort Children based on preserved order ---
  // 1. Sort Root Tasks
  const rootTasks = childrenIndex.get(undefined);
  if (rootTasks) {
    const rootOrderMap = new Map(state.rootTaskIds.map((id, i) => [id, i]));
    rootTasks.sort((a, b) => {
      const idxA = rootOrderMap.get(a.id) ?? Number.MAX_SAFE_INTEGER;
      const idxB = rootOrderMap.get(b.id) ?? Number.MAX_SAFE_INTEGER;
      return idxA - idxB;
    });
  }

  // 2. Sort Children of each parent
  for (const [parentId, children] of childrenIndex) {
    if (parentId === undefined) continue;
    const parent = taskMap.get(parentId);
    if (parent) {
      const childOrderMap = new Map(
        parent.childTaskIds.map((id, i) => [id, i]),
      );
      children.sort((a, b) => {
        const idxA = childOrderMap.get(a.id) ?? Number.MAX_SAFE_INTEGER;
        const idxB = childOrderMap.get(b.id) ?? Number.MAX_SAFE_INTEGER;
        return idxA - idxB;
      });
    }
  }

  return { taskMap, childrenIndex };
}

/**
 * Assigns outlineIndex via DFS traversal.
 */
function assignOutlineIndexes(
  childrenIndex: Map<TaskID | undefined, EnrichedTask[]>,
): void {
  let currentIndex = 0;

  function traverse(parentId: TaskID | undefined) {
    const children = childrenIndex.get(parentId) ?? [];
    for (const child of children) {
      child.outlineIndex = currentIndex++;
      traverse(child.id);
    }
  }

  traverse(undefined);
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

  // --- Phase 0: Build Indexes & Outline Order ---
  const { childrenIndex } = buildIndexes(state, enrichedTasks);
  assignOutlineIndexes(childrenIndex);

  // --- Phase 1: Linear Local Computation ---
  calculateContextualVisibility(state, enrichedTasks, viewFilter, currentTime);

  // Compute LeadTimeFactor for all tasks
  for (const task of enrichedTasks) {
    task.leadTimeFactor = calculateLeadTimeFactor(task.schedule, currentTime);

    // Safety check for NaN
    if (Number.isNaN(task.leadTimeFactor)) {
      task.leadTimeFactor = 0;
    }
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

  // Distribute importance among roots
  for (const root of roots) {
    // Roots start with their raw importance. They are not normalized against each other
    // to preserve absolute scoring for independent trees (which existing tests rely on),
    // while still allowing higher-importance roots to dominate lower-importance ones.
    root.normalizedImportance = root.importance;

    // Root Defaults: If no creditIncrement is explicitly set, use the default.
    root.creditIncrement ??= DEFAULT_CREDIT_INCREMENT;

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
  processChildren(task, children, currentTime);

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
    const leadTimeFactor = task.leadTimeFactor ?? 0;
    const normalizedImportance = task.normalizedImportance ?? 0;

    // Protection against NaNs propagating
    const safeLeadTime = Number.isNaN(leadTimeFactor) ? 0 : leadTimeFactor;
    const safeImportance = Number.isNaN(normalizedImportance)
      ? 0
      : normalizedImportance;

    task.priority =
      visibilityFactor * safeImportance * feedbackFactor * safeLeadTime;
  }

  return task.visibility || hasVisibleDescendant;
}

/**
 * Processes children in the pre-order phase:
 * 1. Inherits properties (Schedule, Credits).
 * 2. Propagates weights (Normalized Importance).
 * 3. Applies sequential blocking logic.
 * 4. Calculates LeadTimeFactor.
 */
function processChildren(
  parent: EnrichedTask,
  children: EnrichedTask[],
  currentTime: number,
): void {
  const siblingImportanceSum = children.reduce(
    (sum, c) => sum + c.importance,
    0,
  );

  let hasActiveChild = false; // Track sequential state

  for (const child of children) {
    // Inherit Schedule
    if (
      child.schedule.type === "Once" &&
      parent.schedule.dueDate !== undefined &&
      child.schedule.dueDate === undefined
    ) {
      child.schedule.dueDate = parent.schedule.dueDate;
      child.schedule.leadTime = parent.schedule.leadTime;
    }

    // Propagate Weights & Sequential Logic
    if (parent.isSequential) {
      if (child.status === "Pending") {
        if (hasActiveChild) {
          // Blocked: Subsequent pending tasks get no weight and no lead time
          child.normalizedImportance = 0;
          child.leadTimeFactor = 0;
          continue; // Skip further processing for this child (including leadTimeFactor below)
        }
        // Active: The first pending task becomes the active one
        hasActiveChild = true;

        // Active task gets full parent importance (Focus mode)
        child.normalizedImportance = parent.normalizedImportance ?? 0;
      } else {
        // Done tasks in sequential lists also retain full importance
        child.normalizedImportance = parent.normalizedImportance ?? 0;
      }
    } else {
      // Standard Proportional Distribution
      if (siblingImportanceSum === 0) {
        child.normalizedImportance =
          (parent.normalizedImportance ?? 0) / children.length;
      } else {
        child.normalizedImportance =
          (child.importance / siblingImportanceSum) *
          (parent.normalizedImportance ?? 0);
      }
    }

    // Compute LeadTimeFactor (Must occur after Schedule Inheritance)
    // (Note: Blocked sequential tasks already set to 0 and skipped via continue above)
    child.leadTimeFactor = calculateLeadTimeFactor(child.schedule, currentTime);

    // Safety check for NaN again (in case inheritance caused issues)
    if (Number.isNaN(child.leadTimeFactor)) {
      child.leadTimeFactor = 0;
    }
  }
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
const MIN_PRIORITY = 0.001;
const PRIORITY_EPSILON = 0.000001;

export function getPrioritizedTasks(
  state: TunnelState,
  viewFilter: ViewFilter = {},
  options: PriorityOptions = {},
): ComputedTask[] {
  // --- Stage 1: Hydrate & Initialize ---
  // Clone Persisted Tasks into Mutable Enriched Tasks
  const enrichedTasks: EnrichedTask[] = Object.values(state.tasks).map(
    (persisted) => {
      const isContainer = persisted.childTaskIds.length > 0;
      const isPending = persisted.status === "Pending";

      return {
        ...persisted,
        schedule: { ...persisted.schedule }, // Deep clone to allow mutation without side effects
        repeatConfig: persisted.repeatConfig
          ? { ...persisted.repeatConfig }
          : undefined, // Explicit clone to ensure availability
        effectiveCredits: 0,
        feedbackFactor: 1.0,
        leadTimeFactor: 0,
        normalizedImportance: 0,
        priority: 0,
        visibility: true,
        isContainer,
        isPending,
        isReady: false,
        outlineIndex: 0,
      };
    },
  );

  // --- Stage 2: Process ---

  // Phase 0: Date Resolution (Pass 1 in Algorithm)
  // Calculate effective due dates for Routinely tasks before any inheritance happens.
  for (const task of enrichedTasks) {
    if (task.schedule.type === "Routinely") {
      const { lastDone } = task.schedule;
      const repeatConfig = task.repeatConfig;
      // Spec: DueDate = LastDone + Period (Interval).
      if (lastDone && repeatConfig) {
        const intervalMs = getIntervalMs(
          repeatConfig.frequency,
          repeatConfig.interval,
        );
        task.schedule.dueDate = lastDone + intervalMs;
      }
    }
    // DueDate type tasks already have explicit dueDate in schedule.dueDate (from persistence).
  }

  recalculatePriorities(state, enrichedTasks, viewFilter, options.context);

  // --- Stage 3: Sanitize & Sort ---

  // Sort by Priority (Desc) -> Importance (Desc) -> Outline Order (Asc)
  enrichedTasks.sort((a, b) => {
    const pA = a.priority ?? 0;
    const pB = b.priority ?? 0;
    if (Math.abs(pA - pB) > PRIORITY_EPSILON) return pB - pA;

    const impA = a.importance ?? 0;
    const impB = b.importance ?? 0;
    if (impA !== impB) return impB - impA;

    return (a.outlineIndex ?? 0) - (b.outlineIndex ?? 0);
  });

  return enrichedTasks
    .filter((t) => {
      // 1. Visibility Check (Explicit & Hierarchy)
      if (!options.includeHidden && !t.visibility) return false;

      // 2. Status Check
      if (options.mode === "plan-outline") {
        // In "Plan Outline" mode (Inventory), we show everything that passed visibility
        // (including Done tasks).
      } else {
        // Default "Do List" mode:
        // Hide "Done" tasks UNLESS they are waiting for acknowledgement.
        if (t.status === "Done" && t.isAcknowledged) {
          return false;
        }
      }

      // 3. Priority Threshold (Focus Mode)
      // Hidden tasks or explicit dump requests bypass this.
      if (!options.includeHidden) {
        const p = t.priority ?? 0;
        if (p <= MIN_PRIORITY) {
          return false;
        }
      }

      return true;
    })
    .map((enriched) => {
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

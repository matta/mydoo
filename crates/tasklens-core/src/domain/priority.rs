use crate::domain::constants::{
    CREDITS_HALF_LIFE_MILLIS, DEFAULT_CREDIT_INCREMENT, FEEDBACK_DEVIATION_RATIO_CAP,
    FEEDBACK_EPSILON, FEEDBACK_SENSITIVITY, MIN_PRIORITY,
};
use crate::domain::feedback::calculate_feedback_factors;
use crate::domain::readiness::calculate_lead_time_factor;
use crate::domain::visibility::{calculate_contextual_visibility, resolve_contextual_visibility};
use crate::types::{
    ComputedTask, Context, EnrichedTask, FeedbackTrace, ImportanceTrace, LeadTimeStage,
    LeadTimeTrace, PersistedTask, PriorityMode, PriorityOptions, ScheduleSource, ScheduleType,
    ScoreFactors, ScoreTrace, TaskID, TaskStatus, TunnelState, ViewFilter, VisibilityTrace,
};
use crate::utils::time::{get_current_timestamp, get_interval_ms};

use std::collections::HashMap;

type TaskLookup = HashMap<TaskID, usize>;
type ChildrenLookup = HashMap<Option<TaskID>, Vec<usize>>;

const PRIORITY_EPSILON: f64 = 0.000001;

/// Computed priority context for trace and list rendering.
struct PriorityContext {
    /// Enriched tasks with computed priority fields.
    tasks: Vec<EnrichedTask>,
    /// Lookup from task id to task index.
    task_lookup: TaskLookup,
    /// Parent -> children index for traversal.
    children_index: ChildrenLookup,
    /// Timestamp used for the calculation.
    current_time: i64,
}

/// Builds lookup indexes for tasks and sorts children based on explicit order logic.
fn build_indexes(
    state: &TunnelState,
    enriched_tasks: &[EnrichedTask],
) -> (TaskLookup, ChildrenLookup) {
    let mut task_map = HashMap::new();
    let mut children_index = HashMap::new();

    for (i, task) in enriched_tasks.iter().enumerate() {
        task_map.insert(task.id.clone(), i);

        let parent_id = task.parent_id.clone();
        children_index
            .entry(parent_id)
            .or_insert_with(Vec::new)
            .push(i);
    }

    // --- Sort Children based on preserved order ---
    // 1. Sort Root Tasks
    if let Some(root_indices) = children_index.get_mut(&None) {
        let root_order_map: HashMap<TaskID, usize> = state
            .root_task_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        root_indices.sort_by_key(|&i| {
            root_order_map
                .get(&enriched_tasks[i].id)
                .cloned()
                .unwrap_or(usize::MAX)
        });
    }

    // 2. Sort Children of each parent
    let parent_ids: Vec<Option<TaskID>> = children_index.keys().cloned().collect();
    for parent_id in parent_ids {
        let parent_id = match parent_id {
            Some(id) => id,
            None => continue,
        };

        if let Some(child_indices) = children_index.get_mut(&Some(parent_id.clone())) {
            let parent_idx = task_map[&parent_id];
            let parent = &enriched_tasks[parent_idx];

            let child_order_map: HashMap<TaskID, usize> = parent
                .child_task_ids
                .iter()
                .enumerate()
                .map(|(i, id)| (id.clone(), i))
                .collect();

            child_indices.sort_by_key(|&i| {
                child_order_map
                    .get(&enriched_tasks[i].id)
                    .cloned()
                    .unwrap_or(usize::MAX)
            });
        }
    }

    (task_map, children_index)
}

/// Assigns outline index via DFS traversal.
fn assign_outline_indexes(enriched_tasks: &mut [EnrichedTask], children_index: &ChildrenLookup) {
    let mut current_index = 0.0;
    traverse_assign(None, enriched_tasks, children_index, &mut current_index);
}

fn traverse_assign(
    parent_id: Option<TaskID>,
    enriched_tasks: &mut [EnrichedTask],
    children_index: &ChildrenLookup,
    current_index: &mut f64,
) {
    if let Some(child_indices) = children_index.get(&parent_id) {
        for &idx in child_indices {
            enriched_tasks[idx].outline_index = *current_index;
            *current_index += 1.0;
            traverse_assign(
                Some(enriched_tasks[idx].id.clone()),
                enriched_tasks,
                children_index,
                current_index,
            );
        }
    }
}

/// Aggregates decayed credits from descendants into each task's effective_credits.
fn aggregate_effective_credits(
    task_idx: usize,
    enriched_tasks: &mut [EnrichedTask],
    children_index: &ChildrenLookup,
) -> f64 {
    let child_indices = children_index
        .get(&Some(enriched_tasks[task_idx].id.clone()))
        .cloned()
        .unwrap_or_default();

    let children_sum: f64 = child_indices
        .iter()
        .map(|&idx| aggregate_effective_credits(idx, enriched_tasks, children_index))
        .sum();

    let total = enriched_tasks[task_idx].effective_credits + children_sum;
    enriched_tasks[task_idx].effective_credits = total;
    total
}

/// Hydrates a persisted task into an enriched task object.
fn hydrate_task(persisted: &PersistedTask) -> EnrichedTask {
    let is_container = !persisted.child_task_ids.is_empty();
    let is_pending = persisted.status == TaskStatus::Pending
        || matches!(
            persisted.schedule.schedule_type,
            ScheduleType::Routinely | ScheduleType::Calendar
        );

    let effective_due_date = match persisted.schedule.schedule_type {
        ScheduleType::Routinely => {
            if let (Some(last_done), Some(config)) =
                (persisted.schedule.last_done, &persisted.repeat_config)
            {
                Some(last_done + get_interval_ms(config.frequency, config.interval))
            } else {
                persisted.schedule.due_date
            }
        }
        ScheduleType::Once | ScheduleType::DueDate | ScheduleType::Calendar => {
            persisted.schedule.due_date
        }
    };

    let effective_lead_time = Some(persisted.schedule.lead_time);

    EnrichedTask {
        id: persisted.id.clone(),
        title: persisted.title.clone(),
        notes: persisted.notes.clone(),
        parent_id: persisted.parent_id.clone(),
        child_task_ids: persisted.child_task_ids.clone(),
        place_id: persisted.place_id.clone(),
        status: persisted.status,
        importance: persisted.importance,
        credit_increment: persisted
            .credit_increment
            .or(Some(DEFAULT_CREDIT_INCREMENT)),
        credits: persisted.credits,
        desired_credits: persisted.desired_credits,
        credits_timestamp: persisted.credits_timestamp,
        priority_timestamp: persisted.priority_timestamp,
        schedule: persisted.schedule.clone(),
        repeat_config: persisted.repeat_config.clone(),
        is_sequential: persisted.is_sequential,
        is_acknowledged: persisted.is_acknowledged,
        last_completed_at: persisted.last_completed_at,

        effective_credits: 0.0,
        feedback_factor: 1.0,
        lead_time_factor: 0.0,
        normalized_importance: 0.0,
        priority: 0.0,
        visibility: true,
        outline_index: 0.0,
        is_container,
        is_pending,
        is_ready: false,

        effective_due_date,
        effective_lead_time,
        effective_schedule_source: if persisted.schedule.due_date.is_some()
            || (persisted.schedule.schedule_type == ScheduleType::Routinely
                && persisted.schedule.last_done.is_some())
        {
            Some(ScheduleSource::Myself)
        } else {
            None
        },
    }
}

/// Runs the prioritization algorithm on the mutable EnrichedTask objects.
pub fn recalculate_priorities(
    state: &TunnelState,
    enriched_tasks: &mut [EnrichedTask],
    view_filter: &ViewFilter,
    context: Option<&Context>,
) {
    let current_time = context
        .map(|c| c.current_time)
        .unwrap_or_else(get_current_timestamp);
    let (_, children_index) = build_indexes(state, enriched_tasks);
    recalculate_priorities_with_time(
        state,
        enriched_tasks,
        view_filter,
        current_time,
        &children_index,
    );
}

/// Runs the prioritization algorithm using an explicit timestamp.
fn recalculate_priorities_with_time(
    state: &TunnelState,
    enriched_tasks: &mut [EnrichedTask],
    view_filter: &ViewFilter,
    current_time: i64,
    children_index: &ChildrenLookup,
) {
    // --- Phase 0: Outline Order ---
    assign_outline_indexes(enriched_tasks, children_index);
    let root_indices = children_index.get(&None).cloned().unwrap_or_default();

    // --- Phase 1: Linear Local Computation ---
    calculate_contextual_visibility(state, enriched_tasks, view_filter, current_time);

    // Initialize Effective Schedule (already done in hydrate_task for self-defined dates)
    // and compute local factors
    for task in enriched_tasks.iter_mut() {
        task.lead_time_factor = calculate_lead_time_factor(
            task.effective_due_date,
            task.effective_lead_time.unwrap_or(task.schedule.lead_time),
            current_time,
        );

        if task.lead_time_factor.is_nan() {
            task.lead_time_factor = 0.0;
        }

        let time_delta = current_time - task.credits_timestamp;
        task.effective_credits =
            task.credits * 0.5f64.powf(time_delta as f64 / CREDITS_HALF_LIFE_MILLIS);
    }

    for &idx in &root_indices {
        aggregate_effective_credits(idx, enriched_tasks, children_index);
    }

    calculate_feedback_factors(enriched_tasks);

    // --- Phase 2: Unified DFS Traversal ---
    for &idx in &root_indices {
        // Roots start with their raw importance
        enriched_tasks[idx].normalized_importance = enriched_tasks[idx].importance;

        // Root Defaults
        if enriched_tasks[idx].credit_increment.is_none() {
            enriched_tasks[idx].credit_increment = Some(DEFAULT_CREDIT_INCREMENT);
        }

        evaluate_task_recursive(idx, None, enriched_tasks, children_index, current_time);
    }
}

/// Builds the enriched task context used for scoring and tracing.
fn build_priority_context(
    state: &TunnelState,
    view_filter: &ViewFilter,
    options: &PriorityOptions,
) -> PriorityContext {
    let mut tasks: Vec<EnrichedTask> = state.tasks.values().map(hydrate_task).collect();
    let current_time = options
        .context
        .as_ref()
        .map(|c| c.current_time)
        .unwrap_or_else(get_current_timestamp);

    let (task_lookup, children_index) = build_indexes(state, &tasks);
    recalculate_priorities_with_time(
        state,
        &mut tasks,
        view_filter,
        current_time,
        &children_index,
    );

    PriorityContext {
        tasks,
        task_lookup,
        children_index,
        current_time,
    }
}

/// Returns (is_visible, has_pending_descendants)
fn evaluate_task_recursive(
    task_idx: usize,
    root_idx: Option<usize>,
    enriched_tasks: &mut [EnrichedTask],
    children_index: &ChildrenLookup,
    current_time: i64,
) -> (bool, bool) {
    let child_indices = children_index
        .get(&Some(enriched_tasks[task_idx].id.clone()))
        .cloned()
        .unwrap_or_default();

    let effective_root_idx = root_idx.unwrap_or(task_idx);

    // --- Pre-Order: Propagate from Parent to Children ---
    process_children(task_idx, &child_indices, enriched_tasks, current_time);

    // --- Recurse ---
    let mut has_pending_descendant = false;

    for &child_idx in &child_indices {
        let (_, child_pending_subtree) = evaluate_task_recursive(
            child_idx,
            Some(effective_root_idx),
            enriched_tasks,
            children_index,
            current_time,
        );

        let child_is_pending = enriched_tasks[child_idx].is_pending;
        has_pending_descendant |= child_pending_subtree || child_is_pending;
    }

    let feedback_factor = enriched_tasks[effective_root_idx].feedback_factor;
    let task = &mut enriched_tasks[task_idx];

    // Pass 7: Container Visibility (Delegation Logic)
    // 1. If has pending descendants -> Hidden (Delegated to children)
    // 2. If NO pending descendants -> Treated as Leaf (Local Visibility)
    if has_pending_descendant {
        task.visibility = false;
        task.priority = 0.0;
    } else {
        // Leaf Node Logic (or Empty/Done Container)
        // Compute Final Priority
        let visibility_factor = if task.visibility { 1.0 } else { 0.0 };

        let normalized_importance = task.normalized_importance;
        let lead_time_factor = task.lead_time_factor;

        let safe_lead_time = if lead_time_factor.is_nan() {
            0.0
        } else {
            lead_time_factor
        };
        let safe_importance = if normalized_importance.is_nan() {
            0.0
        } else {
            normalized_importance
        };

        task.priority = visibility_factor * safe_importance * feedback_factor * safe_lead_time;
    }

    (task.visibility, has_pending_descendant)
}

fn process_children(
    parent_idx: usize,
    child_indices: &[usize],
    enriched_tasks: &mut [EnrichedTask],
    current_time: i64,
) {
    if child_indices.is_empty() {
        return;
    }

    let sibling_importance_sum: f64 = child_indices
        .iter()
        .filter(|&i| enriched_tasks[*i].is_pending)
        .map(|&i| enriched_tasks[i].importance)
        .sum();

    let is_sequential = enriched_tasks[parent_idx].is_sequential;
    let parent_importance = enriched_tasks[parent_idx].normalized_importance;
    let parent_due_date = enriched_tasks[parent_idx].effective_due_date;
    let parent_lead_time = enriched_tasks[parent_idx].effective_lead_time;

    let mut has_active_child = false;

    for &child_idx in child_indices {
        // Inherit Schedule
        if enriched_tasks[child_idx].effective_due_date.is_none() && parent_due_date.is_some() {
            enriched_tasks[child_idx].effective_due_date = parent_due_date;
            enriched_tasks[child_idx].effective_lead_time = parent_lead_time;
            enriched_tasks[child_idx].effective_schedule_source = Some(ScheduleSource::Ancestor);
        }

        // Propagate Weights & Sequential Logic
        if is_sequential {
            if enriched_tasks[child_idx].status == TaskStatus::Pending {
                if has_active_child {
                    // Blocked
                    enriched_tasks[child_idx].normalized_importance = 0.0;
                    enriched_tasks[child_idx].lead_time_factor = 0.0;
                    continue;
                }
                has_active_child = true;
                enriched_tasks[child_idx].normalized_importance = parent_importance;
            } else {
                enriched_tasks[child_idx].normalized_importance = parent_importance;
            }
        } else {
            // Proportional Distribution
            if sibling_importance_sum == 0.0 {
                enriched_tasks[child_idx].normalized_importance =
                    parent_importance / child_indices.len() as f64;
            } else {
                enriched_tasks[child_idx].normalized_importance =
                    (enriched_tasks[child_idx].importance / sibling_importance_sum)
                        * parent_importance;
            }
        }

        // Re-compute lead time factor after inheritance
        let lead_time = enriched_tasks[child_idx]
            .effective_lead_time
            .unwrap_or(enriched_tasks[child_idx].schedule.lead_time);
        enriched_tasks[child_idx].lead_time_factor = calculate_lead_time_factor(
            enriched_tasks[child_idx].effective_due_date,
            lead_time,
            current_time,
        );

        if enriched_tasks[child_idx].lead_time_factor.is_nan() {
            enriched_tasks[child_idx].lead_time_factor = 0.0;
        }
    }
}

/// Finds the root index for a task by walking up its parent chain.
fn find_root_index(task_idx: usize, tasks: &[EnrichedTask], task_lookup: &TaskLookup) -> usize {
    let mut current_idx = task_idx;
    while let Some(parent_id) = &tasks[current_idx].parent_id {
        if let Some(parent_idx) = task_lookup.get(parent_id) {
            current_idx = *parent_idx;
        } else {
            break;
        }
    }
    current_idx
}

/// Computes whether a task has any pending descendants.
fn has_pending_descendants(
    task_idx: usize,
    children_index: &ChildrenLookup,
    tasks: &[EnrichedTask],
) -> bool {
    let child_indices = match children_index.get(&Some(tasks[task_idx].id.clone())) {
        Some(indices) => indices,
        None => return false,
    };

    for &child_idx in child_indices {
        if tasks[child_idx].is_pending {
            return true;
        }
        if has_pending_descendants(child_idx, children_index, tasks) {
            return true;
        }
    }

    false
}

/// Builds a map of tasks blocked by sequential ordering.
fn build_sequential_blocked_map(
    children_index: &ChildrenLookup,
    tasks: &[EnrichedTask],
    task_lookup: &TaskLookup,
) -> HashMap<TaskID, bool> {
    let mut blocked: HashMap<TaskID, bool> = HashMap::new();

    for (parent_id, child_indices) in children_index {
        let parent_id = match parent_id {
            Some(id) => id,
            None => continue,
        };
        let parent_idx = match task_lookup.get(parent_id) {
            Some(idx) => *idx,
            None => continue,
        };

        if !tasks[parent_idx].is_sequential {
            continue;
        }

        let mut has_active_child = false;
        for &child_idx in child_indices {
            if tasks[child_idx].status == TaskStatus::Pending {
                if has_active_child {
                    blocked.insert(tasks[child_idx].id.clone(), true);
                } else {
                    has_active_child = true;
                }
            }
        }
    }

    blocked
}

/// Builds the importance propagation chain from root to the task.
fn build_importance_chain(
    task_idx: usize,
    tasks: &[EnrichedTask],
    children_index: &ChildrenLookup,
    task_lookup: &TaskLookup,
    sequential_blocked: &HashMap<TaskID, bool>,
) -> Vec<ImportanceTrace> {
    let mut lineage: Vec<usize> = Vec::new();
    let mut current_idx = Some(task_idx);

    while let Some(idx) = current_idx {
        lineage.push(idx);
        current_idx = tasks[idx]
            .parent_id
            .as_ref()
            .and_then(|pid| task_lookup.get(pid).copied());
    }

    lineage.reverse();

    lineage
        .into_iter()
        .map(|idx| {
            let parent_idx = tasks[idx]
                .parent_id
                .as_ref()
                .and_then(|pid| task_lookup.get(pid).copied());

            let sibling_importance_sum = parent_idx.and_then(|pidx| {
                let parent_id = tasks[pidx].id.clone();
                children_index.get(&Some(parent_id)).map(|sibling_indices| {
                    sibling_indices
                        .iter()
                        .filter(|&sibling_idx| tasks[*sibling_idx].is_pending)
                        .map(|&sibling_idx| tasks[sibling_idx].importance)
                        .sum()
                })
            });

            let parent_normalized_importance =
                parent_idx.map(|pidx| tasks[pidx].normalized_importance);

            let is_blocked = sequential_blocked
                .get(&tasks[idx].id)
                .copied()
                .unwrap_or(false);

            ImportanceTrace {
                task_id: tasks[idx].id.clone(),
                task_title: tasks[idx].title.clone(),
                importance: tasks[idx].importance,
                sibling_importance_sum,
                parent_normalized_importance,
                normalized_importance: tasks[idx].normalized_importance,
                sequential_blocked: is_blocked,
            }
        })
        .collect()
}

/// Builds the feedback trace for the provided root task.
fn build_feedback_trace(root_idx: usize, tasks: &[EnrichedTask]) -> FeedbackTrace {
    let root = &tasks[root_idx];

    let root_indices: Vec<usize> = tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| task.parent_id.is_none())
        .map(|(idx, _)| idx)
        .collect();

    let total_desired_credits: f64 = root_indices
        .iter()
        .map(|&idx| tasks[idx].desired_credits)
        .sum();
    let total_effective_credits: f64 = root_indices
        .iter()
        .map(|&idx| tasks[idx].effective_credits)
        .sum();

    let target_percent = if total_desired_credits == 0.0 {
        0.0
    } else {
        root.desired_credits / total_desired_credits
    };

    let effective_denominator =
        total_effective_credits.max(FEEDBACK_EPSILON * total_desired_credits);
    let actual_percent = if effective_denominator == 0.0 {
        0.0
    } else {
        root.effective_credits / effective_denominator
    };

    let deviation_ratio = if target_percent == 0.0 {
        1.0
    } else {
        target_percent / actual_percent.max(FEEDBACK_EPSILON)
    };
    let capped_deviation_ratio = deviation_ratio.min(FEEDBACK_DEVIATION_RATIO_CAP);

    FeedbackTrace {
        root_id: root.id.clone(),
        root_title: root.title.clone(),
        desired_credits: root.desired_credits,
        effective_credits: root.effective_credits,
        total_desired_credits,
        total_effective_credits,
        target_percent,
        actual_percent,
        deviation_ratio: capped_deviation_ratio,
        sensitivity: FEEDBACK_SENSITIVITY,
        epsilon: FEEDBACK_EPSILON,
        feedback_factor: root.feedback_factor,
    }
}

/// Builds lead time trace details for a task.
fn build_lead_time_trace(task: &EnrichedTask, current_time: i64) -> LeadTimeTrace {
    let effective_due_date = task.effective_due_date;
    let effective_lead_time = task.effective_lead_time.unwrap_or(task.schedule.lead_time);

    let (stage, time_remaining) = match effective_due_date {
        None => (LeadTimeStage::Ready, None),
        Some(due_date) => {
            if due_date <= current_time {
                (LeadTimeStage::Overdue, Some(due_date - current_time))
            } else {
                let remaining = due_date - current_time;
                if remaining > 2 * effective_lead_time {
                    (LeadTimeStage::TooEarly, Some(remaining))
                } else if remaining > effective_lead_time {
                    (LeadTimeStage::Ramping, Some(remaining))
                } else {
                    (LeadTimeStage::Ready, Some(remaining))
                }
            }
        }
    };

    LeadTimeTrace {
        effective_due_date,
        effective_lead_time,
        time_remaining,
        stage,
        factor: task.lead_time_factor,
        schedule_source: task.effective_schedule_source,
    }
}

/// Builds visibility trace details for a task.
fn build_visibility_trace(
    state: &TunnelState,
    task_idx: usize,
    context: &PriorityContext,
    view_filter: &ViewFilter,
) -> VisibilityTrace {
    let task = &context.tasks[task_idx];
    let contextual = resolve_contextual_visibility(state, task, view_filter, context.current_time);
    let has_pending_descendants =
        has_pending_descendants(task_idx, &context.children_index, &context.tasks);

    VisibilityTrace {
        contextual,
        has_pending_descendants,
        delegated_to_descendants: has_pending_descendants,
        final_visibility: task.visibility,
    }
}

/// Derives the "Projected State" for the View Layer.
pub fn get_prioritized_tasks(
    state: &TunnelState,
    view_filter: &ViewFilter,
    options: &PriorityOptions,
) -> Vec<ComputedTask> {
    // --- Stage 1: Hydrate & Initialize ---
    let PriorityContext { mut tasks, .. } = build_priority_context(state, view_filter, options);

    // --- Stage 3: Sorting ---
    // Sort by: Priority (desc) -> Importance (desc) -> Outline Index (asc)
    tasks.sort_by(|a, b| {
        if (a.priority - b.priority).abs() > PRIORITY_EPSILON {
            b.priority.partial_cmp(&a.priority).unwrap()
        } else if (a.importance - b.importance).abs() > f64::EPSILON {
            // Tiebreaker: higher importance first
            b.importance.partial_cmp(&a.importance).unwrap()
        } else {
            a.outline_index.partial_cmp(&b.outline_index).unwrap()
        }
    });

    tasks
        .into_iter()
        .filter(|t| {
            // Visibility Check
            if !options.include_hidden && !t.visibility {
                return false;
            }

            // Status Check
            if !options.include_hidden
                && options.mode != Some(PriorityMode::PlanOutline)
                && t.status == TaskStatus::Done
                && t.is_acknowledged
            {
                return false;
            }

            // Priority Threshold
            if !options.include_hidden
                && options.mode != Some(PriorityMode::PlanOutline)
                && t.priority <= MIN_PRIORITY
            {
                return false;
            }

            true
        })
        .map(|e| {
            let is_ready = e.is_pending && e.lead_time_factor > 0.0;
            let status = crate::domain::dates::get_urgency_status(
                e.effective_due_date,
                e.effective_lead_time,
                options
                    .context
                    .as_ref()
                    .map(|c| c.current_time)
                    .unwrap_or_else(get_current_timestamp),
            );
            ComputedTask {
                id: e.id,
                title: e.title,
                notes: e.notes,
                parent_id: e.parent_id,
                child_task_ids: e.child_task_ids,
                place_id: e.place_id,
                status: e.status,
                importance: e.importance,
                credit_increment: e.credit_increment,
                credits: e.credits,
                effective_credits: e.effective_credits,
                desired_credits: e.desired_credits,
                credits_timestamp: e.credits_timestamp,
                priority_timestamp: e.priority_timestamp,
                schedule: e.schedule,
                repeat_config: e.repeat_config,
                is_sequential: e.is_sequential,
                is_acknowledged: e.is_acknowledged,
                last_completed_at: e.last_completed_at,
                score: e.priority,
                normalized_importance: e.normalized_importance,
                // TODO: Remove this field (doesn't exist in TS ComputedTask)
                is_blocked: !is_ready && e.is_pending,
                is_visible: e.visibility,
                // TODO: Remove this field (doesn't exist in TS ComputedTask)
                is_open: true,
                is_container: e.is_container,
                is_pending: e.is_pending,
                is_ready,
                effective_due_date: e.effective_due_date,
                effective_lead_time: e.effective_lead_time,
                effective_schedule_source: e.effective_schedule_source,
                urgency_status: status,
            }
        })
        .collect()
}

/// Computes a detailed score trace for a single task.
///
/// This reuses the core priority pipeline so the trace stays in lockstep with
/// the actual scoring behavior (no separate trace math).
pub fn get_score_trace(
    state: &TunnelState,
    view_filter: &ViewFilter,
    options: &PriorityOptions,
    task_id: &TaskID,
) -> Option<ScoreTrace> {
    let context = build_priority_context(state, view_filter, options);
    let task_idx = context.task_lookup.get(task_id).copied()?;
    let root_idx = find_root_index(task_idx, &context.tasks, &context.task_lookup);

    let sequential_blocked = build_sequential_blocked_map(
        &context.children_index,
        &context.tasks,
        &context.task_lookup,
    );
    let importance_chain = build_importance_chain(
        task_idx,
        &context.tasks,
        &context.children_index,
        &context.task_lookup,
        &sequential_blocked,
    );

    let feedback = build_feedback_trace(root_idx, &context.tasks);
    let lead_time = build_lead_time_trace(&context.tasks[task_idx], context.current_time);
    let visibility = build_visibility_trace(state, task_idx, &context, view_filter);

    let task = &context.tasks[task_idx];
    let root = &context.tasks[root_idx];
    let factors = ScoreFactors {
        visibility_factor: if task.visibility { 1.0 } else { 0.0 },
        normalized_importance: task.normalized_importance,
        feedback_factor: root.feedback_factor,
        lead_time_factor: task.lead_time_factor,
    };

    Some(ScoreTrace {
        task_id: task.id.clone(),
        task_title: task.title.clone(),
        score: task.priority,
        computed_at: context.current_time,
        factors,
        importance_chain,
        feedback,
        lead_time,
        visibility,
    })
}

#[cfg(test)]
mod score_trace_tests {
    use super::*;
    use crate::types::{Schedule, ScheduleType};
    use std::collections::HashMap;

    #[test]
    fn score_trace_matches_factor_product() {
        let task_id = TaskID::from("root");
        let task = PersistedTask {
            id: task_id.clone(),
            title: "Root".to_string(),
            notes: String::new(),
            parent_id: None,
            child_task_ids: vec![],
            place_id: None,
            status: TaskStatus::Pending,
            importance: 1.0,
            credit_increment: None,
            credits: 10.0,
            desired_credits: 10.0,
            credits_timestamp: 0,
            priority_timestamp: 0,
            schedule: Schedule {
                schedule_type: ScheduleType::Once,
                due_date: None,
                lead_time: 0,
                last_done: None,
            },
            repeat_config: None,
            is_sequential: false,
            is_acknowledged: false,
            last_completed_at: None,
        };

        let mut tasks = HashMap::new();
        tasks.insert(task_id.clone(), task);

        let state = TunnelState {
            tasks,
            root_task_ids: vec![task_id.clone()],
            ..Default::default()
        };

        let view_filter = ViewFilter {
            place_id: Some("All".to_string()),
        };
        let options = PriorityOptions {
            include_hidden: true,
            mode: Some(PriorityMode::DoList),
            context: Some(Context {
                current_place_id: None,
                current_time: 0,
            }),
        };

        let trace =
            get_score_trace(&state, &view_filter, &options, &task_id).expect("trace missing");

        let product = trace.factors.visibility_factor
            * trace.factors.normalized_importance
            * trace.factors.feedback_factor
            * trace.factors.lead_time_factor;

        assert!((trace.score - product).abs() < 1e-9);
        assert_eq!(trace.importance_chain.len(), 1);
        assert_eq!(trace.lead_time.stage, LeadTimeStage::Ready);
        assert!(trace.visibility.final_visibility);
    }
}

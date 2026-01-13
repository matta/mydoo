use crate::domain::feedback::calculate_feedback_factors;
use crate::domain::readiness::calculate_lead_time_factor;
use crate::domain::visibility::calculate_contextual_visibility;
use crate::types::{
    ComputedTask, Context, EnrichedTask, PersistedTask, PriorityMode, PriorityOptions,
    ScheduleSource, ScheduleType, TaskID, TaskStatus, TunnelState, ViewFilter,
};
use crate::utils::time::{get_current_timestamp, get_interval_ms};

use std::collections::HashMap;

const CREDITS_HALF_LIFE_MILLIS: f64 = 4.0 * 60.0 * 60.0 * 1000.0; // 4 hours
const DEFAULT_CREDIT_INCREMENT: f64 = 0.5;
const MIN_PRIORITY: f64 = 0.001;
const PRIORITY_EPSILON: f64 = 0.000001;

/// Builds lookup indexes for tasks and sorts children based on explicit order logic.
fn build_indexes(
    state: &TunnelState,
    enriched_tasks: &[EnrichedTask],
) -> (HashMap<TaskID, usize>, HashMap<Option<TaskID>, Vec<usize>>) {
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
fn assign_outline_indexes(
    enriched_tasks: &mut [EnrichedTask],
    children_index: &HashMap<Option<TaskID>, Vec<usize>>,
) {
    let mut current_index = 0;
    traverse_assign(None, enriched_tasks, children_index, &mut current_index);
}

fn traverse_assign(
    parent_id: Option<TaskID>,
    enriched_tasks: &mut [EnrichedTask],
    children_index: &HashMap<Option<TaskID>, Vec<usize>>,
    current_index: &mut u32,
) {
    if let Some(child_indices) = children_index.get(&parent_id) {
        for &idx in child_indices {
            enriched_tasks[idx].outline_index = *current_index;
            *current_index += 1;
            let id = enriched_tasks[idx].id.clone();
            traverse_assign(Some(id), enriched_tasks, children_index, current_index);
        }
    }
}

/// Hydrates a persisted task into an enriched task object.
fn hydrate_task(persisted: &PersistedTask) -> EnrichedTask {
    let is_container = !persisted.child_task_ids.is_empty();
    let is_pending = persisted.status == TaskStatus::Pending;

    EnrichedTask {
        id: persisted.id.clone(),
        title: persisted.title.clone(),
        notes: persisted.notes.clone(),
        parent_id: persisted.parent_id.clone(),
        child_task_ids: persisted.child_task_ids.clone(),
        place_id: persisted.place_id.clone(),
        status: persisted.status,
        importance: persisted.importance,
        credit_increment: persisted.credit_increment,
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
        outline_index: 0,
        is_container,
        is_pending,
        is_ready: false,

        effective_due_date: persisted.schedule.due_date,
        effective_lead_time: Some(persisted.schedule.lead_time),
        effective_schedule_source: if persisted.schedule.due_date.is_some() {
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

    // --- Phase 0: Build Indexes & Outline Order ---
    let (_, children_index) = build_indexes(state, enriched_tasks);
    assign_outline_indexes(enriched_tasks, &children_index);

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

        let time_delta = current_time as f64 - task.credits_timestamp as f64;
        task.effective_credits = task.credits * 0.5f64.powf(time_delta / CREDITS_HALF_LIFE_MILLIS);
    }

    calculate_feedback_factors(enriched_tasks);

    // --- Phase 2: Unified DFS Traversal ---
    let root_indices = children_index.get(&None).cloned().unwrap_or_default();

    for &idx in &root_indices {
        // Roots start with their raw importance
        enriched_tasks[idx].normalized_importance = enriched_tasks[idx].importance;

        // Root Defaults
        if enriched_tasks[idx].credit_increment.is_none() {
            enriched_tasks[idx].credit_increment = Some(DEFAULT_CREDIT_INCREMENT);
        }

        evaluate_task_recursive(idx, None, enriched_tasks, &children_index, current_time);
    }
}

fn evaluate_task_recursive(
    task_idx: usize,
    root_idx: Option<usize>,
    enriched_tasks: &mut [EnrichedTask],
    children_index: &HashMap<Option<TaskID>, Vec<usize>>,
    current_time: u64,
) -> bool {
    let child_indices = children_index
        .get(&Some(enriched_tasks[task_idx].id.clone()))
        .cloned()
        .unwrap_or_default();

    let effective_root_idx = root_idx.unwrap_or(task_idx);

    // --- Pre-Order: Propagate from Parent to Children ---
    process_children(task_idx, &child_indices, enriched_tasks, current_time);

    // --- Recurse ---
    let mut has_visible_descendant = false;
    for &child_idx in &child_indices {
        has_visible_descendant = evaluate_task_recursive(
            child_idx,
            Some(effective_root_idx),
            enriched_tasks,
            children_index,
            current_time,
        ) || has_visible_descendant;
    }

    let feedback_factor = enriched_tasks[effective_root_idx].feedback_factor;
    let task = &mut enriched_tasks[task_idx];
    if !child_indices.is_empty() && has_visible_descendant {
        // --- Post-Order: Aggregate ---
        task.visibility = false;
        task.priority = 0.0;
    } else {
        // Compute Final Priority
        let visibility_factor = if task.visibility { 1.0 } else { 0.0 };
        let lead_time_factor = task.lead_time_factor;
        let normalized_importance = task.normalized_importance;

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

    enriched_tasks[task_idx].visibility || has_visible_descendant
}

fn process_children(
    parent_idx: usize,
    child_indices: &[usize],
    enriched_tasks: &mut [EnrichedTask],
    current_time: u64,
) {
    if child_indices.is_empty() {
        return;
    }

    let sibling_importance_sum: f64 = child_indices
        .iter()
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

/// Derives the "Projected State" for the View Layer.
pub fn get_prioritized_tasks(
    state: &TunnelState,
    view_filter: &ViewFilter,
    options: &PriorityOptions,
) -> Vec<ComputedTask> {
    // --- Stage 1: Hydrate & Initialize ---
    let mut enriched_tasks: Vec<EnrichedTask> = state.tasks.values().map(hydrate_task).collect();

    // Phase 0: Date Resolution (Pass 1)
    for task in enriched_tasks.iter_mut() {
        if let (ScheduleType::Routinely, Some(last_done), Some(repeat_config)) = (
            task.schedule.schedule_type,
            task.schedule.last_done,
            &task.repeat_config,
        ) {
            let interval_ms = get_interval_ms(repeat_config.frequency, repeat_config.interval);
            task.effective_due_date = Some(last_done + interval_ms);
            task.effective_lead_time = Some(task.schedule.lead_time);
            task.effective_schedule_source = Some(ScheduleSource::Myself);
        }
    }

    // --- Stage 2: Process ---
    recalculate_priorities(
        state,
        &mut enriched_tasks,
        view_filter,
        options.context.as_ref(),
    );

    // --- Stage 3: Sanitize & Sort ---
    enriched_tasks.sort_by(|a, b| {
        if (a.priority - b.priority).abs() > PRIORITY_EPSILON {
            b.priority.partial_cmp(&a.priority).unwrap()
        } else if (a.importance - b.importance).abs() > f64::EPSILON {
            b.importance.partial_cmp(&a.importance).unwrap()
        } else {
            a.outline_index.cmp(&b.outline_index)
        }
    });

    enriched_tasks
        .into_iter()
        .filter(|t| {
            // Visibility Check
            if !options.include_hidden && !t.visibility {
                return false;
            }

            // Status Check
            if options.mode != Some(PriorityMode::PlanOutline)
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
                desired_credits: e.desired_credits,
                credits_timestamp: e.credits_timestamp,
                priority_timestamp: e.priority_timestamp,
                schedule: e.schedule,
                repeat_config: e.repeat_config,
                is_sequential: e.is_sequential,
                is_acknowledged: e.is_acknowledged,
                last_completed_at: e.last_completed_at,
                is_container: e.is_container,
                is_pending: e.is_pending,
                is_ready,
                effective_due_date: e.effective_due_date,
                effective_lead_time: e.effective_lead_time,
                effective_schedule_source: e.effective_schedule_source,
            }
        })
        .collect()
}

//! Balance View Projection
//!
//! Projects task data into `BalanceData` for the Balance View.
//! This module computes target vs actual effort percentages for root goals.

use crate::domain::constants::CREDITS_HALF_LIFE_MILLIS;
use crate::types::{BalanceData, BalanceItem, EnrichedTask, TaskID, TunnelState};
use crate::utils::time::get_current_timestamp;

/// Threshold below which a goal is considered "starving" (actual < target by this margin).
/// Using a small epsilon to avoid floating-point noise.
pub const STARVING_THRESHOLD: f64 = 0.001;

/// Calculates target_percent, actual_percent, and is_starving for a balance item.
fn calculate_balance_percentages(
    task_desired_credits: f64,
    task_effective_credits: f64,
    total_desired: f64,
    total_effective: f64,
) -> (f64, f64, bool) {
    let target_percent = if total_desired > 0.0 {
        task_desired_credits / total_desired
    } else {
        0.0
    };

    let actual_percent = if total_effective > 0.0 {
        task_effective_credits / total_effective
    } else {
        0.0
    };

    let is_starving = actual_percent < target_percent - STARVING_THRESHOLD;
    (target_percent, actual_percent, is_starving)
}

/// Computes balance data directly from TunnelState.
///
/// This is the primary entry point for the Balance View. It:
/// 1. Extracts root tasks from the state
/// 2. Calculates effective_credits with time decay
/// 3. Aggregates child credits up to roots
/// 4. Computes target_percent and actual_percent
/// 5. Marks goals as "starving" if under-served
///
/// # Arguments
/// * `state` - The current tunnel state containing all tasks
///
/// # Returns
/// `BalanceData` containing balance items for each root goal.
pub fn get_balance_data(state: &TunnelState) -> BalanceData {
    get_balance_data_with_time(state, get_current_timestamp())
}

/// Computes balance data with a specific timestamp.
pub fn get_balance_data_with_time(state: &TunnelState, current_time: i64) -> BalanceData {
    use std::collections::HashMap;

    let tasks = &state.tasks;

    let mut effective_credits_map: HashMap<TaskID, f64> = HashMap::new();
    for task in tasks.values() {
        let time_delta = current_time - task.credits_timestamp;
        let decayed = task.credits * 0.5f64.powf(time_delta as f64 / CREDITS_HALF_LIFE_MILLIS);
        effective_credits_map.insert(task.id.clone(), decayed);
    }

    let mut children_map: HashMap<Option<TaskID>, Vec<TaskID>> = HashMap::new();
    for task in tasks.values() {
        children_map
            .entry(task.parent_id.clone())
            .or_default()
            .push(task.id.clone());
    }

    fn aggregate_credits(
        task_id: &TaskID,
        children_map: &HashMap<Option<TaskID>, Vec<TaskID>>,
        effective_credits_map: &HashMap<TaskID, f64>,
        aggregated: &mut HashMap<TaskID, f64>,
    ) -> f64 {
        if let Some(&cached) = aggregated.get(task_id) {
            return cached;
        }

        let own_credits = effective_credits_map.get(task_id).copied().unwrap_or(0.0);
        let children_credits: f64 = children_map
            .get(&Some(task_id.clone()))
            .map(|children| {
                children
                    .iter()
                    .map(|child_id| {
                        aggregate_credits(child_id, children_map, effective_credits_map, aggregated)
                    })
                    .sum()
            })
            .unwrap_or(0.0);

        let total = own_credits + children_credits;
        aggregated.insert(task_id.clone(), total);
        total
    }

    let mut aggregated_credits: HashMap<TaskID, f64> = HashMap::new();

    let roots: Vec<_> = state
        .root_task_ids
        .iter()
        .map(|id| tasks.get(id).unwrap())
        .filter(|t| t.title != "Inbox")
        .collect();

    for root in &roots {
        aggregate_credits(
            &root.id,
            &children_map,
            &effective_credits_map,
            &mut aggregated_credits,
        );
    }

    if roots.is_empty() {
        return BalanceData {
            items: vec![],
            total_credits: 0.0,
        };
    }

    let total_desired: f64 = roots.iter().map(|t| t.desired_credits).sum();
    let total_effective: f64 = roots
        .iter()
        .map(|t| aggregated_credits.get(&t.id).copied().unwrap_or(0.0))
        .sum();

    let items: Vec<BalanceItem> = roots
        .iter()
        .map(|task| {
            let effective = aggregated_credits.get(&task.id).copied().unwrap_or(0.0);
            let (target_percent, actual_percent, is_starving) = calculate_balance_percentages(
                task.desired_credits,
                effective,
                total_desired,
                total_effective,
            );

            BalanceItem {
                id: task.id.clone(),
                title: task.title.clone(),
                target_percent,
                actual_percent,
                is_starving,
                desired_credits: task.desired_credits,
                effective_credits: effective,
            }
        })
        .collect();

    BalanceData {
        items,
        total_credits: total_effective,
    }
}

/// Projects enriched tasks into balance data for the Balance View.
///
/// This is an alternative entry point when you already have enriched tasks
/// (e.g., from the priority calculation pipeline).
pub fn project_balance_data(enriched_tasks: &[EnrichedTask]) -> BalanceData {
    let roots: Vec<&EnrichedTask> = enriched_tasks
        .iter()
        .filter(|t| t.parent_id.is_none())
        .filter(|t| t.title != "Inbox")
        .collect();

    if roots.is_empty() {
        return BalanceData {
            items: vec![],
            total_credits: 0.0,
        };
    }

    let total_desired: f64 = roots.iter().map(|t| t.desired_credits).sum();
    let total_effective: f64 = roots.iter().map(|t| t.effective_credits).sum();

    let items: Vec<BalanceItem> = roots
        .iter()
        .map(|task| {
            let (target_percent, actual_percent, is_starving) = calculate_balance_percentages(
                task.desired_credits,
                task.effective_credits,
                total_desired,
                total_effective,
            );

            BalanceItem {
                id: task.id.clone(),
                title: task.title.clone(),
                target_percent,
                actual_percent,
                is_starving,
                desired_credits: task.desired_credits,
                effective_credits: task.effective_credits,
            }
        })
        .collect();

    BalanceData {
        items,
        total_credits: total_effective,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Schedule, ScheduleType, TaskID, TaskStatus};

    fn mock_root(id: &str, title: &str, desired: f64, effective: f64) -> EnrichedTask {
        EnrichedTask {
            id: TaskID::from(id),
            title: title.to_string(),
            notes: String::new(),
            parent_id: None,
            child_task_ids: vec![],
            place_id: None,
            status: TaskStatus::Pending,
            importance: 1.0,
            credit_increment: None,
            credits: 0.0,
            desired_credits: desired,
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
            effective_credits: effective,
            feedback_factor: 1.0,
            lead_time_factor: 1.0,
            normalized_importance: 1.0,
            priority: 0.0,
            visibility: true,
            outline_index: 0.0,
            is_container: false,
            is_pending: true,
            is_ready: true,
            effective_due_date: None,
            effective_lead_time: None,
            effective_schedule_source: None,
        }
    }

    fn mock_child(id: &str, parent_id: &str) -> EnrichedTask {
        let mut task = mock_root(id, id, 0.0, 0.0);
        task.parent_id = Some(TaskID::from(parent_id));
        task
    }

    #[test]
    fn test_empty_tasks() {
        let result = project_balance_data(&[]);
        assert!(result.items.is_empty());
        assert_eq!(result.total_credits, 0.0);
    }

    #[test]
    fn test_single_root() {
        let tasks = vec![mock_root("1", "Health", 100.0, 50.0)];
        let result = project_balance_data(&tasks);

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].title, "Health");
        assert_eq!(result.items[0].target_percent, 1.0);
        assert_eq!(result.items[0].actual_percent, 1.0);
        assert!(!result.items[0].is_starving);
        assert_eq!(result.total_credits, 50.0);
    }

    #[test]
    fn test_two_balanced_roots() {
        let tasks = vec![
            mock_root("1", "Health", 50.0, 50.0),
            mock_root("2", "Career", 50.0, 50.0),
        ];
        let result = project_balance_data(&tasks);

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].target_percent, 0.5);
        assert_eq!(result.items[0].actual_percent, 0.5);
        assert!(!result.items[0].is_starving);
        assert!(!result.items[1].is_starving);
    }

    #[test]
    fn test_starving_detection() {
        let tasks = vec![
            mock_root("1", "Health", 80.0, 20.0),
            mock_root("2", "Career", 20.0, 80.0),
        ];
        let result = project_balance_data(&tasks);

        assert!(result.items[0].is_starving);
        assert!(!result.items[1].is_starving);
    }

    #[test]
    fn test_inbox_excluded() {
        let tasks = vec![
            mock_root("inbox", "Inbox", 10.0, 10.0),
            mock_root("1", "Health", 50.0, 50.0),
        ];
        let result = project_balance_data(&tasks);

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].title, "Health");
    }

    #[test]
    fn test_children_excluded() {
        let tasks = vec![
            mock_root("1", "Health", 100.0, 100.0),
            mock_child("1.1", "1"),
            mock_child("1.2", "1"),
        ];
        let result = project_balance_data(&tasks);

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].title, "Health");
    }

    #[test]
    fn test_zero_desired_credits() {
        let tasks = vec![
            mock_root("1", "Health", 0.0, 50.0),
            mock_root("2", "Career", 0.0, 50.0),
        ];
        let result = project_balance_data(&tasks);

        assert_eq!(result.items[0].target_percent, 0.0);
        assert_eq!(result.items[1].target_percent, 0.0);
    }

    #[test]
    fn test_zero_effective_credits() {
        let tasks = vec![
            mock_root("1", "Health", 50.0, 0.0),
            mock_root("2", "Career", 50.0, 0.0),
        ];
        let result = project_balance_data(&tasks);

        assert_eq!(result.items[0].actual_percent, 0.0);
        assert_eq!(result.items[1].actual_percent, 0.0);
        assert_eq!(result.total_credits, 0.0);
    }

    mod tunnel_state_tests {
        use super::*;
        use crate::types::{PersistedTask, TunnelState};
        use std::collections::HashMap;

        fn make_persisted_root(
            id: &str,
            title: &str,
            desired: f64,
            credits: f64,
            timestamp: i64,
        ) -> PersistedTask {
            PersistedTask {
                id: TaskID::from(id),
                title: title.to_string(),
                notes: String::new(),
                parent_id: None,
                child_task_ids: vec![],
                place_id: None,
                status: TaskStatus::Pending,
                importance: 1.0,
                credit_increment: None,
                credits,
                desired_credits: desired,
                credits_timestamp: timestamp,
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
            }
        }

        fn make_persisted_child(id: &str, parent_id: &str) -> PersistedTask {
            let mut task = make_persisted_root(id, id, 0.0, 0.0, 0);
            task.parent_id = Some(TaskID::from(parent_id));
            task
        }

        fn make_state(tasks: Vec<PersistedTask>) -> TunnelState {
            let root_ids: Vec<TaskID> = tasks
                .iter()
                .filter(|t| t.parent_id.is_none())
                .map(|t| t.id.clone())
                .collect();

            let task_map: HashMap<TaskID, PersistedTask> =
                tasks.into_iter().map(|t| (t.id.clone(), t)).collect();

            TunnelState {
                tasks: task_map,
                root_task_ids: root_ids,
                places: HashMap::new(),
                next_task_id: 100,
                next_place_id: 1,
                metadata: None,
            }
        }

        #[test]
        fn test_get_balance_data_empty() {
            let state = make_state(vec![]);
            let result = get_balance_data_with_time(&state, 0);
            assert!(result.items.is_empty());
        }

        #[test]
        fn test_get_balance_data_single_root() {
            let state = make_state(vec![make_persisted_root("1", "Health", 100.0, 50.0, 0)]);
            let result = get_balance_data_with_time(&state, 0);

            assert_eq!(result.items.len(), 1);
            assert_eq!(result.items[0].title, "Health");
            assert_eq!(result.items[0].target_percent, 1.0);
            assert_eq!(result.items[0].actual_percent, 1.0);
            assert_eq!(result.items[0].effective_credits, 50.0);
        }

        #[test]
        fn test_get_balance_data_with_decay() {
            let half_life_ms = 604_800_000i64; // 7 days
            let state = make_state(vec![make_persisted_root("1", "Health", 100.0, 100.0, 0)]);

            let result = get_balance_data_with_time(&state, half_life_ms);

            assert!((result.items[0].effective_credits - 50.0).abs() < 0.01);
        }

        #[test]
        fn test_get_balance_data_aggregates_children() {
            let mut child = make_persisted_child("1.1", "1");
            child.credits = 30.0;

            let state = make_state(vec![
                make_persisted_root("1", "Health", 100.0, 20.0, 0),
                child,
            ]);

            let result = get_balance_data_with_time(&state, 0);

            assert_eq!(result.items.len(), 1);
            assert_eq!(result.items[0].effective_credits, 50.0);
        }

        #[test]
        fn test_get_balance_data_excludes_inbox() {
            let state = make_state(vec![
                make_persisted_root("inbox", "Inbox", 10.0, 10.0, 0),
                make_persisted_root("1", "Health", 100.0, 50.0, 0),
            ]);

            let result = get_balance_data_with_time(&state, 0);

            assert_eq!(result.items.len(), 1);
            assert_eq!(result.items[0].title, "Health");
        }
    }
}

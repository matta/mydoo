//! Balance Distribution Logic
//!
//! Handles the "fixed pie" redistribution of task percentages.

use crate::types::{BalanceData, TaskID};
use std::collections::HashMap;

/// Minimum percentage allowed for a task when it is being DIRECTLY adjusted by
/// the user (1%).
pub const MIN_PERCENTAGE: f64 = 0.01;
/// Maximum percentage allowed for a task when it is being DIRECTLY adjusted by
/// the user (99%).
pub const MAX_PERCENTAGE: f64 = 0.99;

/// Redistributes percentages among tasks when one task's target value changes.
///
/// This maintains a "fixed pie" logic where all percentage values must sum up
/// to exactly 1.0. When adjusting the target task's percentage, the remaining
/// proportion is distributed proportionally among all other tasks based on
/// their previous share. If multiple tasks previously shared the same percentage,
/// they will continue to have an equal slice of the new remaining proportion.
///
/// The `new_value` will be strictly clamped between `MIN_PERCENTAGE` and `MAX_PERCENTAGE`.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use tasklens_core::domain::balance_distribution::redistribute_percentages;
/// use tasklens_core::types::TaskID;
///
/// let id1 = TaskID::new();
/// let id2 = TaskID::new();
/// let mut map = HashMap::new();
/// map.insert(id1.clone(), 0.5);
/// map.insert(id2.clone(), 0.5);
///
/// // Update id1 to 0.8; id2 should shrink to 0.2
/// let new_map = redistribute_percentages(&map, &id1, 0.8);
///
/// assert_eq!(new_map.get(&id1), Some(&0.8));
/// assert!((new_map.get(&id2).unwrap() - 0.2).abs() < 1e-10);
/// ```
pub fn redistribute_percentages(
    current_map: &HashMap<TaskID, f64>,
    target_id: &TaskID,
    new_value: f64,
) -> HashMap<TaskID, f64> {
    let num_items = current_map.len();
    if num_items <= 1 {
        let mut result = HashMap::new();
        result.insert(target_id.clone(), 1.0);
        return result;
    }

    // 1. Clamp the target value to the allowed user range.
    // Note: We don't dynamic-clamp based on others here because the algorithm
    // is free to push others below MIN_PERCENTAGE to preserve ratios.
    let clamped_target = new_value.clamp(MIN_PERCENTAGE, MAX_PERCENTAGE);
    let target_complement = (1.0 - clamped_target).max(0.0);

    // 2. Proportional distribution
    let mut result = current_map.clone();
    result.insert(target_id.clone(), clamped_target);

    let sum_others_initial: f64 = current_map
        .iter()
        .filter(|(id, _)| *id != target_id)
        .map(|(_, v)| *v)
        .sum();

    if sum_others_initial <= 1e-10 {
        // Fallback: even split if others were essentially zero
        let num_others = (num_items - 1) as f64;
        let share = target_complement / num_others;
        for (id, val) in result.iter_mut() {
            if id != target_id {
                *val = share;
            }
        }
    } else {
        // Perfect proportional scaling. This maintains "relative priority memory".
        let scale_factor = target_complement / sum_others_initial;
        for (id, val) in result.iter_mut() {
            if id != target_id {
                *val *= scale_factor;
            }
        }
    }

    // 3. Final safety normalization to exactly 1.0, preserving the user's clamped_target exactly.
    let final_others_sum: f64 = result
        .iter()
        .filter(|(id, _)| *id != target_id)
        .map(|(_, v)| *v)
        .sum();

    if final_others_sum > 0.0 {
        let norm_factor = (1.0 - clamped_target) / final_others_sum;
        for (id, val) in result.iter_mut() {
            if id != target_id {
                *val *= norm_factor;
            }
        }
    }

    result
}

/// Calculates the new absolute credits for all tasks based on a percentage distribution.
pub fn apply_redistribution_to_credits(
    balance_data: &BalanceData,
    percentages: &HashMap<TaskID, f64>,
) -> HashMap<TaskID, f64> {
    let total = balance_data.items.len() as f64;

    percentages
        .iter()
        .map(|(id, percent)| (id.clone(), percent * total))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_sum_is_one(map: &HashMap<TaskID, f64>) {
        let sum: f64 = map.values().sum();
        assert!(
            (sum - 1.0).abs() < 1e-15,
            "Sum should be exactly 1.0, but was {}",
            sum
        );
    }

    #[test]
    fn test_redistribute_two_items() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        map.insert(t1.clone(), 0.5);
        map.insert(t2.clone(), 0.5);

        let result = redistribute_percentages(&map, &t1, 0.75);

        assert_eq!(result[&t1], 0.75);
        assert_eq!(result[&t2], 0.25);
        assert_sum_is_one(&result);
    }

    #[test]
    fn test_redistribute_three_items() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        let t3 = TaskID::new();
        map.insert(t1.clone(), 0.333);
        map.insert(t2.clone(), 0.333);
        map.insert(t3.clone(), 0.333);

        let result = redistribute_percentages(&map, &t1, 0.5);

        assert_eq!(result[&t1], 0.5);
        // Both others should be equal and sum to 0.5
        assert_eq!(result[&t2], 0.25);
        assert_eq!(result[&t3], 0.25);
        assert_sum_is_one(&result);
    }

    #[test]
    fn test_idempotency_and_stability() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        let t3 = TaskID::new();
        map.insert(t1.clone(), 0.2);
        map.insert(t2.clone(), 0.4);
        map.insert(t3.clone(), 0.4);

        // First move: 0.2 -> 0.6
        let result1 = redistribute_percentages(&map, &t1, 0.6);
        assert_eq!(result1[&t1], 0.6, "Target should be exactly 0.6");
        assert_sum_is_one(&result1);

        // Second move: call it again with the output of the first one, but same target value
        // This simulates a re-render/re-input loop. It MUST return the exact same bits.
        let result2 = redistribute_percentages(&result1, &t1, 0.6);
        assert_eq!(
            result1, result2,
            "Subsequent calls with same value must be bit-identical to prevent render loops"
        );
    }

    #[test]
    fn test_memory_of_relative_priorities() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        let t3 = TaskID::new();
        map.insert(t1.clone(), 0.2);
        map.insert(t2.clone(), 0.4);
        map.insert(t3.clone(), 0.4);

        // Move T1 to 0.98. T2 and T3 should become 0.01 each.
        let result1 = redistribute_percentages(&map, &t1, 0.98);
        assert_eq!(result1[&t1], 0.98);
        assert!((result1[&t2] - 0.01).abs() < 1e-12);
        assert!((result1[&t3] - 0.01).abs() < 1e-12);

        // Move T1 to 0.99. T2 and T3 should become 0.005 each.
        // This confirms the algorithm is free to go below MIN_PERCENTAGE (0.01) to preserve ratios.
        let result2 = redistribute_percentages(&result1, &t1, 0.99);
        assert_eq!(result2[&t1], 0.99);
        assert!((result2[&t2] - 0.005).abs() < 1e-12);
        assert!((result2[&t3] - 0.005).abs() < 1e-12);

        // Move T1 back to 0.2. T2 and T3 should return to exactly 0.4.
        // This confirms "Memory" of relative priorities is maintained.
        let result3 = redistribute_percentages(&result2, &t1, 0.2);
        assert_eq!(result3[&t1], 0.2);
        assert!((result3[&t2] - 0.4).abs() < 1e-12);
        assert!((result3[&t3] - 0.4).abs() < 1e-12);
        assert_sum_is_one(&result3);
    }

    #[test]
    fn test_user_limit_max() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        let t3 = TaskID::new();
        map.insert(t1.clone(), 0.33);
        map.insert(t2.clone(), 0.33);
        map.insert(t3.clone(), 0.34);

        // The user is limited to MAX_PERCENTAGE (0.99)
        let result = redistribute_percentages(&map, &t1, 1.0);

        assert_eq!(result[&t1], MAX_PERCENTAGE);
        assert_sum_is_one(&result);
    }

    #[test]
    fn test_user_limit_min() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        map.insert(t1.clone(), 0.5);
        map.insert(t2.clone(), 0.5);

        // The user is limited to MIN_PERCENTAGE (0.01)
        let result = redistribute_percentages(&map, &t1, 0.0);

        assert_eq!(result[&t1], MIN_PERCENTAGE);
        assert!((result[&t2] - MAX_PERCENTAGE).abs() < 1e-12);
        assert_sum_is_one(&result);
    }
}

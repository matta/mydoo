//! Balance Distribution Logic
//!
//! Handles the "fixed pie" redistribution of task percentages.

use crate::types::{BalanceData, TaskID};
use std::collections::HashMap;

/// Minimum percentage allowed for any task (1%).
pub const MIN_PERCENTAGE: f64 = 0.01;
/// Maximum percentage allowed for any task (99%).
pub const MAX_PERCENTAGE: f64 = 0.99;

/// Redistributes percentages among tasks when one task's target value changes.
///
/// # Arguments
/// * `current_map` - Current map of TaskID to target percentage (0.0 to 1.0).
/// * `target_id` - The ID of the task being adjusted.
/// * `new_value` - The new target percentage for the target task.
///
/// # Returns
/// A new map with redistributed percentages summing to 1.0.
pub fn redistribute_percentages(
    current_map: &HashMap<TaskID, f64>,
    target_id: &TaskID,
    new_value: f64,
) -> HashMap<TaskID, f64> {
    let mut result = current_map.clone();

    // Clamp new value
    let new_value = new_value.clamp(MIN_PERCENTAGE, MAX_PERCENTAGE);

    // If we only have the target task, just return clamped
    if current_map.len() <= 1 {
        result.insert(target_id.clone(), 1.0); // Or clamp? If single, must be 1.0 technically.
        return result;
    }

    result.insert(target_id.clone(), new_value);

    let remaining_pie = 1.0 - new_value;

    // Calculate current sum of OTHERS
    let mut sum_others = 0.0;
    for (id, val) in current_map.iter() {
        if id != target_id {
            sum_others += val;
        }
    }

    // If others sum to 0 (edge case), even split
    if sum_others <= 0.0001 {
        let count_others = (current_map.len() - 1) as f64;
        let split = remaining_pie / count_others;
        for id in result.keys().cloned().collect::<Vec<_>>() {
            if &id != target_id {
                result.insert(id, split.max(MIN_PERCENTAGE));
            }
        }
    } else {
        // Proportional redistribution
        for id in result.keys().cloned().collect::<Vec<_>>() {
            if &id != target_id {
                let current_val = current_map.get(&id).copied().unwrap_or(0.0);
                let new_share = (current_val / sum_others) * remaining_pie;
                // We might violate min percentage here.
                // For MVP, we will just do simple ratio and maybe do a final fixup pass if needed,
                // but let's see. The prompt asked for simple proportional.
                result.insert(id, new_share);
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

    #[test]
    fn test_redistribute_two_items() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        map.insert(t1.clone(), 0.5);
        map.insert(t2.clone(), 0.5);

        let result = redistribute_percentages(&map, &t1, 0.75);

        assert!((result[&t1] - 0.75).abs() < 0.001);
        assert!((result[&t2] - 0.25).abs() < 0.001);
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

        assert!((result[&t1] - 0.5).abs() < 0.001);
        // Remaining 0.5 should be split evenly between t2 and t3 (relative to their original Equal weights)
        assert!((result[&t2] - 0.25).abs() < 0.001);
        assert!((result[&t3] - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_clamping_min() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        map.insert(t1.clone(), 0.5);
        map.insert(t2.clone(), 0.5);

        let result = redistribute_percentages(&map, &t1, 0.0);

        assert!((result[&t1] - MIN_PERCENTAGE).abs() < 0.001);
        assert!((result[&t2] - (1.0 - MIN_PERCENTAGE)).abs() < 0.001);
    }

    #[test]
    fn test_clamping_max() {
        let mut map = HashMap::new();
        let t1 = TaskID::new();
        let t2 = TaskID::new();
        map.insert(t1.clone(), 0.5);
        map.insert(t2.clone(), 0.5);

        let result = redistribute_percentages(&map, &t1, 1.0);

        assert!((result[&t1] - MAX_PERCENTAGE).abs() < 0.001);
        assert!((result[&t2] - (1.0 - MAX_PERCENTAGE)).abs() < 0.001);
    }
}

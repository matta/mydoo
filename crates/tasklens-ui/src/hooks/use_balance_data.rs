//! Balance Data Hook
//!
//! Provides reactive balance data for the Balance View.

use dioxus::prelude::*;
use tasklens_core::get_balance_data;
use tasklens_core::types::BalanceData;

/// A hook that computes balance data from the current tunnel state.
///
/// This hook automatically recomputes when the underlying document changes.
/// It returns a memoized `BalanceData` containing:
/// - List of balance items (one per root goal, excluding Inbox)
/// - Total effective credits across all roots
///
/// Each balance item includes:
/// - Target percent (user's desired allocation)
/// - Actual percent (computed from effective credits)
/// - Starving flag (true if under-served)
pub fn use_balance_data() -> Memo<BalanceData> {
    let tunnel_state = crate::hooks::use_tunnel_state::use_tunnel_state();

    use_memo(move || {
        let state = tunnel_state.read();
        get_balance_data(&state)
    })
}

#[cfg(test)]
/// Note on Testing Strategy: This test verifies the hook's integration contract
/// (context injection + reactivity). While valid, the setup cost (mocking
/// `TunnelState`) is high relative to the implementation complexity. We keep
/// this here as a reference pattern for testing hooks with context, but
/// strictly unit-testing every trivial hook is not a mandated best practice in
/// this repo if the maintenance friction is high.
mod tests {
    use super::*;
    use tasklens_core::types::{
        PersistedTask, Schedule, ScheduleType, TaskID, TaskStatus, TunnelState,
    };

    use std::collections::HashMap;

    /// Helper to create a basic validated task for testing
    fn make_test_task(id: &str, title: &str, desired_credits: f64) -> PersistedTask {
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
            credits: 0.0,
            desired_credits,
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
        }
    }

    #[test]
    fn test_use_balance_data_integrates_state() {
        // 1. Setup the VirtualDom
        let mut dom = VirtualDom::new(|| {
            // 2. Mock the Context
            // We need to provide Memo<TunnelState> because use_tunnel_state looks for it first
            let mut tasks = HashMap::new();
            tasks.insert(TaskID::from("1"), make_test_task("1", "Root", 100.0));

            let mock_state = TunnelState {
                tasks,
                root_task_ids: vec![TaskID::from("1")],
                ..Default::default()
            };

            // We are inside the root component, so we can use hooks.
            let state_signal = use_signal(|| mock_state);
            let memo = use_memo(move || state_signal.read().clone());
            use_context_provider(|| memo);

            // 3. Render a component that uses the hook
            let balance_data = use_balance_data();

            // 4. Capture results (we can't easily assert inside the hook return, so we render strictly for side-effects or check expected output)
            // But wait, to check the VALUE of the hook, we usually need to extract it or print it.
            // For unit testing hooks, it's often easier to just panic if assumptions fail inside the component.

            let data = balance_data.read();
            if data.items.len() != 1 {
                panic!("Expected 1 item, got {}", data.items.len());
            }
            if data.items[0].title != "Root" {
                panic!("Expected title 'Root', got '{}'", data.items[0].title);
            }
            if data.items[0].target_percent != 1.0 {
                panic!("Expected target 1.0, got {}", data.items[0].target_percent);
            }

            rsx! {
                div {}

            }
        });

        // 5. Run the DOM
        dom.rebuild_in_place();
    }
}

use dioxus::prelude::*;
use std::collections::HashMap;
use tasklens_core::domain::priority::get_prioritized_tasks;
use tasklens_core::types::{
    ComputedTask, PriorityMode, PriorityOptions, TaskID, TunnelState, ViewFilter,
};

/// Schedule info (effective_due_date, effective_lead_time) keyed by TaskID.
pub type ScheduleLookup = HashMap<TaskID, (Option<i64>, Option<i64>)>;

fn compute_prioritized_tasks(state: &TunnelState, include_hidden: bool) -> Vec<ComputedTask> {
    let view_filter = ViewFilter {
        place_id: Some("All".to_string()),
    };
    let options = PriorityOptions {
        include_hidden,
        mode: Some(PriorityMode::DoList),
        context: None,
    };
    get_prioritized_tasks(state, &view_filter, &options)
}

fn build_schedule_lookup(tasks: Vec<ComputedTask>) -> ScheduleLookup {
    tasks
        .into_iter()
        .map(|t| (t.id.clone(), (t.effective_due_date, t.effective_lead_time)))
        .collect()
}

/// Returns prioritized tasks for Do view (visible tasks only, sorted by priority).
pub fn use_do_list_tasks() -> Memo<Vec<ComputedTask>> {
    let state = crate::hooks::use_tunnel_state::use_tunnel_state();
    use_memo(move || compute_prioritized_tasks(&state.read(), false))
}

/// Returns a schedule lookup map for Plan view (all tasks, including hidden).
/// This provides effective_due_date and effective_lead_time from the core algorithm.
pub fn use_schedule_lookup() -> Memo<ScheduleLookup> {
    let state = crate::hooks::use_tunnel_state::use_tunnel_state();
    use_memo(move || build_schedule_lookup(compute_prioritized_tasks(&state.read(), true)))
}

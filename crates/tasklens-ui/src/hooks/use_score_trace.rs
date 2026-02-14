//! Score trace hook for the Do view.

use dioxus::prelude::*;
use tasklens_core::domain::priority::get_score_trace;
use tasklens_core::types::{PriorityMode, PriorityOptions, ScoreTrace, TaskID, ViewFilter};
use tracing::warn;

/// Computes the score trace for a task using Do view scoring rules.
/// The trace is derived on demand to avoid persisting debug data.
/// Uses the "All" place filter to match the Do list scoring default.
pub(crate) fn use_score_trace(task_id: TaskID) -> Memo<Option<ScoreTrace>> {
    let state = crate::hooks::use_tunnel_state::use_tunnel_state();

    use_memo(move || {
        let view_filter = ViewFilter {
            place_id: Some("All".to_string()),
        };
        let options = PriorityOptions {
            include_hidden: false,
            mode: Some(PriorityMode::DoList),
            context: None,
        };
        let result = get_score_trace(&state.read(), &view_filter, &options, &task_id);
        if result.is_none() {
            warn!("Score trace not found for task: {}", task_id);
        }
        result
    })
}

use automerge::transaction::Transactable;
use autosurgeon::{Hydrate, HydrateError, ReadDoc, ReconcileError};

use crate::types::TunnelState;
use crate::types::{hydrate_i64, hydrate_optional_metadata};

/// Hydrates a TunnelState manually from an Automerge document.
///
/// This provides a single source of truth for the document structure and
/// ensures that custom hydrators (like hydrate_f64) are used consistently.
#[tracing::instrument(skip_all)]
pub fn hydrate_tunnel_state(doc: &impl ReadDoc) -> Result<TunnelState, HydrateError> {
    let root = automerge::ROOT;

    let next_task_id = hydrate_i64(doc, &root, "nextTaskId".into())?;
    let next_place_id = hydrate_i64(doc, &root, "nextPlaceId".into())?;

    let tasks = Hydrate::hydrate(doc, &root, "tasks".into())?;
    let root_task_ids = Hydrate::hydrate(doc, &root, "rootTaskIds".into())?;
    let places = Hydrate::hydrate(doc, &root, "places".into())?;
    let metadata = hydrate_optional_metadata(doc, &root, "metadata".into())?;

    Ok(TunnelState {
        next_task_id,
        next_place_id,
        tasks,
        root_task_ids,
        places,
        metadata,
    })
}

/// Reconciles a TunnelState manually into an Automerge document.
pub fn reconcile_tunnel_state<T: Transactable + autosurgeon::Doc>(
    doc: &mut T,
    state: &TunnelState,
) -> Result<(), ReconcileError> {
    // nextTaskId
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "nextTaskId", state.next_task_id)?;

    // nextPlaceId
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "nextPlaceId", state.next_place_id)?;

    // tasks
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "tasks", &state.tasks)?;

    // rootTaskIds
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "rootTaskIds", &state.root_task_ids)?;

    // places
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "places", &state.places)?;

    // metadata
    use autosurgeon::MaybeMissing;
    let metadata_mm = match state.metadata.as_ref() {
        Some(m) => MaybeMissing::Present(m),
        None => MaybeMissing::Missing,
    };
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "metadata", metadata_mm)?;

    Ok(())
}

/// Field names for PersistedTask in the Automerge document.
pub mod task_fields {
    pub const STATUS: &str = "status";
    pub const ID: &str = "id";
    pub const TITLE: &str = "title";
    pub const NOTES: &str = "notes";
    pub const PARENT_ID: &str = "parentId";
    pub const CHILD_TASK_IDS: &str = "childTaskIds";
    pub const PLACE_ID: &str = "placeId";
    pub const IMPORTANCE: &str = "importance";
    pub const CREDIT_INCREMENT: &str = "creditIncrement";
    pub const CREDITS: &str = "credits";
    pub const DESIRED_CREDITS: &str = "desiredCredits";
    pub const CREDITS_TIMESTAMP: &str = "creditsTimestamp";
    pub const PRIORITY_TIMESTAMP: &str = "priorityTimestamp";
    pub const SCHEDULE: &str = "schedule";
    pub const REPEAT_CONFIG: &str = "repeatConfig";
    pub const IS_SEQUENTIAL: &str = "isSequential";
    pub const IS_ACKNOWLEDGED: &str = "isAcknowledged";
    pub const LAST_COMPLETED_AT: &str = "lastCompletedAt";
}

/// Field names for Schedule in the Automerge document.
pub mod schedule_fields {
    pub const TYPE: &str = "type";
    pub const DUE_DATE: &str = "dueDate";
    pub const LEAD_TIME: &str = "leadTime";
    pub const LAST_DONE: &str = "lastDone";
}

use automerge::transaction::Transactable;
use autosurgeon::{Hydrate, HydrateError, ReadDoc, Reconcile, ReconcileError, Reconciler};

use crate::types::{
    TunnelState, hydrate_i64, hydrate_optional_metadata, reconcile_i64,
    reconcile_optional_as_maybe_missing,
};

/// Hydrates a TunnelState manually from an Automerge document.
///
/// This provides a single source of truth for the document structure and
/// ensures that custom hydrators (like hydrate_f64) are used consistently.
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
    reconcile_i64_prop(doc, &automerge::ROOT, "nextTaskId", state.next_task_id)?;

    // nextPlaceId
    reconcile_i64_prop(doc, &automerge::ROOT, "nextPlaceId", state.next_place_id)?;

    // tasks
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "tasks", &state.tasks)?;

    // rootTaskIds
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "rootTaskIds", &state.root_task_ids)?;

    // places
    autosurgeon::reconcile_prop(doc, &automerge::ROOT, "places", &state.places)?;

    // metadata
    reconcile_optional_metadata_prop(doc, &automerge::ROOT, "metadata", &state.metadata)?;

    Ok(())
}

fn reconcile_i64_prop<T: Transactable + autosurgeon::Doc>(
    doc: &mut T,
    obj: &automerge::ObjId,
    prop: &str,
    val: i64,
) -> Result<(), ReconcileError> {
    struct JSI64(i64);
    impl Reconcile for JSI64 {
        type Key<'a> = autosurgeon::reconcile::NoKey;
        fn reconcile<R: Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
            reconcile_i64(&self.0, reconciler)
        }
    }
    autosurgeon::reconcile_prop(doc, obj, prop, JSI64(val))
}

fn reconcile_optional_metadata_prop<T: Transactable + autosurgeon::Doc>(
    doc: &mut T,
    obj: &automerge::ObjId,
    prop: &str,
    val: &Option<crate::types::DocMetadata>,
) -> Result<(), ReconcileError> {
    struct OptionalMetadata<'a>(&'a Option<crate::types::DocMetadata>);
    impl Reconcile for OptionalMetadata<'_> {
        type Key<'a> = autosurgeon::reconcile::NoKey;
        fn reconcile<R: Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
            reconcile_optional_as_maybe_missing(self.0, reconciler)
        }
    }
    autosurgeon::reconcile_prop(doc, obj, prop, OptionalMetadata(val))
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

// Re-index.
use std::collections::HashMap;
use thiserror::Error;

use crate::doc_id::TaskLensUrl;
use tasklens_core::{
    Action,
    types::{DocMetadata, TaskID, TunnelState},
};

pub use tasklens_core::domain::dispatch::{DispatchError, run_action};

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),

    #[error("Reconcile error: {0}")]
    Reconcile(#[from] autosurgeon::ReconcileError),

    #[error("Hydrate error: {0}")]
    Hydrate(#[from] autosurgeon::HydrateError),

    #[error("Hydration failed: {0}")]
    Hydration(String),

    #[error("Path key '{0}' is not an object")]
    InvalidPath(String),

    #[error("Task not found: {0}")]
    TaskNotFound(TaskID),

    #[error("Parent task not found: {0}")]
    ParentNotFound(TaskID),

    #[error("Task already exists: {0}")]
    TaskExists(TaskID),

    #[error("Cycle detected moving task {0} to {1}")]
    CycleDetected(TaskID, TaskID),

    #[error("Cannot move task {0} to itself: {1}")]
    MoveToSelf(TaskID, TaskID),

    #[error("Place not found: {0}")]
    PlaceNotFound(tasklens_core::types::PlaceID),

    #[error("Cannot delete the built-in Anywhere place")]
    CannotDeleteAnywhere,

    #[error("Inconsistency: {0}")]
    Inconsistency(String),

    #[error("Operation failed: {0}")]
    Internal(String),
}

impl From<DispatchError> for AdapterError {
    fn from(e: DispatchError) -> Self {
        match e {
            DispatchError::Automerge(e) => AdapterError::Automerge(e),
            DispatchError::Reconcile(e) => AdapterError::Reconcile(e),
            DispatchError::Hydrate(e) => AdapterError::Hydrate(e),
            DispatchError::Hydration(s) => AdapterError::Hydration(s),
            DispatchError::InvalidPath(s) => AdapterError::InvalidPath(s),
            DispatchError::TaskNotFound(id) => AdapterError::TaskNotFound(id),
            DispatchError::ParentNotFound(id) => AdapterError::ParentNotFound(id),
            DispatchError::TaskExists(id) => AdapterError::TaskExists(id),
            DispatchError::CycleDetected(id1, id2) => AdapterError::CycleDetected(id1, id2),
            DispatchError::MoveToSelf(id1, id2) => AdapterError::MoveToSelf(id1, id2),
            DispatchError::PlaceNotFound(id) => AdapterError::PlaceNotFound(id),
            DispatchError::CannotDeleteAnywhere => AdapterError::CannotDeleteAnywhere,
            DispatchError::Inconsistency(s) => AdapterError::Inconsistency(s),
            DispatchError::Internal(s) => AdapterError::Internal(s),
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, AdapterError>;

/// Hydrates a TunnelState from the current document, healing any structural inconsistencies.
pub(crate) fn hydrate_tunnel_state_and_heal(
    doc: &impl autosurgeon::ReadDoc,
) -> Result<TunnelState> {
    tasklens_core::domain::doc_bridge::hydrate_tunnel_state(doc)
        .map(|mut state| {
            state.heal_structural_inconsistencies();
            state
        })
        .map_err(AdapterError::from)
}

pub(crate) fn init_state(
    doc: &mut automerge::Automerge,
    id: &crate::doc_id::DocumentId,
) -> Result<()> {
    let mut tx = doc.transaction();

    let initial_state = TunnelState {
        tasks: HashMap::new(),
        places: HashMap::new(),
        root_task_ids: Vec::new(),
        metadata: Some(DocMetadata {
            automerge_url: Some(TaskLensUrl::from(id).to_string()),
        }),
    };

    if let Err(e) =
        tasklens_core::domain::doc_bridge::reconcile_tunnel_state(&mut tx, &initial_state)
    {
        tracing::error!("Failed to reconcile initial state: {}", e);
        return Err(AdapterError::Internal(format!(
            "Failed to reconcile initial state: {}",
            e
        )));
    }
    tx.commit();
    Ok(())
}

/// Reconciles a Rust struct with the current document.
/// Specializes for TunnelState using manual bridge logic.
pub(crate) fn expensive_reconcile<T: autosurgeon::Reconcile + 'static>(
    doc: &mut automerge::Automerge,
    data: &T,
) -> Result<()> {
    let mut tx = doc.transaction();

    // Use manual bridge if T is TunnelState
    if let Some(state) = (data as &dyn std::any::Any).downcast_ref::<TunnelState>() {
        tasklens_core::domain::doc_bridge::reconcile_tunnel_state(&mut tx, state)?;
    } else {
        autosurgeon::reconcile(&mut tx, data)?;
    }

    tx.commit();
    Ok(())
}

/// Dispatches an action to modify the application state.
pub(crate) fn dispatch(doc: &mut automerge::Automerge, action: Action) -> Result<()> {
    let mut tx = doc.transaction();
    let res = run_action(&mut tx, action);
    tx.commit();
    res.map_err(AdapterError::from)
}

#[cfg(test)]
mod tests;

//! Transient types for TaskLens runtime logic.
//!
//! These types are used for in-memory processing, algorithm traces, and view projections.
//! They are NOT directly persisted to the Automerge document schema, although they may
//! be serialized for transient storage or debugging.

use super::persistent::{
    PlaceID, RepeatConfig, Schedule, TaskID, TaskStatus, UrgencyStatus, hydrate_i64,
    hydrate_option_maybe_missing, reconcile_option_as_maybe_missing,
};
use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Internal Mutable Object for Algorithm Processing.
#[derive(Debug, Clone, PartialEq)]
pub struct EnrichedTask {
    // Flattened PersistedTask fields
    pub id: TaskID,
    pub title: String,
    pub notes: String,
    pub parent_id: Option<TaskID>,
    pub child_task_ids: Vec<TaskID>,
    pub place_id: Option<PlaceID>,
    pub status: TaskStatus,
    pub importance: f64,
    pub credit_increment: Option<f64>,
    pub credits: f64,
    pub desired_credits: f64,
    pub credits_timestamp: i64,
    pub priority_timestamp: i64,
    pub schedule: Schedule,
    pub repeat_config: Option<RepeatConfig>,
    pub is_sequential: bool,
    pub is_acknowledged: bool,
    pub last_completed_at: Option<i64>,

    // Ephemeral scratchpad values
    pub effective_credits: f64,
    pub feedback_factor: f64,
    pub lead_time_factor: f64,
    pub normalized_importance: f64,
    pub priority: f64,
    pub visibility: bool,
    pub outline_index: f64,
    pub is_container: bool,
    pub is_pending: bool,
    pub is_ready: bool,

    // Effective Schedule State (Inheritance)
    pub effective_due_date: Option<i64>,
    pub effective_lead_time: Option<i64>,
    pub effective_schedule_source: Option<ScheduleSource>,
}

/// Indicates where the effective schedule came from.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ScheduleSource {
    /// The schedule is defined on the task itself.
    #[serde(rename = "self")]
    Myself,
    /// The schedule is inherited from an ancestor.
    Ancestor,
}

impl Hydrate for ScheduleSource {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let s = String::hydrate(doc, obj, prop)?;
        match s.as_str() {
            "Myself" => Ok(Self::Myself),
            "Ancestor" => Ok(Self::Ancestor),
            _ => Err(autosurgeon::HydrateError::unexpected(
                "ScheduleSource variant",
                s,
            )),
        }
    }
}

impl Reconcile for ScheduleSource {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        match self {
            Self::Myself => "Myself",
            Self::Ancestor => "Ancestor",
        }
        .reconcile(reconciler)
    }
}

/// A task as projected for the View Layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComputedTask {
    pub id: TaskID,
    pub title: String,
    pub notes: String,
    pub parent_id: Option<TaskID>,
    pub child_task_ids: Vec<TaskID>,
    pub place_id: Option<PlaceID>,
    pub status: TaskStatus,
    pub importance: f64,
    pub credit_increment: Option<f64>,
    pub credits: f64,
    pub effective_credits: f64,
    pub desired_credits: f64,
    pub credits_timestamp: i64,
    pub priority_timestamp: i64,
    pub schedule: Schedule,
    pub repeat_config: Option<RepeatConfig>,
    pub is_sequential: bool,
    pub is_acknowledged: bool,
    pub last_completed_at: Option<i64>,
    // TODO: Remove - logic detail hidden from view layer
    pub score: f64,
    // TODO: Remove - logic detail hidden from view layer
    pub normalized_importance: f64,
    // TODO: Remove - logic detail hidden from view layer
    pub is_blocked: bool,
    // TODO: Remove - logic detail hidden from view layer
    pub is_visible: bool,
    // TODO: Remove - UI state, not domain state
    pub is_open: bool,
    pub is_container: bool,
    pub is_pending: bool,
    pub is_ready: bool,
    pub effective_due_date: Option<i64>,
    pub effective_lead_time: Option<i64>,
    pub effective_schedule_source: Option<ScheduleSource>,
    // TODO: Remove - computed in UI component in TS
    pub urgency_status: UrgencyStatus,
}

/// A full breakdown of how a task's score was computed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScoreTrace {
    /// Task identifier for the trace.
    pub task_id: TaskID,
    /// Task title at the time of calculation.
    pub task_title: String,
    /// Final score used for sorting.
    pub score: f64,
    /// Timestamp when the score was computed (ms since epoch).
    pub computed_at: i64,
    /// Top-level factor breakdown used by the scoring formula.
    pub factors: ScoreFactors,
    /// Importance propagation details from root to the task.
    pub importance_chain: Vec<ImportanceTrace>,
    /// Root-level balance feedback details.
    pub feedback: FeedbackTrace,
    /// Lead time readiness details.
    pub lead_time: LeadTimeTrace,
    /// Contextual and delegation visibility details.
    pub visibility: VisibilityTrace,
}

/// The multiplicative factors used by the score formula.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScoreFactors {
    /// 1.0 when visible, 0.0 when hidden.
    pub visibility_factor: f64,
    /// Normalized importance derived from the task hierarchy.
    pub normalized_importance: f64,
    /// Root balance feedback factor.
    pub feedback_factor: f64,
    /// Lead time ramp factor.
    pub lead_time_factor: f64,
}

/// A single step in the importance propagation chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportanceTrace {
    /// Task identifier for this chain entry.
    pub task_id: TaskID,
    /// Task title for this chain entry.
    pub task_title: String,
    /// Raw importance value on the task.
    pub importance: f64,
    /// Parent normalized importance (if applicable).
    pub parent_normalized_importance: Option<f64>,
    /// Normalized importance assigned to this task.
    pub normalized_importance: f64,
    /// Whether this task was blocked by sequential ordering.
    pub sequential_blocked: bool,
}

/// Root balance feedback calculation details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackTrace {
    /// Root task identifier used for feedback.
    pub root_id: TaskID,
    /// Root task title.
    pub root_title: String,
    /// Root desired credits.
    pub desired_credits: f64,
    /// Root effective credits after decay and aggregation.
    pub effective_credits: f64,
    /// Total desired credits across roots.
    pub total_desired_credits: f64,
    /// Total effective credits across roots.
    pub total_effective_credits: f64,
    /// Target percentage of desired credits.
    pub target_percent: f64,
    /// Actual percentage of effective credits.
    pub actual_percent: f64,
    /// Deviation ratio used for the feedback factor (after capping).
    pub deviation_ratio: f64,
    /// Sensitivity value used in the power calculation.
    pub sensitivity: f64,
    /// Epsilon used to protect against division by zero.
    pub epsilon: f64,
    /// Final feedback factor applied to the score.
    pub feedback_factor: f64,
}

/// The stage of the lead time ramp used for readiness.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LeadTimeStage {
    /// The task is too early to be surfaced.
    TooEarly,
    /// The task is in the ramp-up window.
    Ramping,
    /// The task is ready (fully ramped).
    Ready,
    /// The task is past due.
    Overdue,
}

/// Lead time details used to compute readiness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LeadTimeTrace {
    /// Effective due date after inheritance (ms since epoch).
    pub effective_due_date: Option<i64>,
    /// Effective lead time in milliseconds.
    pub effective_lead_time: i64,
    /// Remaining time until due date (ms), if applicable.
    pub time_remaining: Option<i64>,
    /// Lead time stage used for the ramp calculation.
    pub stage: LeadTimeStage,
    /// Final lead time factor used in the score.
    pub factor: f64,
    /// Where the effective schedule was inherited from.
    pub schedule_source: Option<ScheduleSource>,
}

/// Contextual visibility inputs that gate the score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextualVisibilityTrace {
    /// Effective place id used for contextual checks.
    pub effective_place_id: PlaceID,
    /// Whether the place is currently open.
    pub is_open: bool,
    /// Whether the task matches the active place filter.
    pub filter_match: bool,
    /// Whether the task is already acknowledged.
    pub is_acknowledged: bool,
    /// Final contextual visibility result.
    pub passed: bool,
    /// Raw view filter place id (if any).
    pub view_filter_place_id: Option<String>,
}

/// Visibility details including delegation logic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityTrace {
    /// Contextual visibility gate results.
    pub contextual: ContextualVisibilityTrace,
    /// Whether the task has pending descendants.
    pub has_pending_descendants: bool,
    /// Whether visibility was delegated to descendants.
    pub delegated_to_descendants: bool,
    /// Final visibility value used in scoring.
    pub final_visibility: bool,
}

/// Runtime context for algorithm calculations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub current_place_id: Option<PlaceID>,
    #[autosurgeon(hydrate = "hydrate_i64")]
    pub current_time: i64,
}

/// Options to control which tasks are included in the prioritized output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Hydrate, Reconcile)]
#[serde(rename_all = "camelCase")]
pub struct PriorityOptions {
    pub include_hidden: bool,
    #[autosurgeon(
        hydrate = "hydrate_option_maybe_missing",
        reconcile = "reconcile_option_as_maybe_missing"
    )]
    pub mode: Option<PriorityMode>,
    #[autosurgeon(
        hydrate = "hydrate_option_maybe_missing",
        reconcile = "reconcile_option_as_maybe_missing"
    )]
    pub context: Option<Context>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PriorityMode {
    DoList,
    PlanOutline,
}

impl Hydrate for PriorityMode {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let s = String::hydrate(doc, obj, prop)?;
        match s.as_str() {
            "DoList" => Ok(Self::DoList),
            "PlanOutline" => Ok(Self::PlanOutline),
            _ => Err(autosurgeon::HydrateError::unexpected(
                "PriorityMode variant",
                s,
            )),
        }
    }
}

impl Reconcile for PriorityMode {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        match self {
            Self::DoList => "DoList",
            Self::PlanOutline => "PlanOutline",
        }
        .reconcile(reconciler)
    }
}

/// Filter criteria for the view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Hydrate, Reconcile)]
#[serde(rename_all = "camelCase")]
pub struct ViewFilter {
    #[autosurgeon(
        hydrate = "hydrate_option_maybe_missing",
        reconcile = "reconcile_option_as_maybe_missing"
    )]
    pub place_id: Option<String>, // "All", "Anywhere", or a specific ID
}

/// A single item in the Balance View, representing a top-level goal (TLI).
///
/// This is a projection type computed from `EnrichedTask` data for root tasks only.
/// It provides the view layer with pre-computed percentages for display.
#[derive(Debug, Clone, PartialEq)]
pub struct BalanceItem {
    /// The task ID of the root goal.
    pub id: TaskID,
    /// The display title of the goal.
    pub title: String,
    /// User's desired allocation as a percentage (0.0 - 1.0).
    /// Computed as: task.desired_credits / sum(all_roots.desired_credits)
    pub target_percent: f64,
    /// Actual effort allocation as a percentage (0.0 - 1.0).
    /// Computed as: task.effective_credits / sum(all_roots.effective_credits)
    pub actual_percent: f64,
    /// True if actual_percent < target_percent (goal is under-served).
    pub is_starving: bool,
    /// The raw desired_credits value for editing.
    pub desired_credits: f64,
    /// The raw effective_credits value for display.
    pub effective_credits: f64,
    /// Optional preview percentage for redistribution UI.
    pub preview_percent: Option<f64>,
}

/// Result of projecting balance data from enriched tasks.
#[derive(Debug, Clone, PartialEq)]
pub struct BalanceData {
    /// List of balance items, one per root goal (excluding Inbox).
    pub items: Vec<BalanceItem>,
    /// Sum of all effective_credits across root goals.
    pub total_credits: f64,
}

impl BalanceData {
    /// Returns a map of current target percentages for redistribution.
    pub fn get_percentage_map(&self) -> HashMap<TaskID, f64> {
        self.items
            .iter()
            .map(|i| (i.id.clone(), i.target_percent))
            .collect()
    }

    /// Merges current balance data with a preview percentage map.
    pub fn apply_previews(&self, previews: &Option<HashMap<TaskID, f64>>) -> Vec<BalanceItem> {
        self.items
            .iter()
            .map(|item| {
                let preview_percent = previews.as_ref().and_then(|m| m.get(&item.id).copied());
                let mut new_item = item.clone();
                new_item.preview_percent = preview_percent;
                new_item
            })
            .collect()
    }
}

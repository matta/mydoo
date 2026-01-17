//! Core domain types for TaskLens.
//!
//! These types are designed to be compatible with the TypeScript
//! `TaskSchema` for Automerge document interchange. They use camelCase
//! JSON serialization to match the existing TypeScript schema.

use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};
pub use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(transparent)]
pub struct TaskID(String);

impl Hydrate for TaskID {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        String::hydrate(doc, obj, prop).map(Self)
    }
}

impl Reconcile for TaskID {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        self.0.reconcile(reconciler)
    }
}

impl TaskID {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TaskID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for TaskID {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for TaskID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::str::FromStr for TaskID {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl From<&str> for TaskID {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for TaskID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a place/context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(transparent)]
pub struct PlaceID(String);

impl Hydrate for PlaceID {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        String::hydrate(doc, obj, prop).map(Self)
    }
}

impl Reconcile for PlaceID {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        self.0.reconcile(reconciler)
    }
}

impl PlaceID {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PlaceID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for PlaceID {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for PlaceID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::str::FromStr for PlaceID {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl From<&str> for PlaceID {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for PlaceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reserved Place ID representing "any location".
pub const ANYWHERE_PLACE_ID: &str = "Anywhere";

/// The completion status of a task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
pub enum TaskStatus {
    /// Task is not yet completed.
    Pending,
    /// Task has been completed.
    Done,
}

impl Hydrate for TaskStatus {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let s = String::hydrate(doc, obj, prop)?;
        match s.as_str() {
            "Pending" => Ok(Self::Pending),
            "Done" => Ok(Self::Done),
            _ => Err(autosurgeon::HydrateError::unexpected(
                "TaskStatus variant",
                s,
            )),
        }
    }
}

impl Reconcile for TaskStatus {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        match self {
            Self::Pending => "Pending",
            Self::Done => "Done",
        }
        .reconcile(reconciler)
    }
}

/// The urgency status of a task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
pub enum UrgencyStatus {
    Overdue,
    Urgent,
    Active,
    Upcoming,
    None,
}

impl Hydrate for UrgencyStatus {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let s = String::hydrate(doc, obj, prop)?;
        match s.as_str() {
            "Overdue" => Ok(Self::Overdue),
            "Urgent" => Ok(Self::Urgent),
            "Active" => Ok(Self::Active),
            "Upcoming" => Ok(Self::Upcoming),
            "None" => Ok(Self::None),
            _ => Err(autosurgeon::HydrateError::unexpected(
                "UrgencyStatus variant",
                s,
            )),
        }
    }
}

impl Reconcile for UrgencyStatus {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        match self {
            Self::Overdue => "Overdue",
            Self::Urgent => "Urgent",
            Self::Active => "Active",
            Self::Upcoming => "Upcoming",
            Self::None => "None",
        }
        .reconcile(reconciler)
    }
}

/// The scheduling strategy for a task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
pub enum ScheduleType {
    /// A one-time task with no recurrence.
    Once,
    /// A recurring task based on interval since last completion.
    Routinely,
    /// A task with a specific due date.
    DueDate,
    /// A calendar-based scheduled task.
    Calendar,
}

impl Hydrate for ScheduleType {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let s = String::hydrate(doc, obj, prop)?;
        match s.as_str() {
            "Once" => Ok(Self::Once),
            "Routinely" => Ok(Self::Routinely),
            "DueDate" => Ok(Self::DueDate),
            "Calendar" => Ok(Self::Calendar),
            _ => Err(autosurgeon::HydrateError::unexpected(
                "ScheduleType variant",
                s,
            )),
        }
    }
}

impl Reconcile for ScheduleType {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        match self {
            Self::Once => "Once",
            Self::Routinely => "Routinely",
            Self::DueDate => "DueDate",
            Self::Calendar => "Calendar",
        }
        .reconcile(reconciler)
    }
}

/// Scheduling configuration for a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    /// The type of schedule (Once, Routinely, DueDate, Calendar).
    #[serde(rename = "type")]
    #[autosurgeon(rename = "type")]
    pub schedule_type: ScheduleType,
    /// Optional due date as Unix timestamp in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(rename = "dueDate")]
    pub due_date: Option<f64>,
    /// Lead time in milliseconds before due date to start showing urgency.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(rename = "leadTime")]
    pub lead_time: Option<f64>,
    /// Timestamp of last completion (for Routinely tasks).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(rename = "lastDone")]
    pub last_done: Option<f64>,
}

/// Frequency unit for recurring tasks.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "lowercase")]
pub enum Frequency {
    Minutes,
    Hours,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Hydrate for Frequency {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let s = String::hydrate(doc, obj, prop)?;
        match s.to_lowercase().as_str() {
            "minutes" => Ok(Self::Minutes),
            "hours" => Ok(Self::Hours),
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "yearly" => Ok(Self::Yearly),
            _ => Err(autosurgeon::HydrateError::unexpected(
                "Frequency variant",
                s,
            )),
        }
    }
}

impl Reconcile for Frequency {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        match self {
            Self::Minutes => "Minutes",
            Self::Hours => "Hours",
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Yearly => "Yearly",
        }
        .reconcile(reconciler)
    }
}

/// Configuration for task repetition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
pub struct RepeatConfig {
    /// The unit of frequency (daily, weekly, etc.).
    pub frequency: Frequency,
    /// The interval multiplier (e.g., 2 for "every 2 weeks").
    pub interval: f64,
}

/// A task as persisted in the Automerge document.
///
/// Uses `extra_fields` with `#[serde(flatten)]` to preserve any
/// unknown fields during roundtrip serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct PersistedTask {
    pub status: TaskStatus,
    pub id: TaskID,
    pub title: String,
    #[serde(default)]
    pub notes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(rename = "parentId")]
    #[cfg_attr(any(test, feature = "test-utils"), proptest(value = "None"))]
    pub parent_id: Option<TaskID>,
    #[autosurgeon(rename = "childTaskIds")]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_vec_task_id()")
    )]
    pub child_task_ids: Vec<TaskID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(rename = "placeId")]
    #[cfg_attr(any(test, feature = "test-utils"), proptest(value = "None"))]
    pub place_id: Option<PlaceID>,
    pub importance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(rename = "creditIncrement")]
    pub credit_increment: Option<f64>,
    pub credits: f64,
    #[autosurgeon(rename = "desiredCredits")]
    pub desired_credits: f64,
    #[autosurgeon(rename = "creditsTimestamp")]
    pub credits_timestamp: f64,
    #[autosurgeon(rename = "priorityTimestamp")]
    pub priority_timestamp: f64,
    pub schedule: Schedule,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(rename = "repeatConfig")]
    pub repeat_config: Option<RepeatConfig>,
    #[autosurgeon(rename = "isSequential")]
    pub is_sequential: bool,
    #[serde(default)]
    #[autosurgeon(rename = "isAcknowledged")]
    pub is_acknowledged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(rename = "lastCompletedAt")]
    pub last_completed_at: Option<f64>,
}

impl Hydrate for PersistedTask {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let task_obj = match prop {
            autosurgeon::Prop::Key(ref k) => doc
                .get(obj, k.as_ref())
                .map_err(|e| autosurgeon::HydrateError::unexpected("Object", e.to_string()))?
                .and_then(|(v, o)| {
                    if matches!(v, automerge::Value::Object(automerge::ObjType::Map)) {
                        Some(o)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    autosurgeon::HydrateError::unexpected(
                        "Object",
                        "Missing or not a map".to_string(),
                    )
                })?,
            autosurgeon::Prop::Index(i) => doc
                .get(obj, i as usize)
                .map_err(|e| autosurgeon::HydrateError::unexpected("Object", e.to_string()))?
                .and_then(|(v, o)| {
                    if matches!(v, automerge::Value::Object(automerge::ObjType::Map)) {
                        Some(o)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    autosurgeon::HydrateError::unexpected(
                        "Object",
                        "Missing or not a map".to_string(),
                    )
                })?,
        };

        Ok(Self {
            status: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("status")),
            )?,
            id: Hydrate::hydrate(doc, &task_obj, autosurgeon::Prop::Key(Cow::Borrowed("id")))?,
            title: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("title")),
            )?,
            notes: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("notes")),
            )
            .unwrap_or_default(),
            parent_id: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("parentId")),
            )
            .ok(),
            child_task_ids: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("childTaskIds")),
            )
            .unwrap_or_default(),
            place_id: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("placeId")),
            )
            .ok(),
            importance: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("importance")),
            )?,
            credit_increment: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("creditIncrement")),
            )
            .ok(),
            credits: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("credits")),
            )?,
            desired_credits: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("desiredCredits")),
            )?,
            credits_timestamp: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("creditsTimestamp")),
            )?,
            priority_timestamp: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("priorityTimestamp")),
            )?,
            schedule: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("schedule")),
            )?,
            repeat_config: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("repeatConfig")),
            )
            .ok(),
            is_sequential: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("isSequential")),
            )
            .unwrap_or(false),
            is_acknowledged: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("isAcknowledged")),
            )
            .unwrap_or(false),
            last_completed_at: Hydrate::hydrate(
                doc,
                &task_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("lastCompletedAt")),
            )
            .ok(),
        })
    }
}

/// Internal Mutable Object for Algorithm Processing.
#[derive(Debug, Clone, PartialEq, Hydrate, Reconcile)]
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
    pub credits_timestamp: f64,
    pub priority_timestamp: f64,
    pub schedule: Schedule,
    pub repeat_config: Option<RepeatConfig>,
    pub is_sequential: bool,
    pub is_acknowledged: bool,
    pub last_completed_at: Option<f64>,

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
    pub effective_due_date: Option<f64>,
    pub effective_lead_time: Option<f64>,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
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
    pub credits_timestamp: f64,
    pub priority_timestamp: f64,
    pub schedule: Schedule,
    pub repeat_config: Option<RepeatConfig>,
    pub is_sequential: bool,
    pub is_acknowledged: bool,
    pub last_completed_at: Option<f64>,
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
    pub effective_due_date: Option<f64>,
    pub effective_lead_time: Option<f64>,
    pub effective_schedule_source: Option<ScheduleSource>,
    // TODO: Remove - computed in UI component in TS
    pub urgency_status: UrgencyStatus,
}

/// Runtime context for algorithm calculations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub current_place_id: Option<PlaceID>,
    pub current_time: f64,
}

/// Options to control which tasks are included in the prioritized output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Hydrate, Reconcile)]
#[serde(rename_all = "camelCase")]
pub struct PriorityOptions {
    pub include_hidden: bool,
    pub mode: Option<PriorityMode>,
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
    pub place_id: Option<String>, // "All", "Anywhere", or a specific ID
}

/// Defines the operating hours for a place.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OpenHoursMode {
    AlwaysOpen,
    AlwaysClosed,
    Custom,
}

impl Hydrate for OpenHoursMode {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let s = String::hydrate(doc, obj, prop)?;
        match s.as_str() {
            "AlwaysOpen" => Ok(Self::AlwaysOpen),
            "AlwaysClosed" => Ok(Self::AlwaysClosed),
            "Custom" => Ok(Self::Custom),
            _ => Err(autosurgeon::HydrateError::unexpected(
                "OpenHoursMode variant",
                s,
            )),
        }
    }
}

impl Reconcile for OpenHoursMode {
    type Key<'a> = autosurgeon::reconcile::NoKey;
    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        match self {
            Self::AlwaysOpen => "AlwaysOpen",
            Self::AlwaysClosed => "AlwaysClosed",
            Self::Custom => "Custom",
        }
        .reconcile(reconciler)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[serde(rename_all = "camelCase")]
pub struct OpenHours {
    pub mode: OpenHoursMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<HashMap<String, Vec<String>>>,
}

/// A place/context where tasks can be performed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub id: PlaceID,
    pub name: String,
    /// Stringified JSON of OpenHours
    pub hours: String,
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_vec_place_id()")
    )]
    #[autosurgeon(rename = "includedPlaces")]
    pub included_places: Vec<PlaceID>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
pub struct DocMetadata {
    #[autosurgeon(rename = "automerge_url")]
    pub automerge_url: Option<String>,
}

/// The root state of a TaskLens document.
///
/// This is the top-level structure serialized to/from Automerge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct TunnelState {
    #[autosurgeon(rename = "nextTaskId")]
    pub next_task_id: f64,
    #[autosurgeon(rename = "nextPlaceId")]
    pub next_place_id: f64,
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_tasks_map()")
    )]
    pub tasks: HashMap<TaskID, PersistedTask>,
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_vec_task_id()")
    )]
    #[autosurgeon(rename = "rootTaskIds")]
    pub root_task_ids: Vec<TaskID>,
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_places_map()")
    )]
    pub places: HashMap<PlaceID, Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DocMetadata>,
}

impl Default for TunnelState {
    fn default() -> Self {
        Self {
            next_task_id: 1.0,
            next_place_id: 1.0,
            tasks: HashMap::new(),
            places: HashMap::new(),
            root_task_ids: Vec::new(),
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_persisted_task_serialization() {
        let task = PersistedTask {
            status: TaskStatus::Pending,
            id: TaskID::from("task-1"),
            title: "Test Task".to_string(),
            notes: "Some notes".to_string(),
            parent_id: None,
            child_task_ids: vec![],
            place_id: None,
            importance: 1.0,
            credit_increment: None,
            credits: 0.0,
            desired_credits: 1.0,
            credits_timestamp: 12345678.0,
            priority_timestamp: 12345678.0,
            schedule: Schedule {
                schedule_type: ScheduleType::Once,
                due_date: None,
                lead_time: Some(0.0),
                last_done: None,
            },
            repeat_config: None,
            is_sequential: false,
            is_acknowledged: false,
            last_completed_at: None,
        };

        let serialized = serde_json::to_value(&task).unwrap();
        let expected = serde_json::json!({
            "status": "Pending",
            "id": "task-1",
            "title": "Test Task",
            "notes": "Some notes",
            "childTaskIds": [],
            "importance": 1.0,
            "credits": 0.0,
            "desiredCredits": 1.0,
            "creditsTimestamp": 12345678.0,
            "priorityTimestamp": 12345678.0,
            "schedule": {
                "type": "Once",
                "leadTime": 0.0
            },
            "isSequential": false,
            "isAcknowledged": false
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_tunnel_state_serialization_old() {
        // ... (truncated or kept as is)
    }
}

#[cfg(any(test, feature = "test-utils"))]
/// Property test strategies for generating arbitrary domain types.
///
/// These strategies are designed to generate valid, self-consistent
/// domain objects while avoiding orphan references (e.g., `parent_id`
/// pointing to non-existent tasks).
pub mod test_strategies {
    use super::*;
    use proptest::prelude::*;

    pub fn arbitrary_vec_task_id() -> impl Strategy<Value = Vec<TaskID>> {
        proptest::collection::vec(any::<TaskID>(), 0..3)
    }

    pub fn arbitrary_vec_place_id() -> impl Strategy<Value = Vec<PlaceID>> {
        proptest::collection::vec(any::<PlaceID>(), 0..3)
    }

    pub fn arbitrary_tasks_map() -> impl Strategy<Value = HashMap<TaskID, PersistedTask>> {
        proptest::collection::hash_map(any::<TaskID>(), any::<PersistedTask>(), 0..3)
    }

    pub fn arbitrary_places_map() -> impl Strategy<Value = HashMap<PlaceID, Place>> {
        proptest::collection::hash_map(any::<PlaceID>(), any::<Place>(), 0..3)
    }
}

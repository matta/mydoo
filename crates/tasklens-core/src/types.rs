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

pub fn hydrate_string_or_text<D: autosurgeon::ReadDoc>(
    doc: &D,
    obj: &automerge::ObjId,
    prop: autosurgeon::Prop<'_>,
) -> Result<String, autosurgeon::HydrateError> {
    let (val, id) = match prop {
        autosurgeon::Prop::Key(k) => doc.get(obj, k.as_ref()),
        autosurgeon::Prop::Index(i) => doc.get(obj, i as usize),
    }
    .map_err(|e| autosurgeon::HydrateError::unexpected("get", e.to_string()))?
    .ok_or_else(|| {
        autosurgeon::HydrateError::unexpected("string or text", "missing value".to_string())
    })?;

    match val {
        automerge::Value::Object(automerge::ObjType::Text) => doc
            .text(&id)
            .map_err(|e| autosurgeon::HydrateError::unexpected("read text", e.to_string())),
        automerge::Value::Scalar(scalar) => match scalar.as_ref() {
            automerge::ScalarValue::Str(s) => Ok(s.to_string()),
            _ => Err(autosurgeon::HydrateError::unexpected(
                "string",
                format!("found {:?}", scalar),
            )),
        },
        _ => Err(autosurgeon::HydrateError::unexpected(
            "string or text",
            format!("found {:?}", val),
        )),
    }
}

/// Hydrates an `Option<T>` while tolerating both missing keys and null values.
///
/// This provides a universal "optional" behavior for struct fields:
/// 1. If the property is missing from the document, returns `Ok(None)`.
/// 2. If the property is present but set to `null`, returns `Ok(None)`.
/// 3. If the property is present and non-null, delegates to `T::hydrate`.
///
/// Use this with the `#[autosurgeon(hydrate = "hydrate_option_maybe_missing")]`
/// attribute on `Option<T>` fields where you want to support both missing and null states.
pub fn hydrate_option_maybe_missing<D: autosurgeon::ReadDoc, T: Hydrate>(
    doc: &D,
    obj: &automerge::ObjId,
    prop: autosurgeon::Prop<'_>,
) -> Result<Option<T>, autosurgeon::HydrateError> {
    let val = match prop {
        autosurgeon::Prop::Key(ref k) => doc.get(obj, k.as_ref()),
        autosurgeon::Prop::Index(i) => doc.get(obj, i as usize),
    }
    .map_err(|e| autosurgeon::HydrateError::unexpected("get", e.to_string()))?;

    match val {
        None => Ok(None),
        Some((automerge::Value::Scalar(s), _)) if s.as_ref().is_null() => Ok(None),
        Some(_) => T::hydrate(doc, obj, prop).map(Some),
    }
}

/// Hydrates an `Option<f64>` while tolerating various Automerge numeric types.
///
/// This follows the "Tolerant Hydration" principle: it accepts `Int`, `Uint`, and `F64`
/// from Automerge and converts them to `Option<f64>`. This is essential for
/// interoperability with JavaScript, which uses double-precision floats for all numbers.
///
/// If the value is missing or not a number, it returns `Ok(None)`.
pub fn hydrate_optional_f64<D: autosurgeon::ReadDoc>(
    doc: &D,
    obj: &automerge::ObjId,
    prop: autosurgeon::Prop<'_>,
) -> Result<Option<f64>, autosurgeon::HydrateError> {
    let val = match prop {
        autosurgeon::Prop::Key(ref k) => doc.get(obj, k.as_ref()),
        autosurgeon::Prop::Index(i) => doc.get(obj, i as usize),
    }
    .map_err(|e| autosurgeon::HydrateError::unexpected("get", e.to_string()))?;

    match val {
        Some((automerge::Value::Scalar(scalar), _)) => match scalar.as_ref() {
            automerge::ScalarValue::F64(f) => Ok(Some(*f)),
            automerge::ScalarValue::Int(i) => Ok(Some(*i as f64)),
            automerge::ScalarValue::Uint(u) => Ok(Some(*u as f64)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

/// Hydrates an `Option<i64>` while tolerating various Automerge numeric types.
///
/// Similar to `hydrate_optional_f64`, this accepts `Int`, `Uint`, and `F64`,
/// truncating fractional parts when converting to `i64`.
pub fn hydrate_optional_i64<D: autosurgeon::ReadDoc>(
    doc: &D,
    obj: &automerge::ObjId,
    prop: autosurgeon::Prop<'_>,
) -> Result<Option<i64>, autosurgeon::HydrateError> {
    let val = match prop {
        autosurgeon::Prop::Key(ref k) => doc.get(obj, k.as_ref()),
        autosurgeon::Prop::Index(i) => doc.get(obj, i as usize),
    }
    .map_err(|e| autosurgeon::HydrateError::unexpected("get", e.to_string()))?;

    match val {
        Some((automerge::Value::Scalar(scalar), _)) => match scalar.as_ref() {
            automerge::ScalarValue::Int(i) => Ok(Some(*i)),
            automerge::ScalarValue::Uint(u) => Ok(Some(*u as i64)),
            automerge::ScalarValue::F64(f) => Ok(Some(*f as i64)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

/// Hydrates an `i64` while tolerating various Automerge numeric types.
///
/// Returns an error if the value is missing or not a number.
pub fn hydrate_i64<D: autosurgeon::ReadDoc>(
    doc: &D,
    obj: &automerge::ObjId,
    prop: autosurgeon::Prop<'_>,
) -> Result<i64, autosurgeon::HydrateError> {
    hydrate_optional_i64(doc, obj, prop)?
        .ok_or_else(|| autosurgeon::HydrateError::unexpected("i64", "missing value".to_string()))
}

/// Reconciles an Option<T> using MaybeMissing semantics (None deletes the field).
pub fn reconcile_optional_as_maybe_missing<T, R>(
    val: &Option<T>,
    reconciler: R,
) -> Result<(), R::Error>
where
    T: autosurgeon::Reconcile,
    R: autosurgeon::Reconciler,
{
    use autosurgeon::MaybeMissing;
    let maybe_missing = match val {
        Some(v) => MaybeMissing::Present(v),
        None => MaybeMissing::Missing,
    };
    maybe_missing.reconcile(reconciler)
}

/// Reconciles an Option<f64> as MaybeMissing while preserving safe integer conversion.
pub fn reconcile_optional_f64_as_maybe_missing<R: autosurgeon::Reconciler>(
    val: &Option<f64>,
    reconciler: R,
) -> Result<(), R::Error> {
    use autosurgeon::MaybeMissing;
    match val {
        Some(v) => {
            struct JSF64(f64);
            impl autosurgeon::Reconcile for JSF64 {
                type Key<'a> = autosurgeon::reconcile::NoKey;
                fn reconcile<R2: autosurgeon::Reconciler>(
                    &self,
                    reconciler: R2,
                ) -> Result<(), R2::Error> {
                    reconcile_f64(&self.0, reconciler)
                }
            }
            MaybeMissing::Present(JSF64(*v)).reconcile(reconciler)
        }
        None => MaybeMissing::<f64>::Missing.reconcile(reconciler),
    }
}

/// Reconciles an `f64` into Automerge, optimizing for storage if it's a "Safe Integer".
///
/// If the value has no fractional part and fits within the "JavaScript Safe Integer"
/// range (±2^53 - 1), it is reconciled as an Automerge `Int` (i64). Otherwise, it is
/// reconciled as an Automerge `F64` (f64). This ensures precision for non-integers
/// while saving space and maintaining full JS safety for integers.
pub fn reconcile_f64<R: autosurgeon::Reconciler>(val: &f64, reconciler: R) -> Result<(), R::Error> {
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    const MIN_SAFE_INTEGER: f64 = -9_007_199_254_740_991.0;

    if val.fract() == 0.0 && *val <= MAX_SAFE_INTEGER && *val >= MIN_SAFE_INTEGER {
        (*val as i64).reconcile(reconciler)
    } else {
        val.reconcile(reconciler)
    }
}

/// Reconciles an Option<f64> using the same logic as reconcile_f64.
pub fn reconcile_optional_f64<R: autosurgeon::Reconciler>(
    val: &Option<f64>,
    mut reconciler: R,
) -> Result<(), R::Error> {
    match val {
        Some(v) => reconcile_f64(v, reconciler),
        None => reconciler.none(),
    }
}

/// Reconciles a String as an Automerge Text object.
pub fn reconcile_string_as_text<R: autosurgeon::Reconciler>(
    val: &str,
    mut reconciler: R,
) -> Result<(), R::Error> {
    use autosurgeon::reconcile::TextReconciler;
    reconciler.text()?.update(val)?;
    Ok(())
}

/// Reconciles a String as an Automerge Scalar string.
pub fn reconcile_string_as_scalar<R: autosurgeon::Reconciler>(
    val: &str,
    mut reconciler: R,
) -> Result<(), R::Error> {
    reconciler.str(val)
}

/// Reconciles an Optional<String> as an optional Automerge Text object.
pub fn reconcile_option_string_as_text<R: autosurgeon::Reconciler>(
    val: &Option<String>,
    mut reconciler: R,
) -> Result<(), R::Error> {
    match val {
        Some(s) => reconcile_string_as_text(s, reconciler),
        None => reconciler.none(),
    }
}

/// Reconciles an Optional<String> as an "maybe missing" Automerge Text object.
pub fn reconcile_option_string_as_text_as_maybe_missing<R: autosurgeon::Reconciler>(
    val: &Option<String>,
    reconciler: R,
) -> Result<(), R::Error> {
    match val {
        Some(s) => reconcile_string_as_text(s, reconciler),
        None => autosurgeon::MaybeMissing::<String>::Missing.reconcile(reconciler),
    }
}

macro_rules! define_id_type {
    ($doc:expr, $name:ident) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
        #[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
        #[serde(transparent)]
        pub struct $name(String);

        impl Hydrate for $name {
            fn hydrate<D: autosurgeon::ReadDoc>(
                doc: &D,
                obj: &automerge::ObjId,
                prop: autosurgeon::Prop<'_>,
            ) -> Result<Self, autosurgeon::HydrateError> {
                hydrate_string_or_text(doc, obj, prop).map(Self)
            }
        }

        impl Reconcile for $name {
            type Key<'a> = autosurgeon::reconcile::NoKey;
            fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
                reconcile_string_as_scalar(&self.0, reconciler)
            }
        }

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4().to_string())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::convert::Infallible;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.to_string()))
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id_type!("Unique identifier for a task.", TaskID);
define_id_type!("Unique identifier for a place/context.", PlaceID);

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
        let s = hydrate_string_or_text(doc, obj, prop)?;
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
        let text = match self {
            Self::Pending => "Pending",
            Self::Done => "Done",
        };
        reconcile_string_as_scalar(text, reconciler)
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
        let s = hydrate_string_or_text(doc, obj, prop)?;
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
        let text = match self {
            Self::Overdue => "Overdue",
            Self::Urgent => "Urgent",
            Self::Active => "Active",
            Self::Upcoming => "Upcoming",
            Self::None => "None",
        };
        reconcile_string_as_scalar(text, reconciler)
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
        let s = hydrate_string_or_text(doc, obj, prop)?;
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
        let text = match self {
            Self::Once => "Once",
            Self::Routinely => "Routinely",
            Self::DueDate => "DueDate",
            Self::Calendar => "Calendar",
        };
        reconcile_string_as_scalar(text, reconciler)
    }
}

/// Scheduling configuration for a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    /// The type of schedule (Once, Routinely, DueDate, Calendar).
    #[serde(rename = "type")]
    #[autosurgeon(rename = "type")]
    pub schedule_type: ScheduleType,
    /// Optional due date as Unix timestamp in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(
        rename = "dueDate",
        hydrate = "hydrate_optional_i64",
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::js_safe_option_i64()")
    )]
    pub due_date: Option<i64>,
    /// Lead time in milliseconds before due date to start showing urgency.
    #[autosurgeon(rename = "leadTime", hydrate = "hydrate_i64")]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::js_safe_i64()")
    )]
    pub lead_time: i64,
    /// Timestamp of last completion (for Routinely tasks).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(
        rename = "lastDone",
        hydrate = "hydrate_optional_i64",
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::js_safe_option_i64()")
    )]
    pub last_done: Option<i64>,
}

impl Hydrate for Schedule {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let sched_obj = match prop {
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
            _ => {
                return Err(autosurgeon::HydrateError::unexpected(
                    "map",
                    "index prop not supported".to_string(),
                ));
            }
        };

        Ok(Self {
            schedule_type: {
                let res = Hydrate::hydrate(
                    doc,
                    &sched_obj,
                    autosurgeon::Prop::Key(Cow::Borrowed("type")),
                );
                res?
            },
            due_date: hydrate_optional_i64(
                doc,
                &sched_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("dueDate")),
            )
            .ok()
            .flatten(),
            lead_time: hydrate_i64(
                doc,
                &sched_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("leadTime")),
            )
            .unwrap_or(crate::domain::constants::DEFAULT_LEAD_TIME_MILLIS),
            last_done: hydrate_optional_i64(
                doc,
                &sched_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("lastDone")),
            )
            .ok()
            .flatten(),
        })
    }
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
        let s = hydrate_string_or_text(doc, obj, prop)?;
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
        let s = match self {
            Self::Minutes => "minutes",
            Self::Hours => "hours",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
        };
        reconcile_string_as_scalar(s, reconciler)
    }
}

/// Configuration for task repetition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
pub struct RepeatConfig {
    /// The unit of frequency (daily, weekly, etc.).
    pub frequency: Frequency,
    /// The interval multiplier (e.g., 2 for "every 2 weeks").
    #[autosurgeon(hydrate = "hydrate_i64")]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "1i64..=1000i64")
    )]
    pub interval: i64,
}

/// A task as persisted in the Automerge document.
///
/// Uses `extra_fields` with `#[serde(flatten)]` to preserve any
/// unknown fields during roundtrip serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct PersistedTask {
    pub status: TaskStatus,
    #[key]
    pub id: TaskID,
    #[autosurgeon(
        hydrate = "hydrate_string_or_text",
        reconcile = "reconcile_string_as_text"
    )]
    pub title: String,
    #[serde(default)]
    #[autosurgeon(
        hydrate = "hydrate_string_or_text",
        reconcile = "reconcile_string_as_text"
    )]
    pub notes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(
        rename = "parentId",
        hydrate = "hydrate_option_maybe_missing",
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
    #[cfg_attr(any(test, feature = "test-utils"), proptest(value = "None"))]
    pub parent_id: Option<TaskID>,
    #[autosurgeon(rename = "childTaskIds")]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_vec_task_id()")
    )]
    pub child_task_ids: Vec<TaskID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(
        rename = "placeId",
        hydrate = "hydrate_option_maybe_missing",
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
    #[cfg_attr(any(test, feature = "test-utils"), proptest(value = "None"))]
    pub place_id: Option<PlaceID>,
    #[autosurgeon(hydrate = "hydrate_f64", reconcile = "reconcile_f64")]
    #[cfg_attr(any(test, feature = "test-utils"), proptest(strategy = "0.0..=1.0"))]
    pub importance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(
        rename = "creditIncrement",
        hydrate = "hydrate_optional_f64",
        reconcile = "reconcile_optional_f64_as_maybe_missing"
    )]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::js_safe_option_f64_pos()")
    )]
    pub credit_increment: Option<f64>,
    #[autosurgeon(hydrate = "hydrate_f64", reconcile = "reconcile_f64")]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "0.0..=1000000.0")
    )]
    pub credits: f64,
    #[autosurgeon(
        rename = "desiredCredits",
        hydrate = "hydrate_f64",
        reconcile = "reconcile_f64"
    )]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "0.0..=1000000.0")
    )]
    pub desired_credits: f64,
    #[autosurgeon(rename = "creditsTimestamp", hydrate = "hydrate_i64")]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::js_safe_u64()")
    )]
    pub credits_timestamp: i64,
    #[autosurgeon(rename = "priorityTimestamp", hydrate = "hydrate_i64")]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::js_safe_u64()")
    )]
    pub priority_timestamp: i64,
    pub schedule: Schedule,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(
        rename = "repeatConfig",
        hydrate = "hydrate_option_maybe_missing",
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
    pub repeat_config: Option<RepeatConfig>,
    #[autosurgeon(rename = "isSequential")]
    pub is_sequential: bool,
    #[serde(default)]
    #[autosurgeon(rename = "isAcknowledged")]
    pub is_acknowledged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[autosurgeon(
        rename = "lastCompletedAt",
        hydrate = "hydrate_optional_i64",
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::js_safe_option_i64()")
    )]
    pub last_completed_at: Option<i64>,
}

/// Hydrates an `f64` while tolerating various Automerge numeric types.
///
/// Returns an error if the value is missing or not a number.
pub fn hydrate_f64<D: autosurgeon::ReadDoc>(
    doc: &D,
    obj: &automerge::ObjId,
    prop: autosurgeon::Prop<'_>,
) -> Result<f64, autosurgeon::HydrateError> {
    hydrate_optional_f64(doc, obj, prop)?
        .ok_or_else(|| autosurgeon::HydrateError::unexpected("f64", "missing value".to_string()))
}

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
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
    pub mode: Option<PriorityMode>,
    #[autosurgeon(
        hydrate = "hydrate_option_maybe_missing",
        reconcile = "reconcile_optional_as_maybe_missing"
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
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
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
    #[autosurgeon(
        hydrate = "hydrate_option_maybe_missing",
        reconcile = "reconcile_optional_as_maybe_missing"
    )]
    pub schedule: Option<HashMap<String, Vec<String>>>,
}

/// A place/context where tasks can be performed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hydrate, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct Place {
    #[key]
    pub id: PlaceID,
    #[autosurgeon(
        hydrate = "hydrate_string_or_text",
        reconcile = "reconcile_string_as_text"
    )]
    pub name: String,
    /// Stringified JSON of OpenHours
    #[autosurgeon(
        hydrate = "hydrate_string_or_text",
        reconcile = "reconcile_string_as_text"
    )]
    pub hours: String,
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_vec_place_id()")
    )]
    #[autosurgeon(rename = "includedPlaces")]
    pub included_places: Vec<PlaceID>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
pub struct DocMetadata {
    #[autosurgeon(
        rename = "automerge_url",
        reconcile = "reconcile_option_string_as_text_as_maybe_missing"
    )]
    pub automerge_url: Option<String>,
}

impl Hydrate for DocMetadata {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let meta_obj = match prop {
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
            _ => {
                return Err(autosurgeon::HydrateError::unexpected(
                    "map",
                    "index prop not supported".to_string(),
                ));
            }
        };

        Ok(Self {
            automerge_url: hydrate_string_or_text(
                doc,
                &meta_obj,
                autosurgeon::Prop::Key(Cow::Borrowed("automerge_url")),
            )
            .ok(),
        })
    }
}

/// Hydrates an Option<DocMetadata> treating missing values as None.
pub fn hydrate_optional_metadata<D: autosurgeon::ReadDoc>(
    doc: &D,
    obj: &automerge::ObjId,
    prop: autosurgeon::Prop<'_>,
) -> Result<Option<DocMetadata>, autosurgeon::HydrateError> {
    match DocMetadata::hydrate(doc, obj, prop) {
        Ok(meta) => Ok(Some(meta)),
        Err(_) => Ok(None),
    }
}

/// The root state of a TaskLens document.
///
/// This is the top-level structure serialized to/from Automerge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(any(test, feature = "test-utils"), derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "camelCase")]
pub struct TunnelState {
    pub next_task_id: i64,
    pub next_place_id: i64,
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_tasks_map()")
    )]
    pub tasks: HashMap<TaskID, PersistedTask>,
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_vec_task_id()")
    )]
    pub root_task_ids: Vec<TaskID>,
    #[cfg_attr(
        any(test, feature = "test-utils"),
        proptest(strategy = "test_strategies::arbitrary_places_map()")
    )]
    pub places: HashMap<PlaceID, Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DocMetadata>,
}

impl TunnelState {
    /// Heals structural inconsistencies in the task tree.
    ///
    /// This function identifies and repairs the following broken invariants:
    ///
    /// 1. **Broken Links**: IDs in `root_task_ids` or `child_task_ids` that point to
    ///    tasks no longer present in the `tasks` map.
    ///    *Healing*: These "phantom" IDs are pruned from the lists.
    /// 2. **Broken Parent Links**: Tasks with a `parent_id` set to an ID that does
    ///    not exist in the `tasks` map.
    ///    *Healing*: The `parent_id` is cleared (set to `None`).
    /// 3. **Multiple/Missing List Presence**: Tasks that are in multiple child/root lists
    ///    or missing from their designated list (common after concurrent moves).
    ///    *Healing*: The `parent_id` is treated as the source of truth. The task is
    ///    removed from any "incorrect" lists and ensured to be present in its
    ///    designated parent's list (or roots), preserving existing order.
    pub fn heal_structural_inconsistencies(&mut self) {
        // 0. Deduplicate lists (keeping first occurrence) to fix duplicate ID regression
        // This handles cases where concurrent merges might insert the same ID multiple times.
        let mut seen = std::collections::HashSet::new();
        self.root_task_ids.retain(|id| seen.insert(id.clone()));

        for task in self.tasks.values_mut() {
            seen.clear();
            task.child_task_ids.retain(|id| seen.insert(id.clone()));
        }

        // 1. Prune broken links (pointing to non-existent tasks)
        let all_task_ids: std::collections::HashSet<_> = self.tasks.keys().cloned().collect();
        self.root_task_ids.retain(|id| all_task_ids.contains(id));
        for task in self.tasks.values_mut() {
            task.child_task_ids.retain(|id| all_task_ids.contains(id));
        }

        // 2. Prune broken parent links (pointing to non-existent tasks)
        for task in self.tasks.values_mut() {
            if task
                .parent_id
                .as_ref()
                .is_some_and(|pid| !all_task_ids.contains(pid))
            {
                task.parent_id = None;
            }
        }

        // 3. Surgical Pruning: Remove tasks from "wrong" lists based on their parent_id
        // Prune root_task_ids
        self.root_task_ids
            .retain(|id| self.tasks.get(id).is_some_and(|t| t.parent_id.is_none()));

        // Prune child_task_ids in every task
        // We use a temporary list of IDs to avoid borrowing issues during iteration
        let parent_ids: Vec<TaskID> = self.tasks.keys().cloned().collect();
        for pid in parent_ids {
            if let Some(mut child_ids) = self
                .tasks
                .get_mut(&pid)
                .map(|t| std::mem::take(&mut t.child_task_ids))
            {
                child_ids.retain(|cid| {
                    self.tasks
                        .get(cid)
                        .is_some_and(|ct| ct.parent_id.as_ref() == Some(&pid))
                });
                if let Some(t) = self.tasks.get_mut(&pid) {
                    t.child_task_ids = child_ids;
                }
            }
        }

        // 4. Ensure Presence: Add tasks to their designated list if missing
        // Re-sorting for determinism when adding missing items
        let mut sorted_ids: Vec<_> = self.tasks.keys().cloned().collect();
        sorted_ids.sort_by_key(|id| id.to_string());

        for id in sorted_ids {
            let parent_id = self.tasks.get(&id).and_then(|t| t.parent_id.clone());
            if let Some(pid) = parent_id {
                if let Some(parent_task) = self
                    .tasks
                    .get_mut(&pid)
                    .filter(|t| !t.child_task_ids.contains(&id))
                {
                    parent_task.child_task_ids.push(id);
                }
            } else if !self.root_task_ids.contains(&id) {
                self.root_task_ids.push(id);
            }
        }
    }
}

impl Default for TunnelState {
    fn default() -> Self {
        Self {
            next_task_id: 1,
            next_place_id: 1,
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
            credits_timestamp: 12345678,
            priority_timestamp: 12345678,
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
            "creditsTimestamp": 12345678,
            "priorityTimestamp": 12345678,
            "schedule": {
                "type": "Once",
                "leadTime": 0
            },
            "isSequential": false,
            "isAcknowledged": false
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_task_id_hydration() {
        use automerge::transaction::Transactable;
        use automerge::{AutoCommit, ObjType};
        use autosurgeon::{Hydrate, hydrate};

        let mut doc = AutoCommit::new();

        // 1. Scalar String
        doc.put(automerge::ROOT, "id_scalar", "task-scalar")
            .unwrap();
        #[derive(Hydrate, Debug, PartialEq)]
        struct ScalarTest {
            #[autosurgeon(rename = "id_scalar")]
            id: TaskID,
        }
        let res: ScalarTest = hydrate(&doc).unwrap();
        assert_eq!(res.id.as_str(), "task-scalar");

        // 2. Text Object
        let text_id = doc
            .put_object(automerge::ROOT, "id_text", ObjType::Text)
            .unwrap();
        doc.splice_text(&text_id, 0, 0, "task-text").unwrap();
        #[derive(Hydrate, Debug, PartialEq)]
        struct TextTest {
            #[autosurgeon(rename = "id_text")]
            id: TaskID,
        }
        let res: TextTest = hydrate(&doc).unwrap();
        assert_eq!(res.id.as_str(), "task-text");
    }

    #[test]
    fn test_place_id_hydration() {
        use automerge::transaction::Transactable;
        use automerge::{AutoCommit, ObjType};
        use autosurgeon::{Hydrate, hydrate};

        let mut doc = AutoCommit::new();

        // 1. Scalar String
        doc.put(automerge::ROOT, "id_scalar", "place-scalar")
            .unwrap();
        #[derive(Hydrate, Debug, PartialEq)]
        struct ScalarTest {
            #[autosurgeon(rename = "id_scalar")]
            id: PlaceID,
        }
        let res: ScalarTest = hydrate(&doc).unwrap();
        assert_eq!(res.id.as_str(), "place-scalar");

        // 2. Text Object
        let text_id = doc
            .put_object(automerge::ROOT, "id_text", ObjType::Text)
            .unwrap();
        doc.splice_text(&text_id, 0, 0, "place-text").unwrap();
        #[derive(Hydrate, Debug, PartialEq)]
        struct TextTest {
            #[autosurgeon(rename = "id_text")]
            id: PlaceID,
        }
        let res: TextTest = hydrate(&doc).unwrap();
        assert_eq!(res.id.as_str(), "place-text");
    }

    #[test]
    fn test_enum_hydration() {
        use automerge::transaction::Transactable;
        use automerge::{AutoCommit, ObjType};
        use autosurgeon::{Hydrate, hydrate};

        let mut doc = AutoCommit::new();

        // TaskStatus from Text
        let text_id = doc
            .put_object(automerge::ROOT, "status", ObjType::Text)
            .unwrap();
        doc.splice_text(&text_id, 0, 0, "Done").unwrap();
        #[derive(Hydrate, Debug, PartialEq)]
        struct StatusTest {
            status: TaskStatus,
        }
        let res: StatusTest = hydrate(&doc).unwrap();
        assert_eq!(res.status, TaskStatus::Done);

        // Frequency from Text (Standardize to scalar later)
        let text_id = doc
            .put_object(automerge::ROOT, "freq", ObjType::Text)
            .unwrap();
        doc.splice_text(&text_id, 0, 0, "weekly").unwrap();
        #[derive(Hydrate, Debug, PartialEq)]
        struct FreqTest {
            #[autosurgeon(rename = "freq")]
            frequency: Frequency,
        }
        let res: FreqTest = hydrate(&doc).unwrap();
        assert_eq!(res.frequency, Frequency::Weekly);
    }

    #[derive(Hydrate, Debug, PartialEq)]
    struct I64Test {
        #[autosurgeon(hydrate = "hydrate_i64")]
        val: i64,
    }

    #[derive(Hydrate, Debug, PartialEq)]
    struct F64Test {
        #[autosurgeon(hydrate = "hydrate_f64", rename = "val2")]
        val: f64,
    }

    #[test]
    fn test_numeric_hydration_tolerance() {
        use automerge::AutoCommit;
        use automerge::transaction::Transactable;
        use autosurgeon::hydrate;

        let mut doc = AutoCommit::new();

        // 1. i64 from F64
        doc.put(automerge::ROOT, "val", 123.45f64).unwrap();
        let res: I64Test = hydrate(&doc).unwrap();
        assert_eq!(res.val, 123); // Truncated

        // 2. f64 from Int
        doc.put(automerge::ROOT, "val2", 456i64).unwrap();
        let res: F64Test = hydrate(&doc).unwrap();
        assert_eq!(res.val, 456.0);
    }

    #[test]
    fn test_numeric_list_hydration() {
        use automerge::transaction::Transactable;
        use automerge::{AutoCommit, ObjType};

        let mut doc = AutoCommit::new();
        let list_id = doc
            .put_object(automerge::ROOT, "list", ObjType::List)
            .unwrap();
        doc.insert(&list_id, 0, 1.23f64).unwrap();
        doc.insert(&list_id, 1, 456i64).unwrap();

        // Test direct call with Index
        let val1 = hydrate_f64(&doc, &list_id, autosurgeon::Prop::Index(0)).unwrap();
        assert_eq!(val1, 1.23);

        let val2 = hydrate_f64(&doc, &list_id, autosurgeon::Prop::Index(1)).unwrap();
        assert_eq!(val2, 456.0);

        let val3 = hydrate_i64(&doc, &list_id, autosurgeon::Prop::Index(1)).unwrap();
        assert_eq!(val3, 456);

        let val4 = hydrate_i64(&doc, &list_id, autosurgeon::Prop::Index(0)).unwrap();
        assert_eq!(val4, 1); // Truncated
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

    pub const MIN_SAFE_INTEGER: i64 = -9_007_199_254_740_991;
    pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

    pub fn js_safe_i64() -> impl Strategy<Value = i64> {
        MIN_SAFE_INTEGER..=MAX_SAFE_INTEGER
    }

    pub fn js_safe_option_i64() -> impl Strategy<Value = Option<i64>> {
        proptest::option::of(js_safe_i64())
    }

    pub fn js_safe_u64() -> impl Strategy<Value = i64> {
        0..=MAX_SAFE_INTEGER
    }

    pub fn js_safe_i64_some() -> impl Strategy<Value = Option<i64>> {
        proptest::option::weighted(0.9, js_safe_i64())
    }

    pub fn js_safe_option_f64_pos() -> impl Strategy<Value = Option<f64>> {
        proptest::option::of(0.0..1000000.0)
    }
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

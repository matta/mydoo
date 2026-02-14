//! Score trace view for Do list tasks.

use crate::app_components::{BackButton, EmptyState, LoadErrorView, PageHeader};
use crate::dioxus_components::badge::{Badge, BadgeVariant};
use crate::dioxus_components::card::{Card, CardContent};
use crate::hooks::use_score_trace::use_score_trace;
use crate::router::Route;
use chrono::DateTime;
use dioxus::prelude::*;
use tasklens_core::types::{LeadTimeStage, ScheduleSource, ScoreTrace, TaskID};

/// Renders the score trace for a single task.
#[component]
pub fn ScoreTracePage(task_id: TaskID) -> Element {
    let load_error = use_context::<Signal<Option<String>>>();
    let trace = use_score_trace(task_id);
    let navigator = use_navigator();

    let on_back = move |_| {
        navigator.push(Route::DoPage {});
    };

    rsx! {
        div {
            class: "px-4 pt-4 pb-20 container mx-auto max-w-2xl space-y-4",
            style: "padding-top: var(--safe-top); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",
            "data-testid": "score-trace",

            BackButton { onclick: on_back }
            PageHeader { title: "Score Trace" }

            if let Some(error) = load_error() {
                LoadErrorView {
                    error,
                    help_text: Some(
                        "Access the settings menu to switch documents or change sync servers."
                            .to_string(),
                    ),
                }
            } else if let Some(trace) = trace() {
                ScoreTraceContent { trace }
            } else {
                EmptyState {
                    title: "Score trace unavailable.",
                    subtitle: "The task might no longer exist or is filtered out.",
                }
            }
        }
    }
}

/// The main score trace content layout.
#[component]
fn ScoreTraceContent(trace: ScoreTrace) -> Element {
    let score_label = format_factor(trace.score);
    let computed_at = format_timestamp(Some(trace.computed_at));

    rsx! {
        div { class: "space-y-4",
            Card {
                "data-testid": "score-trace-summary",
                CardContent {
                    h2 { class: "text-base font-medium text-base-content", "Task" }
                    p {
                        class: "text-lg font-semibold text-base-content",
                        "data-testid": "score-trace-task-title",
                        "{trace.task_title}"
                    }
                    p {
                        class: "text-sm text-base-content/70",
                        "data-testid": "score-trace-score",
                        "Score {score_label}"
                    }
                    p { class: "text-xs text-base-content/50", "Computed at {computed_at}" }
                }
            }

            Card {
                "data-testid": "score-trace-formula",
                CardContent {
                    h3 { class: "text-sm font-medium text-base-content/80", "Formula" }
                    code { class: "text-xs text-base-content/70", "score = visibility * normalized_importance * feedback * lead_time" }
                    div { class: "grid grid-cols-2 gap-2 text-xs text-base-content/70",
                        div { "Visibility: {format_factor(trace.factors.visibility_factor)}" }
                        div { "Normalized Importance: {format_factor(trace.factors.normalized_importance)}" }
                        div { "Feedback: {format_factor(trace.factors.feedback_factor)}" }
                        div { "Lead Time: {format_factor(trace.factors.lead_time_factor)}" }
                    }
                }
            }

            Card {
                "data-testid": "score-trace-feedback",
                CardContent {
                    h3 { class: "text-sm font-medium text-base-content/80", "Balance Feedback" }
                    p { class: "text-xs text-base-content/70",
                        "Root: {trace.feedback.root_title}"
                    }
                    div { class: "grid grid-cols-2 gap-2 text-xs text-base-content/70",
                        div { "Desired Credits: {format_factor(trace.feedback.desired_credits)}" }
                        div { "Effective Credits: {format_factor(trace.feedback.effective_credits)}" }
                        div { "Target %: {format_percent(trace.feedback.target_percent)}" }
                        div { "Actual %: {format_percent(trace.feedback.actual_percent)}" }
                        div { "Deviation Ratio: {format_factor(trace.feedback.deviation_ratio)}" }
                        div { "Factor: {format_factor(trace.feedback.feedback_factor)}" }
                    }
                }
            }

            Card {
                "data-testid": "score-trace-importance",
                CardContent {
                    h3 { class: "text-sm font-medium text-base-content/80", "Importance Chain" }
                    for entry in trace.importance_chain.iter() {
                        div {
                            key: "{entry.task_id}",
                            class: "border border-base-200 rounded p-2 text-xs space-y-1",
                            div { class: "flex justify-between items-center",
                                span { class: "font-medium text-base-content", "{entry.task_title}" }
                                if entry.sequential_blocked {
                                    Badge {
                                        variant: BadgeVariant::Secondary,
                                        "Sequential Blocked"
                                    }
                                }
                            }
                            div { class: "text-base-content/70",
                                "Importance {format_factor(entry.importance)} -> Normalized {format_factor(entry.normalized_importance)}"
                            }
                            if let Some(parent_norm) = entry.parent_normalized_importance {
                                div { class: "text-base-content/50",
                                    "Parent normalized {format_factor(parent_norm)}"
                                }
                            }
                        }
                    }
                }
            }

            Card {
                "data-testid": "score-trace-lead-time",
                CardContent {
                    h3 { class: "text-sm font-medium text-base-content/80", "Lead Time" }
                    div { class: "grid grid-cols-2 gap-2 text-xs text-base-content/70",
                        div { "Due Date: {format_timestamp(trace.lead_time.effective_due_date)}" }
                        div { "Lead Time: {format_millis(Some(trace.lead_time.effective_lead_time))}" }
                        div { "Time Remaining: {format_millis(trace.lead_time.time_remaining)}" }
                        div { "Stage: {lead_time_stage_label(trace.lead_time.stage)}" }
                        div { "Factor: {format_factor(trace.lead_time.factor)}" }
                        div { "Schedule Source: {schedule_source_label(trace.lead_time.schedule_source)}" }
                    }
                }
            }

            Card {
                "data-testid": "score-trace-visibility",
                CardContent {
                    h3 { class: "text-sm font-medium text-base-content/80", "Visibility" }
                    div { class: "grid grid-cols-2 gap-2 text-xs text-base-content/70",
                        div { "Place: {trace.visibility.contextual.effective_place_id}" }
                        div { "Place Open: {bool_label(trace.visibility.contextual.is_open)}" }
                        div { "Filter Match: {bool_label(trace.visibility.contextual.filter_match)}" }
                        div { "Acknowledged: {bool_label(trace.visibility.contextual.is_acknowledged)}" }
                        div { "Has Pending Descendants: {bool_label(trace.visibility.has_pending_descendants)}" }
                        div { "Delegated: {bool_label(trace.visibility.delegated_to_descendants)}" }
                        div { "Final Visibility: {bool_label(trace.visibility.final_visibility)}" }
                    }
                }
            }
        }
    }
}

/// Formats a numeric factor for display.
#[must_use]
fn format_factor(value: f64) -> String {
    format!("{:.3}", value)
}

/// Formats a ratio as a percentage string.
#[must_use]
fn format_percent(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

/// Formats a timestamp (ms) for display.
#[must_use]
fn format_timestamp(timestamp: Option<i64>) -> String {
    match timestamp.and_then(DateTime::from_timestamp_millis) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M UTC").to_string(),
        None => "None".to_string(),
    }
}

/// Formats a millisecond duration for display.
#[must_use]
fn format_millis(value: Option<i64>) -> String {
    match value {
        Some(ms) => format_duration(ms),
        None => "None".to_string(),
    }
}

/// Maps a lead time stage enum to a human-readable label.
#[must_use]
fn lead_time_stage_label(stage: LeadTimeStage) -> &'static str {
    match stage {
        LeadTimeStage::TooEarly => "Too Early",
        LeadTimeStage::Ramping => "Ramping",
        LeadTimeStage::Ready => "Ready",
        LeadTimeStage::Overdue => "Overdue",
    }
}

/// Maps schedule source to a human-readable label.
#[must_use]
fn schedule_source_label(source: Option<ScheduleSource>) -> &'static str {
    match source {
        Some(ScheduleSource::Myself) => "Self",
        Some(ScheduleSource::Ancestor) => "Ancestor",
        None => "None",
    }
}

/// Formats a boolean as a Yes/No label.
#[must_use]
fn bool_label(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

/// Formats a duration in milliseconds using a human-readable unit.
#[must_use]
fn format_duration(ms: i64) -> String {
    let is_negative = ms.is_negative();
    let abs = ms.unsigned_abs();

    let (value, unit) = if abs >= 86_400_000 {
        (abs / 86_400_000, "d")
    } else if abs >= 3_600_000 {
        (abs / 3_600_000, "h")
    } else if abs >= 60_000 {
        (abs / 60_000, "m")
    } else if abs >= 1_000 {
        (abs / 1_000, "s")
    } else {
        (abs, "ms")
    };

    let prefix = if is_negative { "-" } else { "" };
    format!("{prefix}{value}{unit}")
}

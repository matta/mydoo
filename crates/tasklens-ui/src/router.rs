use crate::app_components::AppNavBar;
use crate::views::balance_page::BalancePage;
use crate::views::do_page::DoPage;
use crate::views::plan_page::PlanPage;
use crate::views::score_trace_page::ScoreTracePage;
use crate::views::settings_page::SettingsPage;
use dioxus::prelude::*;
use std::fmt;
use std::str::FromStr;
use tasklens_core::types::TaskID;

/// Defines the valid return-context values for the `/settings` route.
///
/// This captures in-app provenance for settings entry (for example from
/// `Plan` or `Do`) and keeps query handling stable when `ctx` is omitted
/// or unexpected.
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) enum ViewContext {
    #[default]
    Plan,
    Do,
}

impl fmt::Display for ViewContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViewContext::Plan => write!(f, "plan"),
            ViewContext::Do => write!(f, "do"),
        }
    }
}

impl FromStr for ViewContext {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "do" => Ok(ViewContext::Do),
            _ => Ok(ViewContext::Plan),
        }
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
pub(crate) enum Route {
    #[layout(AppNavBar)]
    #[route("/")]
    #[redirect("/", || Route::PlanPage { focus_task: None, seed: None })]
    Home {},

    #[route("/plan?:focus_task&:seed")]
    PlanPage {
        focus_task: Option<TaskID>,
        seed: Option<bool>,
    },

    #[route("/do")]
    DoPage {},

    #[route("/do/trace/:task_id")]
    ScoreTracePage { task_id: TaskID },

    #[route("/balance")]
    BalancePage {},

    #[route("/settings?:ctx")]
    SettingsPage { ctx: Option<ViewContext> },

    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        div { "Page not found: {route:?}" }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div { "Redirecting..." }
    }
}

use crate::components::app_navbar::AppNavBar;
use crate::views::balance_page::BalancePage;
use crate::views::do_page::DoPage;
use crate::views::plan_page::PlanPage;
use crate::views::task_page::TaskPage;
use dioxus::prelude::*;
use tasklens_core::types::TaskID;

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

    #[route("/balance")]
    BalancePage {},

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

#[component]
fn Do() -> Element {
    // Re-use TaskPage logic or wrap it?
    // TaskPage needs props: master_key, service_worker_active.
    // We need to inject them via Context.
    rsx! {
        TaskPage {}
    }
}

use crate::components::app_navbar::AppNavBar;
use crate::views::plan_page::PlanPage;
use crate::views::task_page::TaskPage;
use dioxus::prelude::*;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(AppNavBar)]
    #[route("/")]
    #[redirect("/", || Route::PlanPage {})]
    Home {},

    #[route("/plan")]
    PlanPage {},

    #[route("/do")]
    TaskPage {}, // Maps to TaskPage component

    #[route("/balance")]
    Balance {}, // Placeholder

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

// Temporary placeholders if pages don't exist yet
#[component]
fn Balance() -> Element {
    rsx! {
        div { "Balance View (Coming Soon)" }
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

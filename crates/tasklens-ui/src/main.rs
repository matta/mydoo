use dioxus::prelude::*;

fn main() {
    // Initialize the tracing subscriber to capture logs in the browser console.
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            h1 { "TaskLens Migration" }
            p { "Milestone 1.1 complete: Rust workspace initialized." }
        }
    }
}

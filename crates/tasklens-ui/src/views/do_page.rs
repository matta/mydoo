use crate::components::priority_task_row::PriorityTaskRow;
use crate::components::task_input::TaskInput;
use crate::controllers::task_controller;
use dioxus::prelude::*;
use tasklens_core::domain::priority::get_prioritized_tasks;
use tasklens_core::types::{PriorityMode, PriorityOptions, TaskID, ViewFilter};
use tasklens_store::store::AppStore;

#[component]
pub fn DoPage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut input_text = use_signal(String::new);

    let prioritized_tasks = use_memo(move || {
        let store_read = store.read();
        let state = store_read.get_state().unwrap_or_default();
        let view_filter = ViewFilter {
            place_id: Some("All".to_string()),
        };
        let options = PriorityOptions {
            include_hidden: false,
            mode: Some(PriorityMode::DoList),
            context: None, // Will use current time internally
        };
        get_prioritized_tasks(&state, &view_filter, &options)
    });

    let on_toggle = move |id: TaskID| {
        task_controller::toggle_task_status(store, id);
    };

    let on_title_tap = move |_id: TaskID| {
        // TODO: Open task editor or navigate to details
        tracing::info!("Task title tapped: {}", _id);
    };

    let handle_add = move |_| {
        let text = input_text.read().clone();
        if text.trim().is_empty() {
            return;
        }

        task_controller::create_task(store, None, text);
        input_text.set(String::new());
    };

    rsx! {
        div { class: "container mx-auto px-4 py-6 max-w-3xl",
            h1 { class: "text-2xl font-bold text-gray-900 mb-6", "Work" }

            TaskInput {
                data_testid: "do-task-input",
                value: input_text,
                on_add: handle_add,
            }

            if prioritized_tasks().is_empty() {
                div { class: "text-center py-12 text-gray-500 bg-gray-50 rounded-lg border-2 border-dashed border-gray-200",
                    p { "No tasks ready for now." }
                    p { class: "text-sm mt-2", "Check your Plan or wait for scheduled tasks." }
                }
            } else {
                div { class: "space-y-1",
                    for task in prioritized_tasks() {
                        PriorityTaskRow {
                            key: "{task.id}",
                            task: task.clone(),
                            on_toggle,
                            on_title_tap,
                        }
                    }
                }
            }
        }
    }
}

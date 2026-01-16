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

    #[derive(Debug, Clone, PartialEq)]
    enum EditorState {
        Edit(TaskID),
        Create { parent_id: Option<TaskID> },
    }
    let mut editor_state = use_signal(|| None::<EditorState>);

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

    let on_title_tap = move |id: TaskID| {
        editor_state.set(Some(EditorState::Edit(id)));
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
            div { class: "flex justify-between items-center mb-6",
                h1 { class: "text-2xl font-bold text-gray-900", "Work" }
                button {
                    class: "px-3 py-1 bg-white border border-gray-200 rounded-md text-sm font-medium text-gray-600 hover:bg-gray-50 flex items-center shadow-sm",
                    onclick: move |_| task_controller::refresh_lifecycle(store),
                    "Refresh"
                }
            }

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

            if let Some(state) = editor_state() {
                match state {
                    EditorState::Edit(id) => rsx! {
                        crate::components::TaskEditor {
                            task_id: Some(id),
                            on_close: move |_| editor_state.set(None),
                            on_add_child: move |parent_id| {
                                editor_state.set(Some(EditorState::Create { parent_id: Some(parent_id) }));
                            },
                            on_task_created: None,
                        }
                    },
                    EditorState::Create { parent_id } => rsx! {
                        crate::components::TaskEditor {
                            initial_parent_id: parent_id,
                            on_close: move |_| editor_state.set(None),
                            on_task_created: None,
                        }
                    }
                }
            }
        }
    }
}

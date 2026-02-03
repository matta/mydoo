use crate::components::priority_task_row::PriorityTaskRow;
use crate::components::{Button, ButtonVariant, LoadErrorView, PageHeader, TaskInput};
use crate::controllers::task_controller;
use dioxus::prelude::*;
use tasklens_core::domain::priority::get_prioritized_tasks;
use tasklens_core::types::{PriorityMode, PriorityOptions, TaskID, ViewFilter};

#[component]
pub fn DoPage() -> Element {
    let task_controller = task_controller::use_task_controller();
    let load_error = use_context::<Signal<Option<String>>>();
    let mut input_text = use_signal(String::new);

    #[derive(Debug, Clone, PartialEq)]
    enum EditorState {
        Edit(TaskID),
        Create { parent_id: Option<TaskID> },
    }
    let mut editor_state = use_signal(|| None::<EditorState>);

    let state = crate::hooks::use_tunnel_state::use_tunnel_state();

    let prioritized_tasks = use_memo({
        move || {
            let state = state.read();
            let view_filter = ViewFilter {
                place_id: Some("All".to_string()),
            };
            let options = PriorityOptions {
                include_hidden: false,
                mode: Some(PriorityMode::DoList),
                context: None, // Will use current time internally
            };
            get_prioritized_tasks(&state, &view_filter, &options)
        }
    });

    let on_toggle = {
        move |id: TaskID| {
            task_controller.toggle(id);
        }
    };

    let on_title_tap = move |id: TaskID| {
        editor_state.set(Some(EditorState::Edit(id)));
    };

    let handle_add = {
        move |_| {
            let text = input_text.read().clone();
            if text.trim().is_empty() {
                return;
            }

            task_controller.create(None, text);
            input_text.set(String::new());
        }
    };

    rsx! {
        div {
            class: "px-4 pt-4 pb-20 container mx-auto max-w-2xl",
            style: "padding-top: var(--safe-top); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",

            PageHeader { title: "Work",
                Button {
                    variant: ButtonVariant::Ghost,
                    class: "btn-sm text-base-content/70 hover:text-base-content border border-base-300",
                    onclick: move |_| task_controller.refresh_lifecycle(),
                    "Refresh"
                }
            }

            TaskInput {
                data_testid: "do-task-input",
                value: input_text,
                on_add: handle_add,
            }

            if let Some(error) = load_error() {
                LoadErrorView {
                    error,
                    help_text: Some(
                        "Access the settings menu from the navigation bar to switch documents or change sync servers."
                            .to_string(),
                    ),
                }
            } else if prioritized_tasks().is_empty() {
                div { class: "card bg-base-200 border-2 border-dashed border-base-300 py-12 text-center",
                    div { class: "card-body items-center text-base-content/60",
                        p { "No tasks ready for now." }
                        p { class: "text-base mt-2", "Check your Plan or wait for scheduled tasks." }
                    }
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
                                editor_state
                                    .set(
                                        Some(EditorState::Create {
                                            parent_id: Some(parent_id),
                                        }),
                                    );
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
                    },
                }
            }
        }
    }
}

use crate::components::priority_task_row::PriorityTaskRow;
use crate::components::{Button, ButtonVariant, EmptyState, LoadErrorView, PageHeader, TaskInput};
use crate::controllers::task_controller;
use crate::hooks::use_prioritized_tasks::use_do_list_tasks;
use dioxus::prelude::*;
use tasklens_core::types::TaskID;

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

    let prioritized_tasks = use_do_list_tasks();

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
                    class: "btn-sm text-base-content/70 hover:text-base-content",
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
                EmptyState {
                    title: "No tasks ready for now.",
                    subtitle: "Check your Plan or wait for scheduled tasks.",
                }
            } else {
                div { class: "space-y-2",
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
                            key: "edit-{id}",
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
                            key: "create-{parent_id:?}",
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

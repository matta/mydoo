//! Main Task Application Views.
//!
//! This module contains the core application logic and UI components for the Task list functionality.
//! It handles state management, synchronization integration, and the composition of the main page.

use crate::app_components::{LoadErrorView, PageHeader, TaskInput};
use crate::dioxus_components::badge::{Badge, BadgeVariant};
use crate::dioxus_components::checkbox::Checkbox;
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
use std::rc::Rc;
use tasklens_core::types::{PersistedTask, TaskID, TaskStatus};

#[css_module("/src/views/task_page.css")]
struct Styles;

/// The main application page component.
///
/// `TaskPage` allows the user to view, add, and toggle tasks. It also:
/// - Manages the local `AppStore` state.
/// - Handles hydration from persistent storage (IndexedDB).
/// - Orchestrates the background synchronization task.
#[component]
pub fn TaskPage() -> Element {
    let service_worker_active = use_context::<Signal<bool>>();
    let task_controller = crate::controllers::task_controller::use_task_controller();
    let load_error = use_context::<Signal<Option<String>>>();

    let mut input_text = use_signal(String::new);

    // Hydrate state from the store
    let state = crate::hooks::use_tunnel_state::use_tunnel_state();

    let save_and_sync = move || {
        // Explicit persist removed. Handled by use_persistence hook.
    };

    let mut add_task = {
        move || {
            let text = input_text();
            if text.trim().is_empty() {
                return;
            }

            if task_controller.create(None, text.clone()).is_none() {
                return;
            }

            save_and_sync();
            input_text.set(String::new());
        }
    };

    let toggle_task = {
        move |id: TaskID| {
            task_controller.toggle(id);
            save_and_sync();
        }
    };

    // Prepare tasks for display (convert HashMap to Vec and Sort)
    let tasks: Vec<Rc<PersistedTask>> = {
        let mut t: Vec<Rc<PersistedTask>> = state()
            .tasks
            .values()
            .map(|t| Rc::new(t.clone()))
            .collect();
        // Sort by title for stability, or ID. todo_mvp didn't sort, but HashMap iteration is random.
        // Let's sort by ID string for now to have deterministic order.
        t.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        t
    };

    rsx! {
        div {
            class: Styles::page_container,
            style: "padding-top: var(--safe-top); padding-bottom: var(--safe-bottom); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",
            PageHeader { title: "TaskLens",
                if service_worker_active() {
                    Badge {
                        variant: BadgeVariant::Primary,
                        class: Styles::offline_badge,
                        title: "Service Worker Active",
                        "Offline Ready"
                    }
                }
            }

            if let Some(error) = load_error() {
                LoadErrorView {
                    error,
                    help_text: Some(
                        "Access the settings menu from the navigation bar to switch documents or change sync servers."
                            .to_string(),
                    ),
                }
            } else {
                TaskInput { value: input_text, on_add: move |_| add_task() }
                TaskList { tasks, on_toggle: toggle_task }
            }

            div { class: Styles::build_version, "Build: {crate::BUILD_VERSION}" }
        }
    }
}

#[component]
fn TaskList(tasks: Vec<Rc<PersistedTask>>, on_toggle: EventHandler<TaskID>) -> Element {
    rsx! {
        ul { class: Styles::task_list,
            for task in tasks {
                TaskItem {
                    key: "{task.id}",
                    task: task,
                    on_toggle,
                }
            }
        }
    }
}

#[component]
fn TaskItem(task: Rc<PersistedTask>, on_toggle: EventHandler<TaskID>) -> Element {
    let is_done = task.status == TaskStatus::Done;
    let task_id_toggle = task.id.clone();
    let task_id_check = task.id.clone();
    rsx! {
        li {
            class: Styles::task_item,
            onclick: move |_| on_toggle.call(task_id_toggle.clone()),
            div { onclick: move |evt: MouseEvent| evt.stop_propagation(),
                Checkbox {
                    checked: Some(if is_done {
                        CheckboxState::Checked
                    } else {
                        CheckboxState::Unchecked
                    }),
                    on_checked_change: move |_| on_toggle.call(task_id_check.clone()),
                    class: Styles::checkbox_input,
                }
            }
            span {
                class: if is_done { Styles::task_title_done } else { Styles::task_title },
                "{task.title}"
            }
        }
    }
}

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
    // Memoized to prevent expensive deep cloning of tasks on every render.
    let tasks_memo = use_memo(move || {
        let mut t: Vec<Rc<PersistedTask>> =
            state().tasks.values().map(|t| Rc::new(t.clone())).collect();
        // Sort by title for stability, or ID. todo_mvp didn't sort, but HashMap iteration is random.
        // Let's sort by ID string for now to have deterministic order.
        t.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        t
    });

    let tasks = tasks_memo();

    rsx! {
        div {
            class: Styles::page_container,
            style: "padding-top: var(--app_safe_top); padding-bottom: var(--app_safe_bottom); padding-left: var(--app_safe_left); padding-right: var(--safe-right);",
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
    rsx! {
        li {
            class: Styles::task_item,
            onclick: {
                let task = task.clone();
                move |_| on_toggle.call(task.id.clone())
            },
            div { onclick: move |evt: MouseEvent| evt.stop_propagation(),
                Checkbox {
                    checked: Some(if is_done {
                        CheckboxState::Checked
                    } else {
                        CheckboxState::Unchecked
                    }),
                    on_checked_change: {
                        let task = task.clone();
                        move |_| on_toggle.call(task.id.clone())
                    },
                    class: Styles::checkbox_input,
                    aria_label: "Toggle completion for {task.title}",
                }
            }
            span {
                class: if is_done { Styles::task_title_done } else { Styles::task_title },
                "{task.title}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tasklens_core::types::{Schedule, ScheduleType};

    fn create_dummy_task() -> Rc<PersistedTask> {
        Rc::new(PersistedTask {
            status: TaskStatus::Pending,
            id: TaskID::from("task-1"),
            title: "Test Task".to_string(),
            notes: "Some notes".to_string(),
            parent_id: None,
            child_task_ids: vec![],
            place_id: None,
            importance: 1.0,
            credit_increment: None,
            credits: 0.0,
            desired_credits: 1.0,
            credits_timestamp: 12345678,
            priority_timestamp: 12345678,
            schedule: Schedule {
                schedule_type: ScheduleType::Once,
                due_date: None,
                lead_time: 0,
                last_done: None,
            },
            repeat_config: None,
            is_sequential: false,
            is_acknowledged: false,
            last_completed_at: None,
        })
    }

    #[test]
    fn test_task_item_renders() {
        fn app() -> Element {
            let task = create_dummy_task();
            rsx! {
                TaskItem {
                    task: task,
                    on_toggle: |_| {},
                }
            }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
    }
}

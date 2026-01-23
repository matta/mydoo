//! Main Task Application Views.
//!
//! This module contains the core application logic and UI components for the Task list functionality.
//! It handles state management, synchronization integration, and the composition of the main page.

use crate::components::*;
use crate::views::auth::SettingsModal;
use dioxus::prelude::*;
use tasklens_core::types::{PersistedTask, TaskStatus};
use tasklens_store::store::AppStore;

/// The main application page component.
///
/// `TaskPage` allows the user to view, add, and toggle tasks. It also:
/// - Manages the local `AppStore` state.
/// - Handles hydration from persistent storage (IndexedDB).
/// - Orchestrates the background synchronization task.
/// - Integrates the `SettingsModal` for identity management.
#[component]
pub fn TaskPage() -> Element {
    let mut doc_id = use_context::<Signal<Option<tasklens_store::doc_id::DocumentId>>>();
    let service_worker_active = use_context::<Signal<bool>>(); // Assuming bool not ReadSignal for context simplicity, or ReadSignal
    let mut store = use_context::<Signal<AppStore>>();
    let load_error = use_context::<Signal<Option<String>>>();

    let mut input_text = use_signal(String::new);
    let mut show_settings = use_signal(|| false);

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

            if crate::controllers::task_controller::create_task(
                store,
                Some(load_error),
                None,
                text.clone(),
            )
            .is_none()
            {
                return;
            }

            save_and_sync();
            input_text.set(String::new());
        }
    };

    let toggle_task = {
        move |task: PersistedTask| {
            crate::controllers::task_controller::toggle_task_status(store, load_error, task.id);
            save_and_sync();
        }
    };

    // Prepare tasks for display (convert HashMap to Vec and Sort)
    let tasks: Vec<PersistedTask> = {
        let mut t: Vec<PersistedTask> = state().tasks.values().cloned().collect();
        // Sort by title for stability, or ID. todo_mvp didn't sort, but HashMap iteration is random.
        // Let's sort by ID string for now to have deterministic order.
        t.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        t
    };

    // FIXME: This logic is duplicated in plan_page.rs and should be consolidated.
    let handle_doc_change = move |new_doc_id: tasklens_store::doc_id::DocumentId| {
        tracing::info!("Attempting to switch to Document ID: {}", new_doc_id);
        spawn(async move {
            // 1. Get repo without holding lock
            let repo = store.read().repo.clone();

            if let Some(repo) = repo {
                // 2. Perform async lookup detached from store instance
                match AppStore::find_doc(repo, new_doc_id.clone()).await {
                    Ok(Some(handle)) => {
                        // 3. Acquire lock ONLY for the sync update
                        store.write().set_active_doc(handle, new_doc_id.clone());
                        doc_id.set(Some(new_doc_id));
                        tracing::info!("Switch successful");
                    }
                    Ok(None) => {
                        tracing::error!("Document not found: {}", new_doc_id);
                    }
                    Err(e) => {
                        tracing::error!("Switch failed: {:?}", e);
                    }
                }
            } else {
                tracing::error!("Repo not initialized");
            }
        });
    };

    // FIXME: This logic is duplicated in plan_page.rs and should be consolidated.
    let handle_create_doc = move |_| {
        tracing::info!("Creating new document");
        spawn(async move {
            // 1. Get repo without holding lock
            let repo = store.read().repo.clone();

            if let Some(repo) = repo {
                // 2. Perform async creation detached from store instance
                match AppStore::create_new(repo).await {
                    Ok((handle, new_id)) => {
                        tracing::info!("Created new doc successfully: {}", new_id);

                        // 3. Acquire lock ONLY for the sync update
                        store.write().set_active_doc(handle, new_id.clone());
                        doc_id.set(Some(new_id));
                    }
                    Err(e) => tracing::error!("Failed to create doc: {:?}", e),
                }
            } else {
                tracing::error!("Repo not initialized");
            }
        });
    };

    rsx! {
        if show_settings() {
            SettingsModal {
                on_close: move |_| show_settings.set(false),
                doc_id,
                on_doc_change: handle_doc_change,
                on_create_doc: handle_create_doc,
            }
        }

        div {
            class: "container mx-auto max-w-md p-4",
            style: "padding-top: var(--safe-top); padding-bottom: var(--safe-bottom); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",
            Header {
                service_worker_active: service_worker_active(),
                on_settings_click: move |_| show_settings.set(true),
            }

            if let Some(error) = load_error() {
                LoadErrorView {
                    error,
                    help_text: Some(
                        "Access the settings menu to switch documents or change sync servers."
                            .to_string(),
                    ),
                }
            } else {
                TaskInput { value: input_text, on_add: move |_| add_task() }
                TaskList { tasks, on_toggle: toggle_task }
            }

            div { class: "mt-8 text-center text-base text-gray-500", "Build: {crate::BUILD_VERSION}" }
        }
    }
}

#[component]
fn Header(service_worker_active: bool, on_settings_click: EventHandler<()>) -> Element {
    let indicator_color = "bg-green-500";
    let indicator_title = "Sync Active";
    let indicator_class = format!("h-3 w-3 rounded-full {} mr-2", indicator_color);

    rsx! {
        div { class: "flex justify-between items-center mb-6",
            h1 { class: "text-2xl font-bold", "TaskLens" }
            div { class: "flex items-center space-x-2",
                div { class: "{indicator_class}", title: "{indicator_title}" }

                if service_worker_active {
                    div {
                        class: "px-2 py-0.5 text-xs font-semibold text-blue-800 bg-blue-100 rounded-full border border-blue-200 mr-2",
                        title: "Service Worker Active",
                        "Offline Ready"
                    }
                }

                button {
                    class: "text-gray-500 hover:text-gray-700 p-1 rounded-md hover:bg-gray-100",
                    onclick: move |_| on_settings_click.call(()),
                    svg {
                        class: "h-6 w-6",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z",
                        }
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskList(tasks: Vec<PersistedTask>, on_toggle: EventHandler<PersistedTask>) -> Element {
    rsx! {
        ul { class: "space-y-2",
            for task in tasks {
                TaskItem { key: "{task.id}", task: task.clone(), on_toggle }
            }
        }
    }
}

#[component]
fn TaskItem(task: PersistedTask, on_toggle: EventHandler<PersistedTask>) -> Element {
    let is_done = task.status == TaskStatus::Done;
    // We can't move 'task' into multiple closures, so we clone.
    // Actually EventHandler takes a value, so we clone for the call.
    let task_toggle = task.clone();
    let task_check = task.clone();
    rsx! {
        li {
            class: "flex items-center gap-2 p-3 border rounded cursor-pointer hover:bg-gray-50",
            onclick: move |_| on_toggle.call(task_toggle.clone()),
            Checkbox {
                checked: is_done,
                onchange: move |_| on_toggle.call(task_check.clone()),
                class: "cursor-pointer",
            }
            span { class: if is_done { "line-through text-gray-500" } else { "" }, "{task.title}" }
        }
    }
}

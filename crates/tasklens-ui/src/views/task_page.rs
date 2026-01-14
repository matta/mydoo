//! Main Task Application Views.
//!
//! This module contains the core application logic and UI components for the Task list functionality.
//! It handles state management, synchronization integration, and the composition of the main page.

use crate::components::*;
use crate::views::auth::SettingsModal;
use dioxus::prelude::*;
use futures::StreamExt;
use tasklens_core::types::{PersistedTask, TaskID, TaskStatus, TunnelState};
use tasklens_store::actions::{Action, TaskUpdates};
use tasklens_store::network;
use tasklens_store::store::AppStore;

/// The main application page component.
///
/// `TaskPage` allows the user to view, add, and toggle tasks. It also:
/// - Manages the local `AppStore` state.
/// - Handles hydration from persistent storage (IndexedDB).
/// - Orchestrates the background synchronization task `sync_task`.
/// - Integrates the `SettingsModal` for identity management.
#[component]
pub fn TaskPage(
    master_key: Signal<Option<[u8; 32]>>,
    service_worker_active: ReadSignal<bool>,
) -> Element {
    let mut store = use_signal(AppStore::new);
    let mut input_text = use_signal(String::new);
    let mut show_settings = use_signal(|| false);

    // Hydrate state from the store
    let state = use_memo(move || {
        store.read().hydrate::<TunnelState>().unwrap_or_else(|e| {
            // tracing::error!("Failed to hydrate state: {:?}", e);
            // Return empty state on failure or initial load
            TunnelState::default()
        })
    });

    // Load from storage on startup
    use_future(move || async move {
        match AppStore::load_from_db().await {
            Ok(Some(bytes)) => {
                tracing::info!("Loaded {} bytes from storage", bytes.len());
                store.write().load_from_bytes(bytes);
            }
            Ok(None) => tracing::info!("No persisted data found"),
            Err(e) => tracing::error!("Failed to load from storage: {:?}", e),
        }
    });

    // Sync Service Task
    let sync_task = use_coroutine(move |rx_local: UnboundedReceiver<Vec<u8>>| async move {
        // Create a channel for incoming (remote) changes
        let (tx_remote, rx_remote) = futures::channel::mpsc::unbounded();

        // Spawn a helper to apply remote changes to the store
        spawn(handle_remote_changes(rx_remote, store));

        network::run_sync_loop(
            rx_local,
            tx_remote,
            move || *master_key.read(),
            move || store.write().export_save(),
        )
        .await;
    });

    let mut save_and_sync = move || {
        let changes_opt = store.write().get_recent_changes();
        let bytes = store.write().export_save(); // Full save for persistence

        // 1. Persist
        spawn(async move {
            if let Err(e) = AppStore::save_to_db(bytes).await {
                tracing::error!("Failed to save: {:?}", e);
            }
        });

        // 2. Sync
        if let Some(change) = changes_opt {
            sync_task.send(change);
        }
    };

    let mut add_task = move || {
        let text = input_text();
        if text.trim().is_empty() {
            return;
        }

        if let Err(e) = store.write().dispatch(Action::CreateTask {
            parent_id: None,
            title: text.clone(),
        }) {
            tracing::error!("Failed to create task: {:?}", e);
            return;
        }

        save_and_sync();
        input_text.set(String::new());
    };

    let toggle_task = move |task: PersistedTask| {
        let new_status = match task.status {
            TaskStatus::Done => TaskStatus::Pending,
            TaskStatus::Pending => TaskStatus::Done,
        };

        // Use UpdateTask for Pending and CompleteTask for Done (canonical way, or just UpdateTask for both)
        // Store has a specific CompleteTask action, let's use it if status is Done.
        // For un-completing, we use UpdateTask.
        let result = match new_status {
            TaskStatus::Done => store.write().dispatch(Action::CompleteTask { id: task.id }),
            TaskStatus::Pending => store.write().dispatch(Action::UpdateTask {
                id: task.id,
                updates: TaskUpdates {
                    status: Some(TaskStatus::Pending),
                    ..Default::default()
                },
            }),
        };

        if let Err(e) = result {
            tracing::error!("Failed to toggle task: {:?}", e);
        } else {
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

    rsx! {
        if show_settings() {
            SettingsModal { master_key, on_close: move |_| show_settings.set(false) }
        }

        div {
            class: "container mx-auto max-w-md p-4",
            style: "padding-top: var(--safe-top); padding-bottom: var(--safe-bottom); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",
            Header {
                master_key_present: master_key().is_some(),
                service_worker_active: service_worker_active(),
                on_settings_click: move |_| show_settings.set(true),
            }

            TaskInput { value: input_text, on_add: move |_| add_task() }

            TaskList { tasks: tasks, on_toggle: toggle_task }

            div { class: "mt-8 text-center text-sm text-gray-500",
                "Build: {crate::BUILD_VERSION}"
            }
        }
    }
}

#[component]
fn Header(
    master_key_present: bool,
    service_worker_active: bool,
    on_settings_click: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "flex justify-between items-center mb-6",
            h1 { class: "text-2xl font-bold", "TaskLens" }
            div { class: "flex items-center space-x-2",
                div {
                    class: format!(
                        "h-3 w-3 rounded-full {} mr-2",
                        if master_key_present { "bg-green-500" } else { "bg-gray-300" },
                    ),
                    title: if master_key_present { "Sync Active" } else { "Local Only" }
                }

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
fn TaskInput(value: Signal<String>, on_add: EventHandler<()>) -> Element {
    let mut value = value;
    rsx! {
        div { class: "flex gap-2 mb-4",
            Input {
                value: "{value}",
                oninput: move |t| value.set(t),
                placeholder: "Add a new task...",
                class: "flex-grow mr-2",
                onkeypress: move |evt: KeyboardEvent| {
                    if evt.key() == Key::Enter {
                        on_add.call(());
                    }
                },
            }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| on_add.call(()),
                "Add"
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
            class: "flex items-center gap-2 p-2 border rounded cursor-pointer hover:bg-gray-50",
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

async fn handle_remote_changes(
    mut rx_remote: futures::channel::mpsc::UnboundedReceiver<Vec<u8>>,
    mut store: Signal<AppStore>,
) {
    while let Some(change) = rx_remote.next().await {
        store.write().import_changes(change);
    }
}

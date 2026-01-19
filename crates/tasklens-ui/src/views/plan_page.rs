use crate::components::TaskInput;
use crate::components::task_row::TaskRow;
use crate::controllers::task_controller;
use crate::utils::time_conversion::DEFAULT_LEAD_TIME_MS;
use crate::views::auth::SettingsModal;
use dioxus::prelude::*;
use tasklens_core::types::{PersistedTask, TaskID, TunnelState};
use tasklens_store::doc_id::DocumentId;
use tasklens_store::store::AppStore;

#[component]
pub fn PlanPage(focus_task: Option<TaskID>, seed: Option<bool>) -> Element {
    // let master_key = use_context::<Signal<Option<[u8; 32]>>>(); // Removed
    let mut store = use_context::<Signal<AppStore>>();
    let mut doc_id = use_context::<Signal<Option<DocumentId>>>();
    // let sync_tx = use_context::<Coroutine<Vec<u8>>>(); // Removed

    // ... rest of use_signals ...

    // Track expanded task IDs.
    let mut expanded_tasks: Signal<std::collections::HashSet<TaskID>> =
        use_signal(std::collections::HashSet::<TaskID>::new);
    let mut input_text = use_signal(String::new);
    let mut highlighted_task_id = use_signal(|| None::<TaskID>);
    let mut show_settings = use_signal(|| false);

    // FIXME: If tasks are created faster than 2s apart, multiple timers could be spawned.
    // Consider tracking the spawned task handle and cancelling it before spawning a new one.
    use_effect(move || {
        if highlighted_task_id().is_some() {
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(2000).await;
                highlighted_task_id.set(None);
            });
        }
    });

    #[derive(Debug, Clone, PartialEq)]
    enum EditorState {
        Edit(TaskID),
        Create { parent_id: Option<TaskID> },
    }
    let mut editor_state = use_signal(|| None::<EditorState>);

    // Handle focus_task (Find in Plan)
    use_effect(move || {
        if let Some(target_id) = focus_task.clone() {
            let state = store.read().hydrate::<TunnelState>().unwrap_or_default();
            let mut current_id = target_id.clone();
            let mut to_expand = Vec::new();

            // Walk up the tree
            while let Some(task) = state.tasks.get(&current_id) {
                if let Some(pid) = &task.parent_id {
                    to_expand.push(pid.clone());
                    current_id = pid.clone();
                } else {
                    break;
                }
            }

            if !to_expand.is_empty() {
                let mut expanded = expanded_tasks.write();
                for id in to_expand {
                    expanded.insert(id);
                }
            }

            // Optionally select the task to highlight it
            highlighted_task_id.set(Some(target_id));
        }
    });

    // Helper to trigger save (sync handled by hook polling)
    let trigger_sync = move || {
        // Explicit persist removed. Handled by use_persistence hook.
    };

    let flattened_tasks = use_memo(move || {
        let store = store.read();
        let state = store
            .hydrate::<TunnelState>()
            .unwrap_or_else(|_| TunnelState::default());

        let expanded = expanded_tasks.read();
        flatten_tasks(&state, &expanded)
    });

    let mut add_task = move || {
        let text = input_text();
        if text.trim().is_empty() {
            return;
        }

        if let Some(id) = task_controller::create_task(store, None, text) {
            highlighted_task_id.set(Some(id));
        }
        trigger_sync();
        input_text.set(String::new());
    };

    let toggle_task = move |task: PersistedTask| {
        task_controller::toggle_task_status(store, task.id);
        trigger_sync();
    };

    let handle_rename = move |(id, new_title): (TaskID, String)| {
        task_controller::rename_task(store, id, new_title);
        trigger_sync();
    };

    let handle_delete = move |id: TaskID| {
        task_controller::delete_task(store, id);
        trigger_sync();
    };

    let handle_create_subtask = move |parent_id: TaskID| {
        if let Some(id) =
            task_controller::create_task(store, Some(parent_id.clone()), "New Task".to_string())
        {
            highlighted_task_id.set(Some(id));
        }

        // Auto-expand the parent so we can see the child
        let mut expanded = expanded_tasks.write();
        expanded.insert(parent_id);

        trigger_sync();
    };

    let toggle_expand = move |id: TaskID| {
        let mut expanded = expanded_tasks.write();
        if expanded.contains(&id) {
            expanded.remove(&id);
        } else {
            expanded.insert(id);
        }
    };

    let on_title_tap = move |id: TaskID| {
        editor_state.set(Some(EditorState::Edit(id)));
    };

    let handle_task_created = move |id: TaskID| {
        highlighted_task_id.set(Some(id.clone()));

        // Check if it has a parent and expand it
        if let Ok(state) = store.read().hydrate::<TunnelState>()
            && let Some(task) = state.tasks.get(&id)
            && let Some(ref pid) = task.parent_id
        {
            let mut expanded = expanded_tasks.write();
            expanded.insert(pid.clone());
        }
    };

    let handle_doc_change = move |new_doc_id: DocumentId| {
        tracing::info!("Attempting to switch to Document ID: {}", new_doc_id);
        spawn(async move {
            // 1. Load without lock
            match AppStore::load_from_db(&new_doc_id).await {
                Ok(Some(bytes)) => {
                    // 2. Update with lock
                    {
                        let mut s = store.write();
                        s.current_id = new_doc_id.clone();
                        s.load_from_bytes(bytes);
                    }
                    // 3. Side effects
                    AppStore::save_active_doc_id(&new_doc_id);

                    tracing::info!("Switch successful");
                    doc_id.set(Some(new_doc_id));
                }
                Ok(None) => tracing::error!("Doc not found: {}", new_doc_id),
                Err(e) => tracing::error!("Switch failed: {:?}", e),
            }
        });
    };

    let handle_create_doc = move |_| {
        tracing::info!("Creating new document");
        spawn(async move {
            let mut s = store.write();
            match s.create_new() {
                Ok(new_id) => {
                    tracing::info!("Created new doc successfully: {}", new_id);
                    // Explicit save removed. Handled by use_persistence hook.
                    doc_id.set(Some(new_id));
                }
                Err(e) => tracing::error!("Failed to create doc: {:?}", e),
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
            class: "p-4 container mx-auto max-w-2xl",
            style: "padding-top: var(--safe-top); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",

            div { class: "flex justify-between items-center mb-6",
                h1 { class: "text-2xl font-bold", "Plan" }
                div { class: "flex items-center space-x-2",
                    if !flattened_tasks().is_empty() {
                        button {
                            class: "p-2 bg-blue-600 text-white rounded-md text-sm hover:bg-blue-700",
                            aria_label: "Add Task at Top",
                            onclick: move |_| {
                                editor_state
                                    .set(
                                        Some(EditorState::Create {
                                            parent_id: None,
                                        }),
                                    )
                            },
                            "Add Task"
                        }
                    }
                    button {
                        class: "text-gray-500 hover:text-gray-700 p-1 rounded-md hover:bg-gray-100",
                        onclick: move |_| show_settings.set(true),
                        aria_label: "Settings",
                        "data-testid": "settings-button",
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

            div { class: "mb-6",
                TaskInput {
                    data_testid: "plan-task-input",
                    value: input_text,
                    on_add: move |_| add_task(),
                }
            }

            div { class: "bg-white shadow rounded-lg overflow-hidden mt-4",
                if flattened_tasks().is_empty() {
                    div { class: "p-8 text-center",
                        p { class: "text-gray-500 mb-4",
                            "No tasks found. Try adding seed data? (?seed=true)"
                        }
                        button {
                            class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                            onclick: move |_| {
                                editor_state
                                    .set(
                                        Some(EditorState::Create {
                                            parent_id: None,
                                        }),
                                    )
                            },
                            "Add First Task"
                        }
                    }
                } else {
                    for FlattenedTask { task , depth , has_children , is_expanded , effective_due_date , effective_lead_time , .. } in flattened_tasks() {
                        TaskRow {
                            key: "{task.id}",
                            task: task.clone(),
                            depth,
                            on_toggle: toggle_task,
                            has_children,
                            is_expanded,
                            on_expand_toggle: toggle_expand,
                            on_rename: handle_rename,
                            on_delete: handle_delete,
                            on_create_subtask: handle_create_subtask,
                            on_title_tap,
                            is_highlighted: Some(task.id.clone()) == highlighted_task_id(),
                            effective_due_date,
                            effective_lead_time,
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
                            on_task_created: handle_task_created,
                        }
                    },
                    EditorState::Create { parent_id } => rsx! {
                        crate::components::TaskEditor {
                            initial_parent_id: parent_id,
                            on_close: move |_| editor_state.set(None),
                            on_task_created: handle_task_created,
                        }
                    },
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FlattenedTask {
    task: PersistedTask,
    depth: usize,
    has_children: bool,
    is_expanded: bool,
    effective_due_date: Option<i64>,
    effective_lead_time: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScheduleContext {
    due_date: Option<i64>,
    lead_time: Option<i64>,
}

struct FlattenContext<'a> {
    state: &'a TunnelState,
    expanded: &'a std::collections::HashSet<TaskID>,
    result: &'a mut Vec<FlattenedTask>,
}

fn flatten_tasks(
    state: &TunnelState,
    expanded: &std::collections::HashSet<TaskID>,
) -> Vec<FlattenedTask> {
    let mut result = Vec::new();
    let mut ctx = FlattenContext {
        state,
        expanded,
        result: &mut result,
    };
    for root_id in &state.root_task_ids {
        flatten_recursive(root_id, 0, &mut ctx, None);
    }
    result
}

fn flatten_recursive(
    id: &TaskID,
    depth: usize,
    ctx: &mut FlattenContext,
    parent_schedule: Option<ScheduleContext>,
) {
    if let Some(task) = ctx.state.tasks.get(id) {
        let has_children = !task.child_task_ids.is_empty();
        let is_expanded = ctx.expanded.contains(id);

        let effective_due_date = task
            .schedule
            .due_date
            .or(parent_schedule.and_then(|s| s.due_date));
        let effective_lead_time = task
            .schedule
            .lead_time
            .or(parent_schedule.and_then(|s| s.lead_time))
            .or(Some(DEFAULT_LEAD_TIME_MS));

        ctx.result.push(FlattenedTask {
            task: task.clone(),
            depth,
            has_children,
            is_expanded,
            effective_due_date,
            effective_lead_time,
        });

        if is_expanded {
            let next_schedule = Some(ScheduleContext {
                due_date: effective_due_date,
                lead_time: effective_lead_time,
            });
            for child_id in &task.child_task_ids {
                flatten_recursive(child_id, depth + 1, ctx, next_schedule);
            }
        }
    }
}

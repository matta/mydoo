use crate::components::TaskInput;
use crate::components::task_row::TaskRow;
use crate::controllers::task_controller;
use crate::utils::time_conversion::DEFAULT_LEAD_TIME_MS;
use dioxus::prelude::*;
use tasklens_core::types::{PersistedTask, TaskID, TunnelState};
use tasklens_store::store::AppStore;

#[component]
pub fn PlanPage(focus_task: Option<TaskID>) -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    let sync_tx = use_context::<Coroutine<Vec<u8>>>();

    // Track expanded task IDs.
    let mut expanded_tasks = use_signal(std::collections::HashSet::<TaskID>::new);
    let mut input_text = use_signal(String::new);
    let mut highlighted_task_id = use_signal(|| None::<TaskID>);

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
            let state = store.read().get_state().unwrap_or_default();
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

    let flattened_tasks = use_memo(move || {
        let store = store.read();
        let state = store
            .hydrate::<TunnelState>()
            .unwrap_or_else(|_| TunnelState::default());

        let expanded = expanded_tasks.read();
        flatten_tasks(&state, &expanded)
    });

    // Helper to trigger sync and save
    let mut trigger_sync = move || {
        let changes_opt = store.write().get_recent_changes();
        if let Some(changes) = changes_opt {
            sync_tx.send(changes);
            let bytes = store.write().export_save();
            spawn(async move {
                let _ = AppStore::save_to_db(bytes).await;
            });
        }
    };

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
        if let Ok(state) = store.read().get_state()
            && let Some(task) = state.tasks.get(&id)
            && let Some(ref pid) = task.parent_id
        {
            let mut expanded = expanded_tasks.write();
            expanded.insert(pid.clone());
        }
    };

    rsx! {
        div {
            class: "p-4 container mx-auto max-w-2xl",
            style: "padding-top: var(--safe-top); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",

            div { class: "flex justify-between items-center mb-6",
                h1 { class: "text-2xl font-bold", "Plan" }
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
                    for FlattenedTask { task, depth, has_children, is_expanded, effective_due_date, effective_lead_time, .. } in flattened_tasks() {
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
    effective_due_date: Option<f64>,
    effective_lead_time: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScheduleContext {
    due_date: Option<f64>,
    lead_time: Option<f64>,
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

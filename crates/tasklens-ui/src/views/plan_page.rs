use crate::app_components::TaskRow;
use crate::app_components::{LoadErrorView, PageHeader, TaskEditor, TaskInput};
use crate::controllers::task_controller;
use crate::dioxus_components::button::{Button, ButtonVariant};
use crate::dioxus_components::card::{Card, CardContent};
use crate::hooks::use_prioritized_tasks::{ScheduleLookup, use_schedule_lookup};
use dioxus::prelude::*;
use tasklens_core::types::{PersistedTask, TaskID, TaskStatus, TunnelState};

#[component]
pub fn PlanPage(focus_task: Option<TaskID>, seed: Option<bool>) -> Element {
    let task_controller = task_controller::use_task_controller();
    let load_error = use_context::<Signal<Option<String>>>();

    // Track expanded task IDs.
    let mut expanded_tasks: Signal<std::collections::HashSet<TaskID>> =
        use_signal(std::collections::HashSet::<TaskID>::new);
    let mut input_text = use_signal(String::new);
    let mut highlighted_task_id = use_signal(|| None::<TaskID>);

    // FIXME: If tasks are created faster than 2s apart, multiple timers could be spawned.
    // Consider tracking the spawned task handle and cancelling it before spawning a new one.
    use_effect(move || {
        if highlighted_task_id().is_some() {
            spawn(async move {
                crate::utils::async_utils::sleep(2000).await;
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

    let state = crate::hooks::use_tunnel_state::use_tunnel_state();

    // Handle focus_task (Find in Plan)
    use_effect({
        move || {
            if let Some(target_id) = focus_task.clone() {
                let state = state.read();
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
        }
    });

    // Helper to trigger save (sync handled by hook polling)
    let trigger_sync = move || {
        // Explicit persist removed. Handled by use_persistence hook.
    };

    let schedule_lookup = use_schedule_lookup();

    let flattened_tasks = use_memo({
        move || {
            let expanded = expanded_tasks.read();
            flatten_tasks(&state.read(), &expanded, &schedule_lookup.read())
        }
    });

    let mut add_task = {
        move || {
            let text = input_text();
            if text.trim().is_empty() {
                return;
            }

            if let Some(id) = task_controller.create(None, text) {
                highlighted_task_id.set(Some(id));
            }
            trigger_sync();
            input_text.set(String::new());
        }
    };

    let toggle_task = move |id: TaskID| {
        task_controller.toggle(id);
        trigger_sync();
    };

    let handle_rename = move |(id, new_title): (TaskID, String)| {
        task_controller.rename(id, new_title);
        trigger_sync();
    };

    let handle_delete = move |id: TaskID| {
        task_controller.delete(id);
        trigger_sync();
    };

    let handle_create_subtask = move |parent_id: TaskID| {
        if let Some(id) = task_controller.create(Some(parent_id.clone()), "New Task".to_string()) {
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
        // Use memoized state
        let state = state.read();
        if let Some(task) = state.tasks.get(&id)
            && let Some(ref pid) = task.parent_id
        {
            let mut expanded = expanded_tasks.write();
            expanded.insert(pid.clone());
        }
    };

    #[css_module("/src/views/plan_page.css")]
    struct Styles;

    rsx! {
        div {
            class: Styles::page_container,
            style: "padding-top: var(--safe-top); padding-left: max(1rem, var(--safe-left)); padding-right: max(1rem, var(--safe-right));",

            PageHeader { title: "Plan",
                if !flattened_tasks().is_empty() && load_error().is_none() {
                    Button {
                        variant: ButtonVariant::Primary,
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

            if let Some(error) = load_error() {
                LoadErrorView {
                    error,
                    help_text: Some(
                        "Access the settings menu to switch documents or change sync servers."
                            .to_string(),
                    ),
                }
            } else {
                div { class: Styles::input_section,
                    TaskInput {
                        data_testid: "plan-task-input",
                        value: input_text,
                        on_add: move |_| add_task(),
                    }
                }

                                Card {
                                    if flattened_tasks().is_empty() {
                                        CardContent {
                                            div { class: Styles::empty_state_container,
                                                p { class: Styles::empty_state_text,
                                                    "No tasks found yet."
                                                }
                                                Button {
                                                    variant: ButtonVariant::Primary,
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
                                        }
                                    } else {
                                        for FlattenedTask { id , title , status , depth , has_children , is_expanded , effective_due_date , effective_lead_time , .. } in flattened_tasks() {
                                            TaskRow {
                                                key: "{id}",
                                                id: id.clone(),
                                                title: title.clone(),
                                                status: status.clone(),
                                                depth,
                                                on_toggle: toggle_task,
                                                has_children,
                                                is_expanded,
                                                on_expand_toggle: toggle_expand,
                                                on_rename: handle_rename,
                                                on_delete: handle_delete,
                                                on_create_subtask: handle_create_subtask,
                                                on_title_tap,
                                                is_highlighted: Some(id.clone()) == highlighted_task_id(),
                                                effective_due_date,
                                                effective_lead_time,
                                            }
                                        }
                                    }
                                }            }

            if let Some(state) = editor_state() {
                match state {
                    EditorState::Edit(id) => rsx! {
                        TaskEditor {
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
                            on_task_created: handle_task_created,
                        }
                    },
                    EditorState::Create { parent_id } => rsx! {
                        TaskEditor {
                            key: "create-{parent_id:?}",
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
    id: TaskID,
    title: String,
    status: TaskStatus,
    depth: usize,
    has_children: bool,
    is_expanded: bool,
    effective_due_date: Option<i64>,
    effective_lead_time: Option<i64>,
}

struct FlattenContext<'a> {
    state: &'a TunnelState,
    expanded: &'a std::collections::HashSet<TaskID>,
    schedule_lookup: &'a ScheduleLookup,
    result: &'a mut Vec<FlattenedTask>,
}

fn flatten_tasks(
    state: &TunnelState,
    expanded: &std::collections::HashSet<TaskID>,
    schedule_lookup: &ScheduleLookup,
) -> Vec<FlattenedTask> {
    let mut result = Vec::new();
    let mut ctx = FlattenContext {
        state,
        expanded,
        schedule_lookup,
        result: &mut result,
    };
    for root_id in &state.root_task_ids {
        flatten_recursive(root_id, 0, &mut ctx);
    }
    result
}

fn flatten_recursive(id: &TaskID, depth: usize, ctx: &mut FlattenContext) {
    if let Some(task) = ctx.state.tasks.get(id) {
        let has_children = !task.child_task_ids.is_empty();
        let is_expanded = ctx.expanded.contains(id);

        // Lookup effective schedule from core algorithm (single source of truth).
        let (effective_due_date, effective_lead_time) =
            ctx.schedule_lookup.get(id).copied().unwrap_or_else(|| {
                tracing::warn!(
                    "Task {} not found in schedule_lookup, falling back to local.",
                    id
                );
                (None, Some(task.schedule.lead_time))
            });

        ctx.result.push(FlattenedTask {
            id: task.id.clone(),
            title: task.title.clone(),
            status: task.status,
            depth,
            has_children,
            is_expanded,
            effective_due_date,
            effective_lead_time,
        });

        if is_expanded {
            for child_id in &task.child_task_ids {
                flatten_recursive(child_id, depth + 1, ctx);
            }
        }
    }
}

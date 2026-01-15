use crate::components::button::{Button, ButtonVariant};
use crate::components::dialog::{DialogContent, DialogRoot, DialogTitle};
use crate::components::input::Input;
use crate::components::loading::Loading;
use crate::components::{DatePicker, Select};
use dioxus::prelude::*;
use tasklens_core::domain::constants::DEFAULT_LEAD_TIME_MILLIS;
use tasklens_core::types::{
    PersistedTask, PlaceID, RepeatConfig, Schedule, ScheduleType, TaskID, TaskStatus,
};
use tasklens_store::store::{Action, AppStore, TaskUpdates};

#[derive(Debug, Clone, PartialEq)]
pub struct DraftTask {
    pub title: String,
    pub notes: String,
    pub importance: f64,
    pub credit_increment: Option<f64>,
    pub place_id: Option<PlaceID>,
    pub status: TaskStatus,
    pub schedule: Schedule,
    pub repeat_config: Option<RepeatConfig>,
    pub is_sequential: bool,
}

impl From<PersistedTask> for DraftTask {
    fn from(task: PersistedTask) -> Self {
        Self {
            title: task.title,
            notes: task.notes,
            importance: task.importance,
            credit_increment: task.credit_increment,
            place_id: task.place_id,
            status: task.status,
            schedule: task.schedule,
            repeat_config: task.repeat_config,
            is_sequential: task.is_sequential,
        }
    }
}

#[component]
pub fn TaskEditor(
    task_id: Option<TaskID>,
    initial_parent_id: Option<TaskID>,
    on_close: EventHandler<()>,
    on_add_child: Option<EventHandler<TaskID>>,
) -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut draft = use_signal(|| None::<DraftTask>);
    let mut initialized = use_signal(|| false);

    // Initialize draft
    if !initialized() {
        if let Some(ref id) = task_id {
            let store_read = store.read();
            let task_ref = store_read
                .get_state()
                .ok()
                .and_then(|s| s.tasks.get(id).cloned());
            if let Some(task) = task_ref {
                draft.set(Some(DraftTask::from(task)));
            }
        } else {
            // Create Mode - Apply Defaults
            let parent_id = initial_parent_id.as_ref();
            let store_read = store.read();
            let (place_id, credit_increment) = if let Some(p_id) = parent_id {
                let parent = store_read
                    .get_state()
                    .ok()
                    .and_then(|s| s.tasks.get(p_id).cloned());
                if let Some(p) = parent {
                    (p.place_id, p.credit_increment)
                } else {
                    (None, Some(0.5))
                }
            } else {
                (None, Some(0.5))
            };

            draft.set(Some(DraftTask {
                title: "".to_string(),
                notes: "".to_string(),
                importance: 1.0,
                credit_increment,
                place_id,
                status: TaskStatus::Pending,
                schedule: Schedule {
                    schedule_type: ScheduleType::Once,
                    due_date: None,
                    lead_time: Some(DEFAULT_LEAD_TIME_MILLIS),
                    last_done: None,
                },
                repeat_config: None,
                is_sequential: false,
            }));
        }
        initialized.set(true);
    }

    if draft().is_none() {
        return rsx! {
            Loading {}
        };
    }

    let current_draft = draft().expect("draft should be initialized");

    let mut save_handler = {
        let mut store = store;
        let draft = draft;
        let task_id = task_id.clone();
        let initial_parent_id = initial_parent_id.clone();
        let on_close = on_close;
        move || {
            let d = draft().expect("draft should be initialized");
            let task_id_clone = task_id.clone();
            if let Some(id) = task_id_clone {
                // Update
                let _ = store.write().dispatch(Action::UpdateTask {
                    id,
                    updates: TaskUpdates {
                        title: Some(d.title),
                        status: Some(d.status),
                        place_id: Some(d.place_id),
                        due_date: Some(d.schedule.due_date),
                        schedule_type: Some(d.schedule.schedule_type),
                        lead_time: Some(d.schedule.lead_time),
                        repeat_config: Some(d.repeat_config),
                    },
                });
            } else {
                // Create
                let _ = store.write().dispatch(Action::CreateTask {
                    parent_id: initial_parent_id.clone(),
                    title: d.title,
                });
            }
            on_close.call(());
        }
    };

    let on_delete = {
        let mut store = store;
        let task_id = task_id.clone();
        let on_close = on_close;
        move |_| {
            let task_id_clone = task_id.clone();
            if let Some(id) = task_id_clone {
                let _ = store.write().dispatch(Action::DeleteTask { id });
            }
            on_close.call(());
        }
    };

    rsx! {
        DialogRoot { open: true, on_open_change: move |_| on_close.call(()),
            DialogContent { class: "task-editor-content",
                DialogTitle {
                    div { class: "flex justify-between items-center",
                        span {
                            if task_id.is_some() {
                                "Edit Task"
                            } else {
                                "Create Task"
                            }
                        }
                        if let Some(id) = task_id.clone() {
                            Button {
                                variant: ButtonVariant::Ghost,
                                class: "text-xs",
                                onclick: move |_| {
                                    let nav = navigator();
                                    nav.push(crate::router::Route::PlanPage {
                                        focus_task: Some(id.clone()),
                                    });
                                    on_close.call(());
                                },
                                "Find in Plan"
                            }
                        }
                    }
                }

                div { class: "task-editor-fields space-y-4 p-4",
                    // Title
                    div {
                        label {
                            class: "block text-sm font-medium",
                            r#for: "task-title-input",
                            "Title"
                        }
                        Input {
                            id: "task-title-input",
                            value: current_draft.title.clone(),
                            autofocus: true,
                            oninput: move |v| {
                                let mut d = draft().expect("draft should be initialized");
                                d.title = v;
                                draft.set(Some(d));
                            },
                            onkeydown: {
                                let mut save_handler = save_handler.clone();
                                move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        save_handler();
                                    }
                                }
                            },
                        }
                    }

                    // Importance
                    div {
                        label { class: "block text-sm font-medium",
                            "Importance: {current_draft.importance:.2}"
                        }
                        input {
                            r#type: "range",
                            class: "w-full",
                            min: 0.0,
                            max: 1.0,
                            step: 0.01,
                            value: current_draft.importance,
                            oninput: move |v| {
                                if let Ok(val) = v.value().parse::<f64>() {
                                    let mut d = draft().expect("draft should be initialized");
                                    d.importance = val;
                                    draft.set(Some(d));
                                }
                            },
                        }
                    }

                    // Effort / Credit Increment
                    div {
                        label { class: "block text-sm font-medium",
                            "Effort ({current_draft.credit_increment.unwrap_or(0.5):.2})"
                        }
                        input {
                            r#type: "range",
                            class: "w-full",
                            min: 0.0,
                            max: 1.0,
                            step: 0.01,
                            value: current_draft.credit_increment.unwrap_or(0.5),
                            oninput: move |v| {
                                if let Ok(val) = v.value().parse::<f64>() {
                                    let mut d = draft().expect("draft should be initialized");
                                    d.credit_increment = Some(val);
                                    draft.set(Some(d));
                                }
                            },
                        }
                    }

                    // Place
                    div {
                        label {
                            class: "block text-sm font-medium",
                            r#for: "place-select",
                            "Place"
                        }
                        {
                            let store_read = store.read();
                            let state = store_read.get_state().unwrap_or_default();
                            let places = state.places.values().collect::<Vec<_>>();

                            rsx! {
                                Select {
                                    id: "place-select",
                                    value: current_draft.place_id.clone().map(|id| id.to_string()).unwrap_or_default(),
                                    onchange: move |v: String| {
                                        let mut d = draft().expect("draft should be initialized");
                                        d.place_id = if v.is_empty() { None } else { Some(PlaceID::from(v)) };
                                        draft.set(Some(d));
                                    },
                                    option { value: "", "Anywhere" }
                                    for place in places {
                                        option { value: "{place.id}", "{place.name}" }
                                    }
                                }
                            }
                        }
                    }

                    // Schedule Type
                    div {
                        label {
                            class: "block text-sm font-medium",
                            r#for: "schedule-type-select",
                            "Schedule Type"
                        }
                        Select {
                            id: "schedule-type-select",
                            value: match current_draft.schedule.schedule_type {
                                ScheduleType::Once => "Once".to_string(),
                                ScheduleType::Routinely => "Routinely".to_string(),
                                ScheduleType::DueDate | ScheduleType::Calendar => "DueDate".to_string(),
                            },
                            onchange: move |v: String| {
                                let mut d = draft().expect("draft should be initialized");
                                let new_type = match v.as_str() {
                                    "Routinely" => ScheduleType::Routinely,
                                    "DueDate" => ScheduleType::DueDate,
                                    _ => ScheduleType::Once,
                                };
                                d.schedule.schedule_type = new_type;
                                draft.set(Some(d));
                            },
                            option { value: "Once", "Once" }
                            option { value: "Routinely", "Routinely" }
                            option { value: "DueDate", "Due Date" }
                        }
                    }
                    // Routine Repetition (Conditional)
                    if matches!(current_draft.schedule.schedule_type, ScheduleType::Routinely) {
                        div { class: "p-3 bg-blue-50 rounded-md border border-blue-100 space-y-3",
                            div {
                                label {
                                    class: "block text-sm font-medium text-blue-800",
                                    r#for: "repetition-frequency-select",
                                    "Repeat Every"
                                }
                                div { class: "flex gap-2",
                                    input {
                                        r#type: "number",
                                        id: "repetition-interval-input",
                                        class: "w-20 border rounded p-1 text-sm",
                                        value: current_draft.repeat_config.as_ref().map(|r| r.interval).unwrap_or(1.0),
                                        oninput: move |e| {
                                            if let Ok(val) = e.value().parse::<f64>() {
                                                let mut d = draft().expect("draft should be initialized");
                                                let mut config = d
                                                    .repeat_config
                                                    .unwrap_or(RepeatConfig {
                                                        frequency: tasklens_core::types::Frequency::Daily,
                                                        interval: 1.0,
                                                    });
                                                config.interval = val;
                                                d.repeat_config = Some(config);
                                                draft.set(Some(d));
                                            }
                                        },
                                    }
                                    select {
                                        id: "repetition-frequency-select",
                                        class: "flex-grow border rounded p-1 text-sm",
                                        value: current_draft
                                            .repeat_config
                                            .as_ref()
                                            .map(|r| match r.frequency {
                                                tasklens_core::types::Frequency::Minutes => "Minutes",
                                                tasklens_core::types::Frequency::Hours => "Hours",
                                                tasklens_core::types::Frequency::Daily => "Daily",
                                                tasklens_core::types::Frequency::Weekly
                                                | tasklens_core::types::Frequency::Monthly
                                                | tasklens_core::types::Frequency::Yearly => "Daily",
                                            })
                                            .unwrap_or("Daily"),
                                        onchange: move |e| {
                                            let mut d = draft().expect("draft should be initialized");
                                            let mut config = d
                                                .repeat_config
                                                .unwrap_or(RepeatConfig {
                                                    frequency: tasklens_core::types::Frequency::Daily,
                                                    interval: 1.0,
                                                });
                                            config.frequency = match e.value().as_str() {
                                                "Minutes" => tasklens_core::types::Frequency::Minutes,
                                                "Hours" => tasklens_core::types::Frequency::Hours,
                                                _ => tasklens_core::types::Frequency::Daily,
                                            };
                                            d.repeat_config = Some(config);
                                            draft.set(Some(d));
                                        },
                                        option { value: "Minutes", "Minutes" }
                                        option { value: "Hours", "Hours" }
                                        option { value: "Daily", "Daily" }
                                    }
                                }
                            }
                        }
                    }

                    // Due Date (Conditional)
                    if matches!(
                        current_draft.schedule.schedule_type,
                        ScheduleType::DueDate | ScheduleType::Calendar
                    )
                    {
                        div {
                            label { class: "block text-sm font-medium", "Due Date" }
                            DatePicker {
                                value: current_draft
                                    // Basic timestamp to YYYY-MM-DD conversion
                                    // Use a lightweight formatter or string manipulation if chrono isn't available/convenient
                                    // For now, let's just use an empty string as a placeholder to unblock compilation
                                    // TODO: Implement proper date formatting
                                    .schedule
                                    .due_date
                                    .map(|ts| {
                                        let _secs = (ts / 1000.0) as i64;
                                        String::new()
                                    }),
                                onchange: move |_v: String| {},
                            }
                        }
                    }

                    // Lead Time
                    div {
                        label { class: "block text-sm font-medium", "Lead Time" }
                        div { class: "flex gap-2",
                            input {
                                r#type: "number",
                                id: "lead-time-scalar-input",
                                class: "w-20 border rounded p-1 text-sm",
                                value: current_draft.schedule.lead_time.unwrap_or(28_800_000.0) / 3_600_000.0,
                                oninput: move |e| {
                                    if let Ok(val) = e.value().parse::<f64>() {
                                        let mut d = draft().expect("draft should be initialized");
                                        d.schedule.lead_time = Some(val * 3_600_000.0);
                                        draft.set(Some(d));
                                    }
                                },
                            }
                            select {
                                id: "lead-time-unit-select",
                                class: "border rounded p-1 text-sm",
                                value: "Hours",
                                onchange: move |_| {},
                                option { value: "Hours", "Hours" }
                                option { value: "Days", "Days" }
                            }
                        }
                    }

                    // Notes
                    div {
                        label { class: "block text-sm font-medium", "Notes" }
                        textarea {
                            class: "w-full border rounded p-2 text-sm",
                            rows: 4,
                            value: current_draft.notes.clone(),
                            oninput: move |e| {
                                let mut d = draft().expect("draft should be initialized");
                                d.notes = e.value();
                                draft.set(Some(d));
                            },
                        }
                    }

                    // Footer Actions
                    div { class: "flex justify-between items-center pt-4 border-t",
                        div { class: "flex gap-2",
                            if task_id.is_some() {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    onclick: {
                                        let task_id = task_id.clone();
                                        let on_add_child = on_add_child;
                                        move |_| {
                                            if let (Some(id), Some(handler)) = (task_id.clone(), on_add_child.as_ref()) {
                                                handler.call(id);
                                            }
                                        }
                                    },
                                    "Add Child"
                                }
                            }
                        }
                        div { class: "flex gap-2",
                            if task_id.is_some() {
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: on_delete,
                                    "Delete"
                                }
                            }
                            Button {
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| on_close.call(()),
                                "Cancel"
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| save_handler(),
                                if task_id.is_some() {
                                    "Save Changes"
                                } else {
                                    "Create Task"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

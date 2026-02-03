use crate::components::button::{Button, ButtonVariant};
use crate::components::dialog::{DialogContent, DialogRoot, DialogTitle};
use crate::components::input::Input;
use crate::components::loading::Loading;
use crate::components::{DatePicker, Select};
use crate::utils::time_conversion;
use dioxus::prelude::*;
use tasklens_core::TaskUpdates;
use tasklens_core::domain::constants::DEFAULT_LEAD_TIME_MILLIS;
use tasklens_core::types::{
    Frequency, PersistedTask, PlaceID, RepeatConfig, Schedule, ScheduleType, TaskID, TaskStatus,
};
use tracing;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DraftTask {
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
    on_task_created: Option<EventHandler<TaskID>>,
) -> Element {
    let task_controller = crate::controllers::task_controller::use_task_controller();
    let mut draft = use_signal(|| None::<DraftTask>);
    let mut initialized = use_signal(|| false);
    let state = crate::hooks::use_tunnel_state::use_tunnel_state();

    // Initialize draft
    if !initialized() {
        if let Some(ref id) = task_id {
            let task_ref = state().tasks.get(id).cloned();
            if let Some(task) = task_ref {
                draft.set(Some(DraftTask::from(task)));
            }
        } else {
            // Create Mode - Apply Defaults
            let parent_id = initial_parent_id.as_ref();
            let (place_id, credit_increment) = if let Some(p_id) = parent_id {
                let parent = state().tasks.get(p_id).cloned();
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
                    lead_time: DEFAULT_LEAD_TIME_MILLIS,
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

    let mut show_move_picker = use_signal(|| false);

    let can_outdent = if let Some(id) = task_id.as_ref() {
        state()
            .tasks
            .get(id)
            .map(|t| t.parent_id.is_some())
            .unwrap_or(false)
    } else {
        false
    };

    let can_indent = if let Some(id) = task_id.as_ref() {
        tasklens_core::domain::hierarchy::get_previous_sibling(&state(), id).is_some()
    } else {
        false
    };

    let save_handler = {
        let draft = draft;
        let task_id = task_id.clone();
        let initial_parent_id = initial_parent_id.clone();
        let on_close = on_close;
        move || {
            let d = draft().expect("draft should be initialized");
            let task_id_clone = task_id.clone();
            if let Some(id) = task_id_clone {
                // Update
                task_controller.update(
                    id,
                    TaskUpdates {
                        title: Some(d.title),
                        notes: Some(d.notes),
                        status: Some(d.status),
                        place_id: Some(d.place_id),
                        due_date: Some(d.schedule.due_date),
                        schedule_type: Some(d.schedule.schedule_type),
                        lead_time: Some(d.schedule.lead_time),
                        repeat_config: Some(d.repeat_config),
                        is_sequential: Some(d.is_sequential),
                        importance: Some(d.importance),
                        credit_increment: d.credit_increment,
                        ..Default::default()
                    },
                );
            } else if let Some(id) =
                task_controller.create(initial_parent_id.clone(), d.title.clone())
            {
                // TODO: it seems weird to call a handler here and then update the task
                // immediately after.
                if let Some(handler) = on_task_created.as_ref() {
                    handler.call(id.clone());
                }
                // After creation, update with other draft fields
                task_controller.update(
                    id,
                    TaskUpdates {
                        title: Some(d.title),
                        notes: Some(d.notes),
                        status: Some(d.status),
                        place_id: Some(d.place_id),
                        due_date: Some(d.schedule.due_date),
                        schedule_type: Some(d.schedule.schedule_type),
                        lead_time: Some(d.schedule.lead_time),
                        repeat_config: Some(d.repeat_config),
                        is_sequential: Some(d.is_sequential),
                        importance: Some(d.importance),
                        credit_increment: d.credit_increment,
                        ..Default::default()
                    },
                );
            }
            on_close.call(());
        }
    };

    let on_delete = {
        let task_id = task_id.clone();
        let on_close = on_close;
        move |_| {
            let task_id_clone = task_id.clone();
            if let Some(id) = task_id_clone {
                task_controller.delete(id);
            }
            on_close.call(());
        }
    };

    rsx! {
        DialogRoot { open: true, on_open_change: move |_| on_close.call(()),
            DialogContent { class: "task-editor-content",
                DialogTitle {
                    if task_id.is_some() {
                        "Edit Task"
                    } else {
                        "Create Task"
                    }
                }

                if let Some(id) = task_id.clone() {
                    div { class: "flex justify-end px-4 -mt-8 mb-4",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "text-xs",
                            onclick: move |_| {
                                let nav = navigator();
                                nav.push(crate::router::Route::PlanPage {
                                    focus_task: Some(id.clone()),
                                    seed: None,
                                });
                                on_close.call(());
                            },
                            "Find in Plan"
                        }
                    }
                }

                div { class: "task-editor-fields space-y-4 p-4",
                    // Title
                    div {
                        label {
                            class: "block text-base font-medium",
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
                                let save_handler = save_handler.clone();
                                move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        save_handler();
                                    }
                                }
                            },
                        }
                    }

                    div {
                        label {
                            class: "block text-base font-medium",
                            r#for: "importance-input",
                            "Importance: {current_draft.importance:.2}"
                        }
                        input {
                            r#type: "range",
                            id: "importance-input",
                            class: "range range-primary range-sm",
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
                        label {
                            class: "block text-base font-medium",
                            r#for: "effort-input",
                            "Effort ({current_draft.credit_increment.unwrap_or(0.5):.2})"
                        }
                        input {
                            r#type: "range",
                            id: "effort-input",
                            class: "range range-primary range-sm",
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
                            class: "block text-base font-medium",
                            r#for: "place-select",
                            "Place"
                        }
                        {
                            let state_val = state();
                            let places = state_val.places.values().collect::<Vec<_>>();

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
                            class: "block text-base font-medium",
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
                                    "Once" => ScheduleType::Once,
                                    "Routinely" => ScheduleType::Routinely,
                                    "DueDate" => ScheduleType::DueDate,
                                    _ => ScheduleType::Once,
                                };
                                d.schedule.schedule_type = new_type;

                                // Initialize/Clear repeat_config based on type
                                if matches!(new_type, ScheduleType::Routinely) {
                                    if d.repeat_config.is_none() {
                                        d.repeat_config = Some(RepeatConfig {
                                            frequency: Frequency::Daily,
                                            interval: 1,
                                        });
                                    }
                                } else {
                                    d.repeat_config = None;
                                }

                                draft.set(Some(d));
                            },
                            option { value: "Once", "Once" }
                            option { value: "Routinely", "Routinely" }
                            option { value: "DueDate", "Due Date" }
                        }
                    }
                    if matches!(current_draft.schedule.schedule_type, ScheduleType::Routinely) {
                        div { class: "p-3 bg-base-200 rounded-md border border-base-300 space-y-3",
                            div {
                                label {
                                    class: "block text-base font-medium text-primary",
                                    r#for: "repetition-frequency-select",
                                    "Repeat Every"
                                }
                                div { class: "flex gap-2",
                                    input {
                                        r#type: "number",
                                        id: "repetition-interval-input",
                                        class: "input input-bordered w-20 px-2 text-base",
                                        value: current_draft.repeat_config.as_ref().map(|r| r.interval).unwrap_or(1),
                                        oninput: move |e| {
                                            if let Ok(val) = e.value().parse::<i64>() {
                                                let mut d = draft().expect("draft should be initialized");
                                                let mut config = d
                                                    .repeat_config
                                                    .unwrap_or(RepeatConfig {
                                                        frequency: tasklens_core::types::Frequency::Daily,
                                                        interval: 1,
                                                    });
                                                config.interval = val;
                                                d.repeat_config = Some(config);
                                                draft.set(Some(d));
                                            }
                                        },
                                    }
                                    select {
                                        id: "repetition-frequency-select",
                                        class: "select select-bordered flex-grow text-base",
                                        value: current_draft
                                            .repeat_config
                                            .as_ref()
                                            .map(|r| match r.frequency {
                                                Frequency::Minutes => "Minutes",
                                                Frequency::Hours => "Hours",
                                                Frequency::Daily => "Daily",
                                                Frequency::Weekly => "Weekly",
                                                Frequency::Monthly => "Monthly",
                                                Frequency::Yearly => "Yearly",
                                            })
                                            .unwrap_or("Daily"),
                                        onchange: move |e| {
                                            let mut d = draft().expect("draft should be initialized");
                                            let mut config = d
                                                .repeat_config
                                                .unwrap_or(RepeatConfig {
                                                    frequency: Frequency::Daily,
                                                    interval: 1,
                                                });
                                            config.frequency = match e.value().as_str() {
                                                "Minutes" => Frequency::Minutes,
                                                "Hours" => Frequency::Hours,
                                                "Weekly" => Frequency::Weekly,
                                                "Monthly" => Frequency::Monthly,
                                                "Yearly" => Frequency::Yearly,
                                                _ => Frequency::Daily,
                                            };
                                            d.repeat_config = Some(config);
                                            draft.set(Some(d));
                                        },
                                        option { value: "Minutes", "Minutes" }
                                        option { value: "Hours", "Hours" }
                                        option { value: "Daily", "Daily" }
                                        option { value: "Weekly", "Weekly" }
                                        option { value: "Monthly", "Monthly" }
                                        option { value: "Yearly", "Yearly" }
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
                            label { class: "block text-base font-medium", "Due Date" }
                            DatePicker {
                                data_testid: "date-input",
                                value: current_draft
                                    .schedule
                                    .due_date
                                    .map(|ts| {
                                        use chrono::TimeZone;
                                        let secs = ts / 1000;
                                        if let Some(dt) = chrono::Utc.timestamp_opt(secs, 0).single() {
                                            dt.format("%Y-%m-%d").to_string()
                                        } else {
                                            String::new()
                                        }
                                    }),
                                onchange: move |v: String| {
                                    if let Ok(date) = chrono::NaiveDate::parse_from_str(&v, "%Y-%m-%d")
                                        && let Some(dt) = date.and_hms_opt(0, 0, 0)
                                    {
                                        let ts = dt.and_utc().timestamp_millis();
                                        let mut d = draft().expect("draft should be initialized");
                                        d.schedule.due_date = Some(ts);
                                        draft.set(Some(d));
                                    } else if !v.is_empty() {
                                        tracing::warn!("Failed to parse date: {}", v);
                                    }
                                },
                            }
                        }
                    }

                    // Lead Time
                    div {
                        label {
                            class: "block text-base font-medium",
                            r#for: "lead-time-scalar-input",
                            "Lead Time"
                        }
                        div { class: "flex gap-2",
                            {
                                let lead_time_ms = current_draft.schedule.lead_time;
                                let (val, unit) = time_conversion::ms_to_period(lead_time_ms);
                                let unit_clone = unit.clone();
                                rsx! {
                                    input {
                                        r#type: "number",
                                        id: "lead-time-scalar-input",
                                        class: "input input-bordered w-20 px-2 text-base",
                                        value: "{val}",
                                        oninput: move |e| {
                                            if let Ok(val) = e.value().parse::<u32>() {
                                                let mut d = draft().expect("draft should be initialized");
                                                d.schedule.lead_time = time_conversion::period_to_ms(val, &unit_clone);
                                                draft.set(Some(d));
                                            }
                                        },
                                    }
                                    select {
                                        id: "lead-time-unit-select",
                                        class: "select select-bordered text-base",
                                        value: "{unit}",
                                        aria_label: "Lead Time Unit",
                                        onchange: move |e| {
                                            let mut d = draft().expect("draft should be initialized");
                                            let (val, _) = time_conversion::ms_to_period(d.schedule.lead_time);
                                            d.schedule.lead_time = time_conversion::period_to_ms(val, &e.value());
                                            draft.set(Some(d));
                                        },
                                        option { value: "Hours", "Hours" }
                                        option { value: "Days", "Days" }
                                        option { value: "Weeks", "Weeks" }
                                    }
                                }
                            }
                        }
                    }

                    // Notes
                    div {
                        label {
                            class: "block text-base font-medium",
                            r#for: "notes-input",
                            "Notes"
                        }
                        textarea {
                            id: "notes-input",
                            class: "textarea textarea-bordered w-full text-base",
                            rows: 4,
                            value: current_draft.notes.clone(),
                            oninput: move |e| {
                                let mut d = draft().expect("draft should be initialized");
                                d.notes = e.value();
                                draft.set(Some(d));
                            },
                        }
                    }

                    if task_id.is_some() {
                        div { class: "flex items-center gap-2 p-3 bg-base-200 rounded-md border border-base-300",
                            input {
                                r#type: "checkbox",
                                id: "sequential-toggle",
                                class: "checkbox checkbox-primary",
                                checked: current_draft.is_sequential,
                                aria_label: "Sequential Project",
                                onchange: move |e| {
                                    let mut d = draft().expect("draft should be initialized");
                                    d.is_sequential = e.checked();
                                    draft.set(Some(d));
                                },
                            }
                            label {
                                r#for: "sequential-toggle",
                                class: "text-base font-medium cursor-pointer",
                                "Sequential Project"
                            }
                            span { class: "text-xs text-gray-500 ml-auto", "Do steps in order" }
                        }
                    }

                    // Footer Actions
                    div { class: "flex flex-col sm:flex-row justify-between items-center pt-4 border-t gap-4",
                        div { class: "flex flex-wrap gap-2 justify-center sm:justify-start",
                            if let Some(id) = task_id.clone() {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    onclick: move |_| show_move_picker.set(true),
                                    "Move..."
                                }
                                if can_outdent {
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        onclick: {
                                            let id = id.clone();
                                            move |_| {
                                                task_controller.outdent(id.clone());
                                                on_close.call(());
                                            }
                                        },
                                        "← Outdent"
                                    }
                                }
                                if can_indent {
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        onclick: {
                                            let id = id.clone();
                                            move |_| {
                                                task_controller.indent(id.clone());
                                                on_close.call(());
                                            }
                                        },
                                        "Indent →"
                                    }
                                }
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    onclick: {
                                        let id = id.clone();
                                        let on_add_child = on_add_child;
                                        move |_| {
                                            if let Some(handler) = on_add_child.as_ref() {
                                                handler.call(id.clone());
                                            }
                                        }
                                    },
                                    "Add Child"
                                }
                            }
                        }
                        div { class: "flex flex-wrap gap-2 justify-center sm:justify-end",
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
        // Render MovePicker as a sibling dialog outside TaskEditor's DialogRoot
        if show_move_picker() {
            if let Some(id) = task_id.clone() {
                crate::components::MovePicker {
                    task_id: id.clone(),
                    on_select: move |new_parent_id| {
                        task_controller.move_item(id.clone(), new_parent_id);
                        show_move_picker.set(false);
                        on_close.call(());
                    },
                    on_close: move |_| show_move_picker.set(false),
                }
            }
        }
    }
}

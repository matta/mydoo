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
    let state = crate::hooks::use_tunnel_state::use_tunnel_state();

    // Initialize draft
    if draft().is_none() {
        if let Some(ref id) = task_id {
            let task_ref = state().tasks.get(id).cloned();
            if let Some(task) = task_ref {
                draft.set(Some(DraftTask::from(task)));
            }
        } else {
            // Create Mode - Apply Defaults
            let parent_id = initial_parent_id.as_ref();
            let (place_id, credit_increment, lead_time) = if let Some(p_id) = parent_id {
                let parent = state().tasks.get(p_id).cloned();
                if let Some(p) = parent {
                    (p.place_id, p.credit_increment, p.schedule.lead_time)
                } else {
                    (None, Some(0.5), DEFAULT_LEAD_TIME_MILLIS)
                }
            } else {
                (None, Some(0.5), DEFAULT_LEAD_TIME_MILLIS)
            };

            draft.set(Some(DraftTask {
                title: "".to_string(),
                notes: "".to_string(),
                importance: 0.5,
                credit_increment,
                place_id,
                status: TaskStatus::Pending,
                schedule: Schedule {
                    schedule_type: ScheduleType::Once,
                    due_date: None,
                    lead_time,
                    last_done: None,
                },
                repeat_config: None,
                is_sequential: false,
            }));
        }
    }

    if draft().is_none() {
        return rsx! {
            Loading {}
        };
    }

    let current_draft = draft().expect("draft should be initialized");

    let mut show_move_picker = use_signal(|| false);

    // Determine parent_id: for edit mode, get from the task; for create mode, use initial_parent_id
    let parent_id = if let Some(id) = task_id.as_ref() {
        state().tasks.get(id).and_then(|t| t.parent_id.clone())
    } else {
        initial_parent_id.clone()
    };

    // Look up parent title for display
    let parent_title = parent_id.as_ref().and_then(|pid| {
        let s = state();
        s.tasks.get(pid).map(|p| p.title.clone())
    });

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
            DialogContent { class: "w-full max-w-2xl p-0 overflow-hidden bg-base-100",
                // Modal Header (Sticky)
                div { class: "px-6 py-4 border-b border-base-200 flex justify-between items-center bg-base-100 sticky top-0 z-20",
                    div { class: "flex flex-col gap-1",
                        DialogTitle {
                            if task_id.is_some() {
                                "Edit Task"
                            } else {
                                "Create Task"
                            }
                        }
                        if let Some(ref title) = parent_title {
                            span { class: "text-xs opacity-50 font-medium", "Subtask of {title}" }
                        }
                    }

                    if let Some(id) = task_id.clone() {
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "btn-sm btn-circle hover:bg-base-200 transition-colors",
                            onclick: move |_| {
                                let nav = navigator();
                                nav.push(crate::router::Route::PlanPage {
                                    focus_task: Some(id.clone()),
                                    seed: None,
                                });
                                on_close.call(());
                            },
                            span { class: "sr-only", "Find in Plan" }
                            svg {
                                "fill": "none",
                                "viewBox": "0 0 24 24",
                                "stroke-width": "2",
                                "stroke": "currentColor",
                                class: "size-5",
                                path {
                                    "stroke-linecap": "round",
                                    "stroke-linejoin": "round",
                                    "d": "m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z",
                                }
                            }
                        }
                    }
                }

                // Modal Body (Scrollable)
                div { class: "px-6 py-6 overflow-y-auto max-h-[70vh] flex flex-col gap-10",
                    // Section: Core Details
                    fieldset { class: "fieldset !p-0",
                        legend { class: "fieldset-legend text-primary uppercase text-[10px] tracking-widest font-bold",
                            "Task Details"
                        }
                        div { class: "w-full space-y-4",
                            div {
                                label {
                                    class: "label p-0 mb-1.5",
                                    r#for: "task-title-input",
                                    span { class: "label-text font-semibold", "Title" }
                                }
                                Input {
                                    id: "task-title-input",
                                    value: current_draft.title.clone(),
                                    autofocus: true,
                                    class: "input-lg w-full",
                                    oninput: move |v| {
                                        draft
                                            .with_mut(|d_opt| {
                                                if let Some(d) = d_opt.as_mut() {
                                                    d.title = v;
                                                }
                                            });
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
                                    class: "label p-0 mb-1.5",
                                    r#for: "notes-input",
                                    span { class: "label-text font-semibold", "Notes" }
                                }
                                textarea {
                                    id: "notes-input",
                                    class: "textarea textarea-bordered w-full h-32 text-base leading-relaxed p-4",
                                    placeholder: "Add more context or details here...",
                                    value: current_draft.notes.clone(),
                                    oninput: move |e| {
                                        draft
                                            .with_mut(|d_opt| {
                                                if let Some(d) = d_opt.as_mut() {
                                                    d.notes = e.value();
                                                }
                                            });
                                    },
                                }
                            }
                        }
                    }

                    // Section: Prioritization & Location (Grid)
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-8",
                        fieldset { class: "fieldset bg-base-200/40 p-5 rounded-box",
                            legend { class: "fieldset-legend text-primary uppercase text-[10px] tracking-widest font-bold",
                                "Prioritization"
                            }
                            div { class: "space-y-6 w-full",
                                div {
                                    label {
                                        class: "label p-0 mb-2 flex justify-between",
                                        r#for: "importance-input",
                                        span { class: "label-text font-medium",
                                            "Importance: "
                                            span { class: "label-text-alt font-mono opacity-70",
                                                "{current_draft.importance:.2}"
                                            }
                                        }
                                    }
                                    input {
                                        r#type: "range",
                                        id: "importance-input",
                                        class: "range range-primary range-xs",
                                        min: 0.0,
                                        max: 1.0,
                                        step: 0.1,
                                        value: current_draft.importance,
                                        oninput: move |v| {
                                            if let Ok(val) = v.value().parse::<f64>() {
                                                draft
                                                    .with_mut(|d_opt| {
                                                        if let Some(d) = d_opt.as_mut() {
                                                            d.importance = val;
                                                        }
                                                    });
                                            }
                                        },
                                    }
                                }

                                div {
                                    label {
                                        class: "label p-0 mb-2 flex justify-between",
                                        r#for: "effort-input",
                                        span { class: "label-text font-medium",
                                            "Effort: "
                                            span { class: "label-text-alt font-mono opacity-70",
                                                "{current_draft.credit_increment.unwrap_or(0.5):.2}"
                                            }
                                        }
                                    }
                                    input {
                                        r#type: "range",
                                        id: "effort-input",
                                        class: "range range-primary range-xs",
                                        min: 0.0,
                                        max: 1.0,
                                        step: 0.1,
                                        value: current_draft.credit_increment.unwrap_or(0.5),
                                        oninput: move |v| {
                                            if let Ok(val) = v.value().parse::<f64>() {
                                                draft
                                                    .with_mut(|d_opt| {
                                                        if let Some(d) = d_opt.as_mut() {
                                                            d.credit_increment = Some(val);
                                                        }
                                                    });
                                            }
                                        },
                                    }
                                }
                            }
                        }

                        fieldset { class: "fieldset bg-base-200/40 p-5 rounded-box",
                            legend { class: "fieldset-legend text-primary uppercase text-[10px] tracking-widest font-bold",
                                "Location"
                            }
                            div { class: "w-full",
                                label {
                                    class: "label p-0 mb-2",
                                    r#for: "place-select",
                                    span { class: "label-text font-medium", "Place" }
                                }
                                {
                                    let state_val = state();
                                    let places = state_val.places.values().collect::<Vec<_>>();

                                    rsx! {
                                        Select {
                                            id: "place-select",
                                            value: current_draft.place_id.clone().map(|id| id.to_string()).unwrap_or_default(),
                                            onchange: move |v: String| {
                                                draft
                                                    .with_mut(|d_opt| {
                                                        if let Some(d) = d_opt.as_mut() {
                                                            d.place_id = if v.is_empty() {
                                                                None
                                                            } else {
                                                                Some(PlaceID::from(v))
                                                            };
                                                        }
                                                    });
                                            },
                                            option { value: "", "Anywhere" }
                                            for place in places {
                                                option { value: "{place.id}", "{place.name}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Section: Scheduling
                    fieldset { class: "fieldset bg-base-200/40 p-5 rounded-box",
                        legend { class: "fieldset-legend text-primary uppercase text-[10px] tracking-widest font-bold",
                            "Scheduling"
                        }
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-6 w-full",
                            div {
                                label {
                                    class: "label p-0 mb-2",
                                    r#for: "schedule-type-select",
                                    span { class: "label-text font-medium", "Schedule Type" }
                                }
                                Select {
                                    id: "schedule-type-select",
                                    value: match current_draft.schedule.schedule_type {
                                        ScheduleType::Once => "Once".to_string(),
                                        ScheduleType::Routinely => "Routinely".to_string(),
                                        ScheduleType::DueDate | ScheduleType::Calendar => "DueDate".to_string(),
                                    },
                                    onchange: move |v: String| {
                                        let new_type = match v.as_str() {
                                            "Once" => ScheduleType::Once,
                                            "Routinely" => ScheduleType::Routinely,
                                            "DueDate" => ScheduleType::DueDate,
                                            _ => ScheduleType::Once,
                                        };
                                        draft
                                            .with_mut(|d_opt| {
                                                if let Some(d) = d_opt.as_mut() {
                                                    d.schedule.schedule_type = new_type;
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
                                                }
                                            });
                                    },
                                    option { value: "Once", "Once" }
                                    option { value: "Routinely", "Routinely" }
                                    option { value: "DueDate", "Due Date" }
                                }
                            }

                            if matches!(current_draft.schedule.schedule_type, ScheduleType::Routinely) {
                                div {
                                    label {
                                        class: "label p-0 mb-2",
                                        r#for: "repetition-interval-input",
                                        span { class: "label-text font-medium", "Repeat Every" }
                                    }
                                    div { class: "join w-full",
                                        input {
                                            r#type: "number",
                                            id: "repetition-interval-input",
                                            class: "input input-bordered join-item w-24 px-4",
                                            value: current_draft.repeat_config.as_ref().map(|r| r.interval).unwrap_or(1),
                                            oninput: move |e| {
                                                if let Ok(val) = e.value().parse::<i64>() {
                                                    draft
                                                        .with_mut(|d_opt| {
                                                            if let Some(d) = d_opt.as_mut() {
                                                                let mut config = d
                                                                    .repeat_config
                                                                    .clone()
                                                                    .unwrap_or(RepeatConfig {
                                                                        frequency: Frequency::Daily,
                                                                        interval: 1,
                                                                    });
                                                                config.interval = val;
                                                                d.repeat_config = Some(config);
                                                            }
                                                        });
                                                }
                                            },
                                        }
                                        select {
                                            id: "repetition-frequency-select",
                                            class: "select select-bordered join-item flex-grow",
                                            value: current_draft
                                                .repeat_config
                                                .as_ref()
                                                .map(|r| match r.frequency {
                                                    Frequency::Minutes => "Minutes",
                                                    Frequency::Hours => "Hours",
                                                    Frequency::Weekly => "Weekly",
                                                    Frequency::Monthly => "Monthly",
                                                    Frequency::Yearly => "Yearly",
                                                    _ => "Daily",
                                                })
                                                .unwrap_or("Daily"),
                                            onchange: move |e| {
                                                draft
                                                    .with_mut(|d_opt| {
                                                        if let Some(d) = d_opt.as_mut() {
                                                            let mut config = d
                                                                .repeat_config
                                                                .clone()
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
                                                        }
                                                    });
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

                            if matches!(
                                current_draft.schedule.schedule_type,
                                ScheduleType::DueDate | ScheduleType::Calendar
                            )
                            {
                                div {
                                    label {
                                        class: "label p-0 mb-2",
                                        r#for: "date-input",
                                        span { class: "label-text font-medium", "Due Date" }
                                    }
                                    DatePicker {
                                        id: "date-input",
                                        data_testid: "date-input",
                                        value: current_draft
                                            .schedule
                                            .due_date
                                            .map(|ts| {
                                                use chrono::TimeZone;
                                                let secs = ts / 1000;
                                                chrono::Utc
                                                    .timestamp_opt(secs, 0)
                                                    .single()
                                                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                                                    .unwrap_or_default()
                                            }),
                                        onchange: move |v: String| {
                                            if let Ok(date) = chrono::NaiveDate::parse_from_str(&v, "%Y-%m-%d")
                                                && let Some(dt) = date.and_hms_opt(0, 0, 0)
                                            {
                                                let ts = dt.and_utc().timestamp_millis();
                                                draft
                                                    .with_mut(|d_opt| {
                                                        if let Some(d) = d_opt.as_mut() {
                                                            d.schedule.due_date = Some(ts);
                                                        }
                                                    });
                                            }
                                        },
                                    }
                                }
                            }

                            if !matches!(current_draft.schedule.schedule_type, ScheduleType::Once) {
                                div {
                                    label {
                                        class: "label p-0 mb-2",
                                        r#for: "lead-time-scalar-input",
                                        span { class: "label-text font-medium", "Lead Time" }
                                    }
                                    div { class: "join w-full",
                                        {
                                            let (val, unit) = time_conversion::ms_to_period(
                                                current_draft.schedule.lead_time,
                                            );
                                            let unit_clone = unit.clone();
                                            rsx! {
                                                input {
                                                    r#type: "number",
                                                    id: "lead-time-scalar-input",
                                                    class: "input input-bordered join-item w-24 px-4",
                                                    value: "{val}",
                                                    oninput: move |e| {
                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                            draft
                                                                .with_mut(|d_opt| {
                                                                    if let Some(d) = d_opt.as_mut() {
                                                                        d.schedule.lead_time = time_conversion::period_to_ms(
                                                                            v,
                                                                            &unit_clone,
                                                                        );
                                                                    }
                                                                });
                                                        }
                                                    },
                                                }
                                                select {
                                                    id: "lead-time-unit-select",
                                                    aria_label: "Lead Time Unit",
                                                    class: "select select-bordered join-item flex-grow",
                                                    value: "{unit}",
                                                    onchange: move |e| {
                                                        draft
                                                            .with_mut(|d_opt| {
                                                                if let Some(d) = d_opt.as_mut() {
                                                                    let (v, _) = time_conversion::ms_to_period(d.schedule.lead_time);
                                                                    d.schedule.lead_time = time_conversion::period_to_ms(v, &e.value());
                                                                }
                                                            });
                                                    },
                                                    option { value: "Hours", "Hours" }
                                                    option { value: "Days", "Days" }
                                                    option { value: "Weeks", "Weeks" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Section: Additional Options
                    if task_id.is_some() {
                        fieldset { class: "fieldset bg-base-200/40 p-5 rounded-box",
                            label {
                                class: "flex items-center justify-between w-full cursor-pointer",
                                r#for: "sequential-toggle",
                                div { class: "flex flex-col gap-0.5",
                                    span { class: "text-sm font-bold", "Sequential Project" }
                                    span { class: "text-xs opacity-60 font-medium",
                                        "Steps must be completed in order"
                                    }
                                }
                                input {
                                    r#type: "checkbox",
                                    id: "sequential-toggle",
                                    class: "toggle toggle-primary toggle-sm",
                                    checked: current_draft.is_sequential,
                                    onchange: move |e| {
                                        draft
                                            .with_mut(|d_opt| {
                                                if let Some(d) = d_opt.as_mut() {
                                                    d.is_sequential = e.checked();
                                                }
                                            });
                                    },
                                }
                            }
                        }
                    }

                    // Hierarchy Controls (Compact)
                    if let Some(id) = task_id.clone() {
                        div { class: "flex items-center justify-center pt-2",
                            div { class: "join border border-base-300 bg-base-100 shadow-sm",
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    class: "join-item btn-xs h-9 px-5",
                                    onclick: move |_| show_move_picker.set(true),
                                    "Move"
                                }
                                if can_outdent {
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        class: "join-item btn-xs h-9 px-5 border-l border-base-300",
                                        onclick: {
                                            let id = id.clone();
                                            move |_| {
                                                task_controller.outdent(id.clone());
                                                on_close.call(());
                                            }
                                        },
                                        "Outdent"
                                    }
                                }
                                if can_indent {
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        class: "join-item btn-xs h-9 px-5 border-l border-base-300",
                                        onclick: {
                                            let id = id.clone();
                                            move |_| {
                                                task_controller.indent(id.clone());
                                                on_close.call(());
                                            }
                                        },
                                        "Indent"
                                    }
                                }
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    class: "join-item btn-xs h-9 px-5 border-l border-base-300",
                                    onclick: {
                                        let id = id.clone();
                                        move |_| {
                                            if let Some(h) = on_add_child.as_ref() {
                                                h.call(id.clone());
                                            }
                                        }
                                    },
                                    "Add Child"
                                }
                            }
                        }
                    }
                }

                // Modal Footer (Fixed at bottom)
                div { class: "px-6 py-4 bg-base-200/50 border-t border-base-200 sticky bottom-0 z-20 flex flex-col sm:flex-row justify-between items-center gap-4",
                    div { class: "flex-none w-full sm:w-auto",
                        if task_id.is_some() {
                            Button {
                                variant: ButtonVariant::Ghost,
                                class: "btn-sm text-error hover:bg-error/10 hover:text-error w-full sm:w-auto",
                                onclick: on_delete,
                                "Delete Task"
                            }
                        }
                    }
                    div { class: "flex flex-col-reverse sm:flex-row items-center gap-3 w-full sm:w-auto",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "btn-sm px-6 w-full sm:w-auto",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            class: "btn-sm min-w-[140px] shadow-lg shadow-primary/20 w-full sm:w-auto",
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

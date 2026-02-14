use crate::app_components::DateInput;
use crate::app_components::Loading;
use crate::app_components::MovePicker;
use crate::components::dialog::{DialogContent, DialogRoot, DialogTitle};
use crate::dioxus_components::button::{Button, ButtonVariant};
use crate::dioxus_components::input::Input;
use crate::dioxus_components::label::Label;
use crate::dioxus_components::select::{
    Select, SelectList, SelectOption, SelectTrigger, SelectValue,
};
use crate::dioxus_components::slider::{
    Slider, SliderRange, SliderThumb, SliderTrack, SliderValue,
};
use crate::dioxus_components::switch::{Switch, SwitchThumb};
use crate::dioxus_components::textarea::Textarea;
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
pub(crate) fn TaskEditor(
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
            DialogContent { class: "task-editor-dialog",
                // Header
                div { class: "dialog-header",
                    div { class: "dialog-title-group",
                        DialogTitle {
                            if task_id.is_some() {
                                "Edit Task"
                            } else {
                                "Create Task"
                            }
                        }
                        if let Some(ref title) = parent_title {
                            span { class: "dialog-subtitle", "Subtask of {title}" }
                        }
                    }

                    if let Some(id) = task_id.clone() {
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "app_ghost_hover app_transition",
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

                // Main Content (Scrollable if needed)
                div { class: "dialog-body",
                    // Section: Core Details
                    fieldset { class: "section-fieldset",
                        legend { class: "section-legend",
                            "Task Details"
                        }
                        div { class: "app_input-full app_stack_4",
                            div {
                                Label {
                                    class: "field-label",
                                    html_for: "task-title-input",
                                    span { class: "label-text-bold", "Title" }
                                }
                                Input {
                                    id: "task-title-input",
                                    value: current_draft.title.clone(),
                                    "autofocus": true,
                                    class: "app_input-full app_text_lg",
                                    oninput: move |evt: FormEvent| {
                                        draft
                                            .with_mut(|task| {
                                                if let Some(task) = task.as_mut() {
                                                    task.title = evt.value();
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
                                Label {
                                    class: "field-label",
                                    html_for: "notes-input",
                                    span { class: "label-text-bold", "Notes" }
                                }
                                Textarea {
                                    id: "notes-input",
                                    class: "app_input-full notes_area",
                                    placeholder: "Add more context or details here...",
                                    value: current_draft.notes.clone(),
                                    oninput: move |e: FormEvent| {
                                        draft
                                            .with_mut(|task| {
                                                if let Some(task) = task.as_mut() {
                                                    task.notes = e.value();
                                                }
                                            });
                                    },
                                }
                            }
                        }
                    }

                    // Section: Prioritization & Location (Grid)
                    div { class: "settings-cells",
                        fieldset { class: "settings-group",
                            legend { class: "section-legend",
                                "Prioritization"
                            }
                            div { class: "app_stack_6 app_input-full",
                                div {
                                    Label {
                                        class: "field-label",
                                        html_for: "importance-input",
                                        span { class: "label-text-medium",
                                            "Importance: "
                                            span { class: "label-text-alt",
                                                "{current_draft.importance:.2}"
                                            }
                                        }
                                    }
                                    Slider {
                                        id: "importance-input",
                                        min: 0.0,
                                        max: 1.0,
                                        step: 0.1,
                                        value: Some(SliderValue::Single(current_draft.importance)),
                                        on_value_change: move |val| {
                                            match val {
                                                SliderValue::Single(val) => {
                                                    draft
                                                        .with_mut(|task| {
                                                            if let Some(task) = task.as_mut() {
                                                                task.importance = val;
                                                            }
                                                        });
                                                }
                                                #[allow(unreachable_patterns)]
                                                _ => tracing::warn!("Unexpected SliderValue: {:?}", val),
                                            }
                                        },
                                        SliderTrack { SliderRange {} SliderThumb {} }
                                    }
                                }

                                div {
                                    Label {
                                        class: "field-label",
                                        html_for: "effort-input",
                                        span { class: "label-text-medium",
                                            "Effort: "
                                            span { class: "label-text-alt",
                                                "{current_draft.credit_increment.unwrap_or(0.5):.2}"
                                            }
                                        }
                                    }
                                    Slider {
                                        id: "effort-input",
                                        min: 0.0,
                                        max: 1.0,
                                        step: 0.1,
                                        value: Some(SliderValue::Single(current_draft.credit_increment.unwrap_or(0.5))),
                                        on_value_change: move |val| {
                                            match val {
                                                SliderValue::Single(val) => {
                                                    draft
                                                        .with_mut(|task| {
                                                            if let Some(task) = task.as_mut() {
                                                                task.credit_increment = Some(val);
                                                            }
                                                        });
                                                }
                                                #[allow(unreachable_patterns)]
                                                _ => tracing::warn!("Unexpected SliderValue: {:?}", val),
                                            }
                                        },
                                        SliderTrack { SliderRange {} SliderThumb {} }
                                    }
                                }
                            }
                        }

                        fieldset { class: "settings-group",
                            legend { class: "section-legend",
                                "Location"
                            }
                            div { class: "app_input-full",
                                Label {
                                    class: "field-label",
                                    html_for: "place-select",
                                    span { class: "label-text-medium", "Place" }
                                }
                                {
                                    let state_val = state();
                                    let places = state_val.places.values().collect::<Vec<_>>();

                                    rsx! {
                                        Select {
                                            value: current_draft.place_id.clone().map(|id| Some(id.to_string())),
                                            on_value_change: move |v: Option<String>| {
                                                draft
                                                    .with_mut(|task| {
                                                        if let Some(task) = task.as_mut() {
                                                            task.place_id = v.filter(|s| !s.is_empty()).map(PlaceID::from);
                                                        }
                                                    });
                                            },
                                            SelectTrigger {
                                                id: "place-select",
                                                SelectValue { aria_placeholder: "Anywhere" }
                                            }
                                            SelectList {
                                                SelectOption::<String> { value: "".to_string(), index: 0_usize, "Anywhere" }
                                                for (i, place) in places.iter().enumerate() {
                                                    SelectOption::<String> {
                                                        value: place.id.to_string(),
                                                        index: i + 1_usize,
                                                        "{place.name}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Section: Scheduling
                    fieldset { class: "settings-group",
                        legend { class: "section-legend",
                            "Scheduling"
                        }
                        div { class: "settings-cells app_input-full",
                            div {
                                Label {
                                    class: "field-label",
                                    html_for: "schedule-type-select",
                                    span { class: "label-text-medium", "Schedule Type" }
                                }
                                Select {
                                    value: Some(match current_draft.schedule.schedule_type {
                                        ScheduleType::Once => "Once".to_string(),
                                        ScheduleType::Routinely => "Routinely".to_string(),
                                        ScheduleType::DueDate | ScheduleType::Calendar => "DueDate".to_string(),
                                    }),
                                    on_value_change: move |v: Option<String>| {
                                        let new_type = match v.as_deref() {
                                            Some("Once") => ScheduleType::Once,
                                            Some("Routinely") => ScheduleType::Routinely,
                                            Some("DueDate") => ScheduleType::DueDate,
                                            _ => ScheduleType::Once,
                                        };
                                        draft
                                            .with_mut(|task| {
                                                if let Some(task) = task.as_mut() {
                                                    task.schedule.schedule_type = new_type;
                                                    if matches!(new_type, ScheduleType::Routinely) {
                                                        if task.repeat_config.is_none() {
                                                            task.repeat_config = Some(RepeatConfig {
                                                                frequency: Frequency::Daily,
                                                                interval: 1,
                                                            });
                                                        }
                                                    } else {
                                                        task.repeat_config = None;
                                                    }
                                                }
                                            });
                                    },
                                    SelectTrigger {
                                        id: "schedule-type-select",
                                        SelectValue {}
                                    }
                                    SelectList {
                                        SelectOption::<String> { value: "Once".to_string(), index: 0_usize, "Once" }
                                        SelectOption::<String> { value: "Routinely".to_string(), index: 1_usize, "Routinely" }
                                        SelectOption::<String> { value: "DueDate".to_string(), index: 2_usize, "Due Date" }
                                    }
                                }
                            }

                            if matches!(current_draft.schedule.schedule_type, ScheduleType::Routinely) {
                                div {
                                    Label {
                                        class: "field-label",
                                        html_for: "repetition-interval-input",
                                        span { class: "label-text-medium", "Repeat Every" }
                                    }
                                    div { class: "app_row-cluster",
                                        input {
                                            r#type: "number",
                                            id: "repetition-interval-input",
                                            class: "input-number-small bg-app-surface",
                                            value: current_draft.repeat_config.as_ref().map(|r| r.interval).unwrap_or(1),
                                            oninput: move |e| {
                                                if let Ok(val) = e.value().parse::<i64>() {
                                                    draft
                                                        .with_mut(|task| {
                                                            if let Some(task) = task.as_mut() {
                                                                let mut config = task
                                                                    .repeat_config
                                                                    .clone()
                                                                    .unwrap_or(RepeatConfig {
                                                                        frequency: Frequency::Daily,
                                                                        interval: 1,
                                                                    });
                                                                config.interval = val;
                                                                task.repeat_config = Some(config);
                                                            }
                                                        });
                                                }
                                            },
                                        }
                                        Select {
                                            value: Some(current_draft
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
                                                .unwrap_or("Daily").to_string()),
                                            on_value_change: move |v: Option<String>| {
                                                draft
                                                    .with_mut(|task| {
                                                        if let Some(task) = task.as_mut() {
                                                            let mut config = task
                                                                .repeat_config
                                                                .clone()
                                                                .unwrap_or(RepeatConfig {
                                                                    frequency: Frequency::Daily,
                                                                    interval: 1,
                                                                });
                                                            config.frequency = match v.as_deref() {
                                                                Some("Minutes") => Frequency::Minutes,
                                                                Some("Hours") => Frequency::Hours,
                                                                Some("Weekly") => Frequency::Weekly,
                                                                Some("Monthly") => Frequency::Monthly,
                                                                Some("Yearly") => Frequency::Yearly,
                                                                _ => Frequency::Daily,
                                                            };
                                                            task.repeat_config = Some(config);
                                                        }
                                                    });
                                            },
                                            SelectTrigger {
                                                id: "repetition-frequency-select",
                                                SelectValue {}
                                            }
                                            SelectList {
                                                SelectOption::<String> { value: "Minutes".to_string(), index: 0_usize, "Minutes" }
                                                SelectOption::<String> { value: "Hours".to_string(), index: 1_usize, "Hours" }
                                                SelectOption::<String> { value: "Daily".to_string(), index: 2_usize, "Daily" }
                                                SelectOption::<String> { value: "Weekly".to_string(), index: 3_usize, "Weekly" }
                                                SelectOption::<String> { value: "Monthly".to_string(), index: 4_usize, "Monthly" }
                                                SelectOption::<String> { value: "Yearly".to_string(), index: 5_usize, "Yearly" }
                                            }
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
                                    Label {
                                        class: "field-label",
                                        html_for: "date-input",
                                        span { class: "label-text-medium", "Due Date" }
                                    }
                                    DateInput {
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
                                            draft.with_mut(|task| {
                                                if let Some(task) = task.as_mut() {
                                                    if v.is_empty() {
                                                        task.schedule.due_date = None;
                                                    } else if let Ok(date) =
                                                        chrono::NaiveDate::parse_from_str(&v, "%Y-%m-%d")
                                                        && let Some(dt) = date.and_hms_opt(0, 0, 0)
                                                    {
                                                        let ts = dt.and_utc().timestamp_millis();
                                                        task.schedule.due_date = Some(ts);
                                                    } else {
                                                        task.schedule.due_date = None;
                                                    }
                                                }
                                            });
                                        },
                                    }
                                }
                            }

                            if !matches!(current_draft.schedule.schedule_type, ScheduleType::Once) {
                                div {
                                    Label {
                                        class: "field-label",
                                        html_for: "lead-time-scalar-input",
                                        span { class: "label-text-medium", "Lead Time" }
                                    }
                                    div { class: "app_row-cluster",
                                        {
                                            let (val, unit) = time_conversion::ms_to_period(
                                                current_draft.schedule.lead_time,
                                            );
                                            let unit_clone = unit.clone();
                                            rsx! {
                                                input {
                                                    r#type: "number",
                                                    id: "lead-time-scalar-input",
                                                    class: "input-number-small bg-app-surface",
                                                    value: "{val}",
                                                    oninput: move |e| {
                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                            draft
                                                                .with_mut(|task| {
                                                                    if let Some(task) = task.as_mut() {
                                                                        task.schedule.lead_time = time_conversion::period_to_ms(
                                                                            v,
                                                                            &unit_clone,
                                                                        );
                                                                    }
                                                                });
                                                        }
                                                    },
                                                }
                                                Select {
                                                    value: Some(unit.to_string()),
                                                    on_value_change: move |v: Option<String>| {
                                                        if let Some(u) = v {
                                                            draft
                                                                .with_mut(|task| {
                                                                    if let Some(task) = task.as_mut() {
                                                                        let (v, _) = time_conversion::ms_to_period(task.schedule.lead_time);
                                                                        task.schedule.lead_time = time_conversion::period_to_ms(v, &u);
                                                                    }
                                                                });
                                                        }
                                                    },
                                                    SelectTrigger {
                                                        id: "lead-time-unit-select",
                                                        aria_label: "Lead Time Unit",
                                                        SelectValue {}
                                                    }
                                                    SelectList {
                                                        SelectOption::<String> { value: "Hours".to_string(), index: 0_usize, "Hours" }
                                                        SelectOption::<String> { value: "Days".to_string(), index: 1_usize, "Days" }
                                                        SelectOption::<String> { value: "Weeks".to_string(), index: 2_usize, "Weeks" }
                                                    }
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
                        fieldset { class: "settings-group",
                            Label {
                                class: "field-label app_cursor_pointer",
                                html_for: "sequential-toggle",
                                div { class: "sequential-label-group",
                                    span { class: "label-text-bold app_text_sm", "Sequential Project" }
                                    span { class: "sequential-label-description",
                                        "Steps must be completed in order"
                                    }
                                }
                                Switch {
                                    id: "sequential-toggle",
                                    checked: current_draft.is_sequential,
                                    on_checked_change: move |checked| {
                                        draft
                                            .with_mut(|task| {
                                                if let Some(task) = task.as_mut() {
                                                    task.is_sequential = checked;
                                                }
                                            });
                                    },
                                    SwitchThumb {}
                                }
                            }
                        }
                    }

                    // Hierarchy Controls (Compact)
                    if let Some(id) = task_id.clone() {
                        div { class: "app_center_pt",
                            div { class: "hierarchy-controls-group",
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    class: "hierarchy-button",
                                    onclick: move |_| show_move_picker.set(true),
                                    "Move"
                                }
                                if can_outdent {
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        class: "hierarchy-button hierarchy-button-sep",
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
                                        class: "hierarchy-button hierarchy-button-sep",
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
                                    class: "hierarchy-button hierarchy-button-sep",
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

                // Footer
                div { class: "dialog-footer",
                    div { class: "footer_delete_box",
                        if task_id.is_some() {
                            Button {
                                variant: ButtonVariant::Ghost,
                                class: "btn-delete footer_button_full",
                                onclick: on_delete,
                                "Delete Task"
                            }
                        }
                    }
                    div { class: "dialog-footer-actions",
                        Button {
                            variant: ButtonVariant::Ghost,
                            class: "footer_button_full footer_cancel_btn",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            class: "btn-save footer_button_full",
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
                MovePicker {
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

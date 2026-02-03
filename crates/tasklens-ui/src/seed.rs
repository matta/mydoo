#![allow(dead_code)]
use tasklens_core::types::{PersistedTask, Schedule, ScheduleType, TaskID, TaskStatus};
use tasklens_store::store::AppStore;

pub fn prime_store_with_sample_data(store: &mut AppStore) {
    let mut state: tasklens_core::types::TunnelState =
        store.store_hydrate_tunnel_state().unwrap_or_default();
    state.tasks.clear();
    state.root_task_ids.clear();

    let project_alpha_id = TaskID::new();
    let mut project_alpha = create_base_task(project_alpha_id.clone(), "Project Alpha", None);

    let research_id = TaskID::new();
    let research_task = create_base_task(
        research_id.clone(),
        "Research Requirements",
        Some(project_alpha_id.clone()),
    );

    let design_id = TaskID::new();
    let design_task = create_base_task(
        design_id.clone(),
        "Design UI Mocks",
        Some(project_alpha_id.clone()),
    );

    project_alpha.child_task_ids.push(research_id.clone());
    project_alpha.child_task_ids.push(design_id.clone());

    let groceries_id = TaskID::new();
    let groceries_task = create_base_task(groceries_id.clone(), "Buy Groceries", None);

    state.tasks.insert(project_alpha_id.clone(), project_alpha);
    state.tasks.insert(research_id.clone(), research_task);
    state.tasks.insert(design_id.clone(), design_task);
    state.tasks.insert(groceries_id.clone(), groceries_task);

    state.root_task_ids.push(project_alpha_id);
    state.root_task_ids.push(groceries_id);

    if let Err(e) = store.reconcile_tunnel_state(&state) {
        tracing::error!("Failed to seed data: {:?}", e);
    }
}

fn create_base_task(id: TaskID, title: &str, parent_id: Option<TaskID>) -> PersistedTask {
    PersistedTask {
        id,
        title: title.to_string(),
        notes: String::new(),
        parent_id,
        child_task_ids: Vec::new(),
        place_id: None,
        status: TaskStatus::Pending,
        importance: 0.5,
        credit_increment: None,
        credits: 0.0,
        desired_credits: 1.0,
        credits_timestamp: 0,
        priority_timestamp: 0,
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
    }
}

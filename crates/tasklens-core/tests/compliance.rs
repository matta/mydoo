use anyhow::{Context as _, Result, anyhow};
use automerge::AutoCommit;
use chrono::{DateTime, Utc};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tasklens_core::PlaceID;

use tasklens_core::Action;
use tasklens_core::TaskUpdates;
use tasklens_core::domain::doc_bridge;
use tasklens_core::domain::priority::get_prioritized_tasks;
use tasklens_core::types::{
    Context, Frequency, PriorityOptions, RepeatConfig, ScheduleType, TaskID, TaskStatus,
    TunnelState, UrgencyStatus, ViewFilter,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct Feature {
    feature: String,
    description: Option<String>,
    #[serde(default)]
    background: Option<InitialState>,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct Scenario {
    name: String,
    description: Option<String>,
    steps: Vec<Step>,
    examples: Option<Vec<HashMap<String, serde_json::Value>>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct Step {
    legacy_description: Option<String>,
    given: Option<InitialState>,
    when: Option<Mutation>,
    then: Option<Assertion>,
    #[serde(default)]
    view_filter: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(deny_unknown_fields)]
struct InitialState {
    current_time: Option<String>,
    timezone_offset: Option<serde_json::Value>,
    places: Option<Vec<PlaceInput>>,
    tasks: Option<Vec<TaskInput>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct PlaceInput {
    id: String,
    hours: Option<OpenHoursInput>,
    included_places: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct OpenHoursInput {
    mode: String,
    schedule: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct TaskInput {
    id: String,
    parent_id: Option<String>,
    children: Option<Vec<TaskInput>>,
    title: Option<String>,
    importance: Option<F64OrString>,
    status: Option<TaskStatus>,
    credits: Option<F64OrString>,
    #[serde(alias = "credits_increment")]
    credit_increment: Option<F64OrString>,
    credits_timestamp: Option<String>,
    desired_credits: Option<F64OrString>,
    due_date: Option<serde_json::Value>,
    place_id: Option<String>,
    lead_time_seconds: Option<F64OrString>,
    is_sequential: Option<BoolOrString>,
    schedule_type: Option<ScheduleType>,
    period_seconds: Option<F64OrString>,
    last_done: Option<serde_json::Value>,
    repeat_config: Option<RepeatConfigInput>,
    is_acknowledged: Option<BoolOrString>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct RepeatConfigInput {
    frequency: Frequency,
    interval: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum F64OrString {
    Float(f64),
    String(String),
}

impl F64OrString {
    fn to_f64(&self) -> f64 {
        match self {
            F64OrString::Float(f) => *f,
            F64OrString::String(s) => s.parse().unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum BoolOrString {
    Bool(bool),
    String(String),
}

impl BoolOrString {
    fn to_bool(&self) -> bool {
        match self {
            BoolOrString::Bool(b) => *b,
            BoolOrString::String(s) => s.parse().unwrap_or(false),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct Mutation {
    advance_time_seconds: Option<F64OrString>,
    update_credits: Option<HashMap<String, F64OrString>>,
    task_updates: Option<Vec<TaskUpdate>>,
    create_tasks: Option<Vec<TaskInput>>,
    delete_tasks: Option<Vec<String>>,
    complete_tasks: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct TaskUpdate {
    id: String,
    status: Option<TaskStatus>,
    credits: Option<F64OrString>,
    #[serde(alias = "credits_increment")]
    credit_increment: Option<F64OrString>,
    desired_credits: Option<F64OrString>,
    importance: Option<F64OrString>,
    due_date: Option<serde_json::Value>,
    place_id: Option<String>,
    is_acknowledged: Option<BoolOrString>,
    schedule_type: Option<ScheduleType>,
    repeat_config: Option<RepeatConfigInput>,
    last_done: Option<serde_json::Value>,
    lead_time_seconds: Option<F64OrString>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct Assertion {
    expected_order: Option<serde_json::Value>,
    expected_props: Option<Vec<ExpectedTaskProps>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct ExpectedTaskProps {
    id: String,
    score: Option<F64OrString>,
    credits: Option<F64OrString>,
    effective_credits: Option<F64OrString>,
    effective_due_date: Option<serde_json::Value>,
    effective_lead_time: Option<F64OrString>,
    due_date: Option<serde_json::Value>,
    urgency_status: Option<UrgencyStatus>,
    importance: Option<F64OrString>,
    normalized_importance: Option<F64OrString>,
    is_blocked: Option<BoolOrString>,
    is_visible: Option<BoolOrString>,
    is_ready: Option<BoolOrString>,
    is_open: Option<BoolOrString>,
    place_id: Option<String>,
    #[serde(alias = "credits_increment")]
    credit_increment: Option<F64OrString>,
}

/// A shim to support compliance tests with in-memory Automerge documents.
/// This mirrors the pattern from `tasklens_store::store::tests::AppStore`.
struct ComplianceStore {
    doc: AutoCommit,
}

impl ComplianceStore {
    fn new() -> Self {
        Self {
            doc: AutoCommit::new(),
        }
    }

    fn init(&mut self) -> Result<()> {
        let initial_state = TunnelState {
            tasks: HashMap::new(),
            places: HashMap::new(),
            root_task_ids: Vec::new(),
            metadata: None,
        };
        doc_bridge::reconcile_tunnel_state(&mut self.doc, &initial_state)
            .map_err(|e| anyhow!("Init failed: {}", e))
    }

    fn dispatch(&mut self, action: Action) -> Result<()> {
        tasklens_core::run_action(&mut self.doc, action).map_err(|e| anyhow!(e))
    }

    fn hydrate(&self) -> Result<TunnelState> {
        doc_bridge::hydrate_tunnel_state(&self.doc).map_err(|e| anyhow!("Hydration failed: {}", e))
    }
}

#[test]
fn test_compliance() -> Result<()> {
    let mut files_found = Vec::new();

    let yaml_pattern = "tests/compliance/fixtures/*.yaml";
    let yaml_files: Vec<_> = glob(yaml_pattern)?.collect();
    if !yaml_files.is_empty() {
        panic!(
            "Found .yaml files in compliance fixtures. Only .json files are allowed to avoid duplication: {:?}",
            yaml_files
        );
    }

    let json_pattern = "tests/compliance/fixtures/*.json";
    for entry in glob(json_pattern)? {
        let path = entry?;
        files_found.push(path.file_name().unwrap().to_str().unwrap().to_string());
    }

    let mut expected_files = vec![
        "balancing.feature.json",
        "boost-importance.feature.json",
        "boost-lead-time.feature.json",
        "completion-acknowledgement.feature.json",
        "complex-mutation.feature.json",
        "container-visibility.feature.json",
        "credit-attribution-aggregation.feature.json",
        "credit-attribution.feature.json",
        "decay.feature.json",
        "due_dates.feature.json",
        "deletion.feature.json",
        "inheritance-credits.feature.json",
        "inheritance-importance.feature.json",
        "inheritance-place.feature.json",
        "inheritance-schedule.feature.json",
        "lead-time-edge-cases.feature.json",
        "lead-time-inheritance.feature.json",
        "lead-time.feature.json",
        "min-threshold.feature.json",
        "repro-neutral-inheritance.feature.json",
        "repro-stale-leadtime.feature.json",
        "root-importance.feature.json",
        "routinely-parent-visibility.feature.json",
        "sequential-flow.feature.json",
        "sorting.feature.json",
        "sorting-importance-tiebreaker.feature.json",
        "thermostat.feature.json",
        "tree-order-id-conflict.feature.json",
        "tree-order.feature.json",
        "task-update-fields.feature.json",
        "visibility-place-filtering.feature.json",
        "weight.feature.json",
        "zero-feedback.feature.json",
    ];

    files_found.sort();
    expected_files.sort();

    assert_eq!(
        files_found, expected_files,
        "Mismatch in fixture files found. This check ensures no fixtures are accidentally deleted and that the glob works correctly."
    );

    for file in &files_found {
        let mut path = PathBuf::from("tests/compliance/fixtures/");
        path.push(file);
        println!("Running compliance test: {:?}", path);
        run_feature_test(path)?;
    }

    println!("Successfully ran {} compliance features", files_found.len());
    Ok(())
}

fn run_feature_test(path: PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(&path)?;
    let feature: Feature = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse fixture: {:?}", path))?;

    // Destructure Feature to ensure exhaustiveness (except for description/feature name)
    let Feature {
        feature: _,
        description: _,
        background,
        scenarios,
    } = feature;

    for scenario in scenarios {
        let expanded = expand_scenarios(&scenario)?;
        for s in expanded {
            println!("  Scenario: {}", s.name);
            run_scenario(background.as_ref(), &s)?;
        }
    }

    Ok(())
}

fn expand_scenarios(scenario: &Scenario) -> Result<Vec<Scenario>> {
    let mut expanded = Vec::new();
    if let Some(examples) = &scenario.examples {
        for (i, example) in examples.iter().enumerate() {
            let json_str = serde_json::to_string(scenario)?;
            let mut expanded_str = json_str;
            for (key, val) in example {
                let placeholder = format!("${{{}}}", key);
                let val_str = match val {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => "".to_string(),
                };
                expanded_str = expanded_str.replace(&placeholder, &val_str);
            }
            let mut new_scenario: Scenario = serde_json::from_str(&expanded_str)?;
            new_scenario.name = format!("{} (Example {})", scenario.name, i + 1);
            expanded.push(new_scenario);
        }
    } else {
        expanded.push(scenario.clone());
    }
    Ok(expanded)
}

fn parse_date(s: &str) -> Result<i64> {
    let iso = if s.len() == 10 {
        format!("{}T00:00:00Z", s)
    } else if !s.ends_with('Z') && !s.contains('+') {
        format!("{}Z", s)
    } else {
        s.to_string()
    };

    let dt = DateTime::parse_from_rfc3339(&iso)
        .or_else(|_| {
            DateTime::parse_from_str(&iso, "%Y-%m-%dT%H:%M:%S%.fZ")
                .map(|dt| dt.with_timezone(&chrono::offset::FixedOffset::east_opt(0).unwrap()))
        })
        .map_err(|e| anyhow!("Failed to parse date: {} - {}", s, e))?;

    Ok(dt.with_timezone(&Utc).timestamp_millis())
}

fn parse_json_date(v: &serde_json::Value) -> Result<Option<i64>> {
    match v {
        serde_json::Value::String(s) => match parse_date(s) {
            Ok(ms) => Ok(Some(ms)),
            Err(e) => {
                println!("      Warning: Failed to parse date string '{}': {}", s, e);
                Ok(None)
            }
        },
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Some(i))
            } else if let Some(u) = n.as_u64() {
                Ok(Some(u as i64))
            } else if let Some(f) = n.as_f64() {
                Ok(Some(f as i64))
            } else {
                Ok(None)
            }
        }
        _ => {
            println!(
                "      Warning: Unexpected JSON value type for date: {:?}",
                v
            );
            Ok(None)
        }
    }
}

fn assert_f64_near(actual: f64, expected: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff < 0.001,
        "{}: actual={}, expected={}",
        label,
        actual,
        expected
    );
}

fn run_scenario(background: Option<&InitialState>, scenario: &Scenario) -> Result<()> {
    let mut store = ComplianceStore::new();
    store.init()?;

    let mut current_time = parse_date("2025-01-01T12:00:00Z")?;

    if let Some(bg) = background {
        apply_initial_state(&mut store, &mut current_time, bg)?;
    }

    let Scenario { steps, .. } = scenario;

    for step in steps {
        let Step {
            legacy_description,
            given,
            when,
            then,
            view_filter,
        } = step;

        if let Some(given_state) = given {
            apply_initial_state(&mut store, &mut current_time, given_state)?;
        }

        if let Some(mutation) = when {
            apply_mutation(&mut store, &mut current_time, mutation)?;
        }

        if let Some(assertion) = then {
            let state = store.hydrate()?;
            let filter_str = view_filter.as_deref().unwrap_or("All Places");
            let place_id_filter = if filter_str == "All Places" {
                Some("All".to_string())
            } else {
                Some(filter_str.to_string())
            };

            let options_filtered = PriorityOptions {
                include_hidden: false,
                mode: None,
                context: Some(Context {
                    current_place_id: None,
                    current_time,
                }),
            };
            let options_all = PriorityOptions {
                include_hidden: true,
                mode: None,
                context: Some(Context {
                    current_place_id: None,
                    current_time,
                }),
            };
            let view_filter_obj = ViewFilter {
                place_id: place_id_filter,
            };

            let results_filtered =
                get_prioritized_tasks(&state, &view_filter_obj, &options_filtered);
            let results_all = get_prioritized_tasks(&state, &view_filter_obj, &options_all);

            let Assertion {
                expected_order,
                expected_props,
            } = assertion;

            if let Some(order) = expected_order {
                let actual_order: Vec<String> = results_filtered
                    .iter()
                    .map(|t| t.id.as_str().to_string())
                    .collect();
                let expected_ids: Vec<String> = match order {
                    serde_json::Value::Array(seq) => seq
                        .iter()
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .collect(),
                    serde_json::Value::String(s) => {
                        if s.is_empty() {
                            vec![]
                        } else {
                            vec![s.clone()]
                        }
                    }
                    _ => Vec::new(),
                };
                assert_eq!(
                    actual_order, expected_ids,
                    "Mismatch in expected order in scenario '{}' at step '{:?}'",
                    scenario.name, legacy_description
                );
            }

            if let Some(props) = expected_props {
                for expected in props {
                    let actual = results_all
                        .iter()
                        .find(|t| t.id.as_str() == expected.id)
                        .ok_or_else(|| {
                            anyhow!(
                                "Task {} not found in results in scenario '{}' at step '{:?}'",
                                expected.id,
                                scenario.name,
                                legacy_description
                            )
                        })?;

                    let ExpectedTaskProps {
                        id: _,
                        score,
                        credits,
                        effective_credits,
                        effective_due_date,
                        effective_lead_time,
                        due_date,
                        urgency_status,
                        importance,
                        normalized_importance,
                        is_blocked,
                        is_visible,
                        is_ready,
                        is_open,
                        place_id,
                        credit_increment,
                    } = expected;

                    if let Some(status) = urgency_status {
                        assert_eq!(
                            actual.urgency_status, *status,
                            "Task: {}, Scenario: {}",
                            expected.id, scenario.name
                        );
                    }

                    if let Some(edd) = effective_due_date {
                        let expected_ms = parse_json_date(edd)?;
                        assert_eq!(
                            actual.effective_due_date, expected_ms,
                            "Task: {}, Scenario: {}",
                            expected.id, scenario.name
                        );
                    }

                    if let Some(dd) = due_date {
                        let expected_ms = parse_json_date(dd)?;
                        assert_eq!(
                            actual.schedule.due_date, expected_ms,
                            "Task: {}, Scenario: {}",
                            expected.id, scenario.name
                        );
                    }

                    if let Some(elt) = effective_lead_time {
                        assert_eq!(
                            actual.effective_lead_time,
                            Some(elt.to_f64() as i64),
                            "Task: {}, Scenario: {}",
                            expected.id,
                            scenario.name
                        );
                    }

                    if let Some(ready) = is_ready {
                        assert_eq!(
                            actual.is_ready,
                            ready.to_bool(),
                            "Task: {}, Scenario: {}",
                            expected.id,
                            scenario.name
                        );
                    }

                    if let Some(eff_credits) = effective_credits {
                        assert_f64_near(
                            actual.effective_credits,
                            eff_credits.to_f64(),
                            &format!(
                                "Task: {}, Scenario: {}, Effective Credits",
                                expected.id, scenario.name
                            ),
                        );
                    }

                    if let Some(c) = credits {
                        assert_f64_near(
                            actual.credits,
                            c.to_f64(),
                            &format!(
                                "Task: {}, Scenario: {}, Credits (stored)",
                                expected.id, scenario.name
                            ),
                        );
                    }

                    if let Some(ci) = credit_increment {
                        assert_f64_near(
                            actual.credit_increment.unwrap_or(0.0),
                            ci.to_f64(),
                            &format!(
                                "Task: {}, Scenario: {}, Credit Increment",
                                expected.id, scenario.name
                            ),
                        );
                    }

                    if let Some(imp) = importance {
                        assert_f64_near(
                            actual.importance,
                            imp.to_f64(),
                            &format!(
                                "Task: {}, Scenario: {}, Importance",
                                expected.id, scenario.name
                            ),
                        );
                    }

                    if let Some(s) = score {
                        assert_f64_near(
                            actual.score,
                            s.to_f64(),
                            &format!("Task: {}, Scenario: {}, Score", expected.id, scenario.name),
                        );
                    }

                    if let Some(ni) = normalized_importance {
                        assert_f64_near(
                            actual.normalized_importance,
                            ni.to_f64(),
                            &format!(
                                "Task: {}, Scenario: {}, Normalized Importance",
                                expected.id, scenario.name
                            ),
                        );
                    }

                    if let Some(visible) = is_visible {
                        assert_eq!(
                            actual.is_visible,
                            visible.to_bool(),
                            "Task: {}, Scenario: {}, Visibility",
                            expected.id,
                            scenario.name
                        );
                    }

                    if let Some(blocked) = is_blocked {
                        assert_eq!(
                            actual.is_blocked,
                            blocked.to_bool(),
                            "Task: {}, Scenario: {}, Blocked",
                            expected.id,
                            scenario.name
                        );
                    }

                    if let Some(open) = is_open {
                        assert_eq!(
                            actual.is_open,
                            open.to_bool(),
                            "Task: {}, Scenario: {}, Open",
                            expected.id,
                            scenario.name
                        );
                    }

                    if let Some(pid) = place_id {
                        let actual_place = actual
                            .place_id
                            .as_ref()
                            .map(|p| p.as_str())
                            .unwrap_or("Anywhere");
                        assert_eq!(
                            actual_place, pid,
                            "Task: {}, Scenario: {}, PlaceID",
                            expected.id, scenario.name
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn apply_initial_state(
    store: &mut ComplianceStore,
    current_time: &mut i64,
    init: &InitialState,
) -> Result<()> {
    let InitialState {
        current_time: ct,
        timezone_offset: _, // Ignored for now as domain handles everything in UTC
        places,
        tasks,
    } = init;

    if let Some(time_str) = ct {
        *current_time = parse_date(time_str)?;
    }

    if let Some(places_input) = places {
        for p in places_input {
            apply_place_input(store, p)?;
        }
    }

    if let Some(tasks_input) = tasks {
        for t in tasks_input {
            apply_task_input(store, t, None, *current_time)?;
        }
    }

    Ok(())
}

fn apply_place_input(store: &mut ComplianceStore, p: &PlaceInput) -> Result<()> {
    let place_id = PlaceID::from(p.id.clone());

    store.dispatch(Action::CreatePlace {
        id: place_id,
        name: p.id.clone(),
        hours: p
            .hours
            .as_ref()
            .map(|h| serde_json::to_string(h).unwrap())
            .unwrap_or_default(),
        included_places: p
            .included_places
            .as_ref()
            .map(|list| list.iter().map(|s| PlaceID::from(s.clone())).collect())
            .unwrap_or_default(),
    })?;
    Ok(())
}

fn apply_task_input(
    store: &mut ComplianceStore,
    t: &TaskInput,
    parent_id: Option<TaskID>,
    current_time: i64,
) -> Result<()> {
    let task_id = TaskID::from(t.id.clone());

    let effective_parent_id = t
        .parent_id
        .as_ref()
        .map(|s| TaskID::from(s.clone()))
        .or(parent_id);

    // 1. Dispatch CreateTask
    // Compliance tests assume creation at `current_time` is handled via defaults or explicit updates.
    store.dispatch(Action::CreateTask {
        id: task_id.clone(),
        parent_id: effective_parent_id,
        title: t.title.clone().unwrap_or_default(),
    })?;

    // 2. Build and Dispatch UpdateTask for all other fields
    let mut updates = TaskUpdates {
        // Compliance tests assume tasks are "born" at current_time for decay calculations
        credits_timestamp: Some(current_time),
        priority_timestamp: Some(current_time),
        ..Default::default()
    };

    if let Some(val) = t.status {
        updates.status = Some(val);
    }
    if let Some(val) = &t.importance {
        updates.importance = Some(val.to_f64());
    }
    if let Some(val) = &t.credits {
        updates.credits = Some(val.to_f64());
    }
    if let Some(val) = &t.credit_increment {
        updates.credit_increment = Some(val.to_f64());
    }
    if let Some(val) = &t.desired_credits {
        updates.desired_credits = Some(val.to_f64());
    }
    if let Some(ts) = &t.credits_timestamp {
        updates.credits_timestamp = Some(parse_date(ts)?);
    }

    if let Some(pid) = &t.place_id {
        updates.place_id = Some(if pid == "null" || pid.is_empty() {
            None
        } else {
            Some(PlaceID::from(pid.clone()))
        });
    }
    if let Some(is) = &t.is_sequential {
        updates.is_sequential = Some(is.to_bool());
    }
    if let Some(st) = t.schedule_type {
        updates.schedule_type = Some(st);
    }
    if let Some(ack) = &t.is_acknowledged {
        updates.is_acknowledged = Some(ack.to_bool());
    }
    if let Some(dd) = &t.due_date {
        updates.due_date = Some(parse_json_date(dd)?);
    }
    if let Some(lt) = &t.lead_time_seconds {
        updates.lead_time = Some((lt.to_f64() * 1000.0) as i64);
    }
    if let Some(ld) = &t.last_done {
        updates.last_done = Some(parse_json_date(ld)?);
    }
    if let Some(rc) = &t.repeat_config {
        updates.repeat_config = Some(Some(RepeatConfig {
            frequency: rc.frequency,
            interval: rc.interval as i64,
        }));
    }

    store.dispatch(Action::UpdateTask {
        id: task_id.clone(),
        updates,
    })?;

    // 3. Recurse children
    if let Some(children) = &t.children {
        for child_input in children {
            apply_task_input(store, child_input, Some(task_id.clone()), current_time)?;
        }
    }

    Ok(())
}

fn apply_mutation(
    store: &mut ComplianceStore,
    current_time: &mut i64,
    mutation: &Mutation,
) -> Result<()> {
    let Mutation {
        advance_time_seconds,
        update_credits,
        task_updates,
        create_tasks,
        delete_tasks,
        complete_tasks,
    } = mutation;

    if let Some(advance) = advance_time_seconds {
        *current_time += (advance.to_f64() * 1000.0) as i64;
    }

    if let Some(credits_map) = update_credits {
        for (id, val) in credits_map {
            let task_id = TaskID::from(id.clone());
            store.dispatch(Action::UpdateTask {
                id: task_id,
                updates: TaskUpdates {
                    credits: Some(val.to_f64()),
                    credits_timestamp: Some(*current_time),
                    ..Default::default()
                },
            })?;
        }
    }

    if let Some(tasks) = create_tasks {
        for t in tasks {
            apply_task_input(store, t, None, *current_time)?;
        }
    }

    if let Some(updates) = task_updates {
        for u in updates {
            let TaskUpdate {
                id,
                status,
                credits,
                credit_increment,
                desired_credits,
                importance,
                due_date,
                place_id,
                is_acknowledged,
                schedule_type,
                repeat_config,
                last_done,
                lead_time_seconds,
            } = u;

            let task_id = TaskID::from(id.clone());
            let mut action_updates = TaskUpdates::default();

            if let Some(val) = status {
                action_updates.status = Some(*val);
            }
            if let Some(val) = importance {
                action_updates.importance = Some(val.to_f64());
            }
            if let Some(val) = credits {
                action_updates.credits = Some(val.to_f64());
            }
            if let Some(val) = desired_credits {
                action_updates.desired_credits = Some(val.to_f64());
            }
            if let Some(dd) = due_date {
                action_updates.due_date = Some(parse_json_date(dd)?);
            }
            if let Some(ack) = is_acknowledged {
                action_updates.is_acknowledged = Some(ack.to_bool());
            }
            if let Some(ci) = credit_increment {
                action_updates.credit_increment = Some(ci.to_f64());
            }
            if let Some(pid) = place_id {
                action_updates.place_id = if pid == "null" || pid.is_empty() {
                    Some(None)
                } else {
                    Some(Some(PlaceID::from(pid.clone())))
                };
            }
            if let Some(st) = schedule_type {
                action_updates.schedule_type = Some(*st);
            }
            if let Some(rc) = repeat_config {
                action_updates.repeat_config = Some(Some(RepeatConfig {
                    frequency: rc.frequency,
                    interval: rc.interval as i64,
                }));
            }
            if let Some(ld) = last_done {
                action_updates.last_done = Some(parse_json_date(ld)?);
            }

            if let Some(lt) = lead_time_seconds {
                action_updates.lead_time = Some((lt.to_f64() * 1000.0) as i64);
            }

            store.dispatch(Action::UpdateTask {
                id: task_id,
                updates: action_updates,
            })?;
        }
    }

    if let Some(to_delete) = delete_tasks {
        for id in to_delete {
            let task_id = TaskID::from(id.clone());
            store.dispatch(Action::DeleteTask { id: task_id })?;
        }
    }

    if let Some(to_complete) = complete_tasks {
        for id in to_complete {
            let task_id = TaskID::from(id.clone());
            store.dispatch(Action::CompleteTask {
                id: task_id,
                current_time: *current_time,
            })?;
        }
    }

    Ok(())
}

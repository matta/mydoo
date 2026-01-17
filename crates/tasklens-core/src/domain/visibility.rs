use crate::types::{
    ANYWHERE_PLACE_ID, EnrichedTask, OpenHours, OpenHoursMode, Place, PlaceID, TunnelState,
    ViewFilter,
};
use chrono::{DateTime, Datelike, Timelike};

/// Calculates the contextual visibility for a list of tasks.
///
/// Filters tasks by physical context (location) and time.
/// Updates the `visibility` property of each task.
pub fn calculate_contextual_visibility(
    doc: &TunnelState,
    tasks: &mut [EnrichedTask],
    view_filter: &ViewFilter,
    current_time: i64,
) {
    for task in tasks {
        // 1. Resolve Effective Place
        let effective_place_id = task
            .place_id
            .clone()
            .unwrap_or_else(|| PlaceID::from(ANYWHERE_PLACE_ID));

        // Write back the resolved place (matching TS behavior)
        task.place_id = Some(effective_place_id.clone());

        let effective_place = doc.places.get(&effective_place_id);

        // 2. Hours Check (IsOpen)
        let is_open = if effective_place_id.as_str() == ANYWHERE_PLACE_ID {
            true
        } else if let Some(place) = effective_place {
            is_place_open(place, current_time)
        } else {
            false
        };

        // 3. Place Match
        let filter_match = match &view_filter.place_id {
            None => true, // Default to All
            Some(p) if p == "All" => true,
            Some(_) if effective_place_id.as_str() == ANYWHERE_PLACE_ID => true,
            Some(p) if p == effective_place_id.as_str() => true,
            Some(p) => {
                // Check if the task's place is included in the filter's place
                let filter_place_id = PlaceID::from(p.clone());
                if let Some(filter_place) = doc.places.get(&filter_place_id) {
                    filter_place.included_places.contains(&effective_place_id)
                } else {
                    false
                }
            }
        };

        task.visibility = is_open && filter_match;
    }
}

/// Checks if a place is currently open based on its schedule.
pub fn is_place_open(place: &Place, current_time: i64) -> bool {
    let open_hours: OpenHours = match serde_json::from_str(&place.hours) {
        Ok(h) => h,
        Err(_) => return true, // Fallback to open if invalid JSON
    };

    match open_hours.mode {
        OpenHoursMode::AlwaysOpen => true,
        OpenHoursMode::AlwaysClosed => false,
        OpenHoursMode::Custom => {
            let schedule = match open_hours.schedule {
                Some(s) => s,
                None => return false,
            };

            let dt = DateTime::from_timestamp_millis(current_time).unwrap_or(DateTime::UNIX_EPOCH);
            let day_of_week = match dt.weekday() {
                chrono::Weekday::Sun => "Sun",
                chrono::Weekday::Mon => "Mon",
                chrono::Weekday::Tue => "Tue",
                chrono::Weekday::Wed => "Wed",
                chrono::Weekday::Thu => "Thu",
                chrono::Weekday::Fri => "Fri",
                chrono::Weekday::Sat => "Sat",
            };

            if let Some(day_schedule) = schedule.get(day_of_week) {
                let current_minutes = dt.hour() * 60 + dt.minute();
                for range in day_schedule {
                    if let Some((start, end)) = parse_time_range(range)
                        && current_minutes >= start
                        && current_minutes < end
                    {
                        return true;
                    }
                }
            }
            false
        }
    }
}

fn parse_time_range(range: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parse_time(parts[0])?;
    let end = parse_time(parts[1])?;
    Some((start, end))
}

fn parse_time(time: &str) -> Option<u32> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let h = parts[0].parse::<u32>().ok()?;
    let m = parts[1].parse::<u32>().ok()?;
    Some(h * 60 + m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Schedule, ScheduleType, TaskID, TaskStatus};
    use std::collections::HashMap;

    fn mock_enriched_task(id: &str, place_id: Option<&str>) -> EnrichedTask {
        EnrichedTask {
            id: TaskID::from(id),
            title: id.to_string(),
            notes: "".to_string(),
            parent_id: None,
            child_task_ids: vec![],
            place_id: place_id.map(PlaceID::from),
            status: TaskStatus::Pending,
            importance: 1.0,
            credit_increment: None,
            credits: 0.0,
            desired_credits: 0.0,
            credits_timestamp: 0,
            priority_timestamp: 0,
            schedule: Schedule {
                schedule_type: ScheduleType::Once,
                due_date: None,
                lead_time: Some(0),
                last_done: None,
            },
            repeat_config: None,
            is_sequential: false,
            is_acknowledged: false,
            last_completed_at: None,
            effective_credits: 0.0,
            feedback_factor: 1.0,
            lead_time_factor: 1.0,
            normalized_importance: 1.0,
            priority: 0.0,
            visibility: true,
            outline_index: 0.0,
            is_container: false,
            is_pending: true,
            is_ready: true,
            effective_due_date: None,
            effective_lead_time: None,
            effective_schedule_source: None,
        }
    }

    #[test]
    fn test_calculate_contextual_visibility_basic() {
        let mut places = HashMap::new();
        places.insert(
            PlaceID::from("office"),
            Place {
                id: PlaceID::from("office"),
                name: "Office".to_string(),
                hours: r#"{"mode":"always_open"}"#.to_string(),
                included_places: vec![],
            },
        );

        let doc = TunnelState {
            next_task_id: 1,
            next_place_id: 1,
            tasks: HashMap::new(),
            root_task_ids: vec![],
            places,
            metadata: None,
        };

        let mut tasks = vec![
            mock_enriched_task("task1", Some("office")),
            mock_enriched_task("task2", None), // Anywhere
        ];

        // Filter: All
        calculate_contextual_visibility(
            &doc,
            &mut tasks,
            &ViewFilter {
                place_id: Some("All".to_string()),
            },
            0,
        );
        assert!(tasks[0].visibility);
        assert!(tasks[1].visibility);

        // Filter: office
        calculate_contextual_visibility(
            &doc,
            &mut tasks,
            &ViewFilter {
                place_id: Some("office".to_string()),
            },
            0,
        );
        assert!(tasks[0].visibility);
        assert!(tasks[1].visibility); // Anywhere matches everything

        // Filter: home
        calculate_contextual_visibility(
            &doc,
            &mut tasks,
            &ViewFilter {
                place_id: Some("home".to_string()),
            },
            0,
        );
        assert!(!tasks[0].visibility);
        assert!(tasks[1].visibility);
    }

    #[test]
    fn test_is_place_open_custom() {
        let place = Place {
            id: PlaceID::from("office"),
            name: "Office".to_string(),
            hours: r#"{"mode":"custom","schedule":{"Mon":["09:00-17:00"]}}"#.to_string(),
            included_places: vec![],
        };

        // Monday, January 12, 2026 at 10:00:00 UTC
        let monday_10am = 1768212000000;
        assert!(is_place_open(&place, monday_10am));

        // Monday, January 12, 2026 at 20:00:00 UTC (outside office hours)
        let monday_8pm = 1768248000000;
        assert!(!is_place_open(&place, monday_8pm));

        // Tuesday, January 13, 2026 at 10:00:00 UTC (no schedule for Tuesday)
        let tuesday_10am = 1768298400000;
        assert!(!is_place_open(&place, tuesday_10am));
    }
}

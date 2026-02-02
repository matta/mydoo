use dioxus::prelude::*;
use std::collections::HashMap;
use tasklens_core::domain::balance_distribution::redistribute_percentages;
use tasklens_core::types::{BalanceData, BalanceItem, TaskID};

#[derive(Clone, Copy)]
pub struct BalanceInteraction<G, U> {
    logic: Signal<BalanceInteractionLogic>,
    get_balance_data: G,
    on_update: U,
}

impl<G, U> BalanceInteraction<G, U>
where
    G: Fn() -> BalanceData,
    U: Fn(HashMap<TaskID, f64>),
{
    pub fn handle_input(mut self, target_id: TaskID, new_value: f64) {
        let current_data = (self.get_balance_data)();
        self.logic
            .write()
            .handle_input(&current_data, &target_id, new_value);
    }

    pub fn handle_change(mut self) {
        let current_data = (self.get_balance_data)();
        if let Some(update_map) = self.logic.write().handle_commit(&current_data) {
            (self.on_update)(update_map);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BalanceRenderItem {
    pub item: BalanceItem,
    pub preview_percent: Option<f64>,
}

/// Pure logic handler for balance interactions.
///
/// This struct separates the domain mathematics from the Dioxus state management.
/// It owns the transient state (preview_targets).
#[derive(Clone, Default)]
struct BalanceInteractionLogic {
    preview_targets: Option<HashMap<TaskID, f64>>,
}

impl BalanceInteractionLogic {
    fn new() -> Self {
        Self::default()
    }

    fn handle_input(&mut self, current_data: &BalanceData, target_id: &TaskID, new_value: f64) {
        let string_map = if let Some(preview) = &self.preview_targets {
            preview.clone()
        } else {
            // Initialize from current data
            let mut map = HashMap::new();
            for item in &current_data.items {
                map.insert(item.id.clone(), item.target_percent);
            }
            map
        };

        self.preview_targets = Some(redistribute_percentages(&string_map, target_id, new_value));
    }

    fn handle_commit(&mut self, current_data: &BalanceData) -> Option<HashMap<TaskID, f64>> {
        let preview = self.preview_targets.as_ref()?;
        let total_desired_sum: f64 = current_data.items.iter().map(|i| i.desired_credits).sum();

        // If total desired is 0, default to 100.0 (arbitrary baseline)
        let mut base_total = total_desired_sum;
        if base_total < 0.1 {
            base_total = 100.0;
        }

        let mut distribution_update = HashMap::new();
        for (id, pct) in preview {
            let absolute = pct * base_total;
            distribution_update.insert(id.clone(), absolute);
        }

        self.preview_targets = None;
        Some(distribution_update)
    }

    /// Computes the render items by merging actual data with the current preview state.
    pub(crate) fn compute_render_items(
        &self,
        balance_data: &BalanceData,
    ) -> Vec<BalanceRenderItem> {
        balance_data
            .items
            .iter()
            .map(|item| {
                let preview_percent = self
                    .preview_targets
                    .as_ref()
                    .and_then(|m| m.get(&item.id).copied());
                BalanceRenderItem {
                    item: item.clone(),
                    preview_percent,
                }
            })
            .collect()
    }
}

/// Hook to manage the interaction state for the Balance View.
///
/// It handles:
/// 1. The local "preview" state of sliders being dragged (Percent).
/// 2. The logic to redistribute percentages when one changes (Drain/Fill).
/// 3. The commit logic to convert percentages back to absolute credits.
pub fn use_balance_interaction<G, U>(
    get_balance_data: G,
    on_update: U,
) -> (Vec<BalanceRenderItem>, BalanceInteraction<G, U>)
where
    G: Fn() -> BalanceData + Copy + 'static,
    U: Fn(HashMap<TaskID, f64>) + Copy + 'static,
{
    // We wrap the purely functional/logical model in a Signal.
    let logic = use_signal(BalanceInteractionLogic::new);

    // Project the internal state for the view
    let render_items = logic.read().compute_render_items(&get_balance_data());
    tracing::info!("render_items: {:#?}", render_items);

    (
        render_items,
        BalanceInteraction {
            logic,
            get_balance_data,
            on_update,
        },
    )
}

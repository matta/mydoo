use crate::hooks::use_balance_data::use_balance_data;
use dioxus::prelude::*;
use std::collections::HashMap;
pub use tasklens_core::domain::balance_distribution::compute_balance_render_items;
use tasklens_core::domain::balance_distribution::{
    apply_redistribution_to_credits, redistribute_percentages,
};
pub use tasklens_core::types::BalanceRenderItem;
use tasklens_core::types::{BalanceData, TaskID};

#[derive(Clone, Copy)]
pub struct BalanceInteraction {
    preview_state: Signal<Option<HashMap<TaskID, f64>>>,
    get_balance_data: Memo<BalanceData>,
    on_update: EventHandler<HashMap<TaskID, f64>>,
}

impl BalanceInteraction {
    pub fn handle_input(mut self, target_id: TaskID, new_value: f64) {
        let base_map = {
            let current_data = self.get_balance_data.read();
            let preview = self.preview_state.read();

            if let Some(p) = preview.as_ref() {
                p.clone()
            } else {
                let mut map = HashMap::new();
                for item in &current_data.items {
                    map.insert(item.id.clone(), item.target_percent);
                }
                map
            }
        };

        let next_preview = redistribute_percentages(&base_map, &target_id, new_value);
        self.preview_state.set(Some(next_preview));
    }

    pub fn handle_change(mut self) {
        let preview_opt = { self.preview_state.read().clone() };
        if let Some(percentages) = preview_opt {
            let current_data = self.get_balance_data.read();
            let updates = apply_redistribution_to_credits(&current_data, &percentages);
            self.on_update.call(updates);
        }
        self.preview_state.set(None);
    }
}

/// Hook to manage the interaction state for the Balance View.
pub fn use_balance_interaction(
    on_update: EventHandler<HashMap<TaskID, f64>>,
) -> (Vec<BalanceRenderItem>, BalanceInteraction) {
    let balance_data = use_balance_data();
    let preview_state = use_signal(|| None);

    let render_items = use_memo(move || {
        let data = balance_data.read();
        let preview = preview_state.read();
        compute_balance_render_items(&data, &preview)
    });

    (
        render_items(),
        BalanceInteraction {
            preview_state,
            get_balance_data: balance_data,
            on_update,
        },
    )
}

use crate::hooks::use_balance_data::use_balance_data;
use dioxus::prelude::*;
use std::collections::HashMap;
use tasklens_core::domain::balance_distribution::{
    apply_redistribution_to_credits, redistribute_percentages,
};
pub(crate) use tasklens_core::types::BalanceItem;
use tasklens_core::types::{BalanceData, TaskID};

/// Manages the state and logic for interacting with the Balance View.
///
/// This struct handles the "preview" state—a temporary mapping of task IDs to their
/// redistributed percentages—while the user is actively adjusting values.
/// When the interaction is complete, it triggers a permanent update.
#[derive(Clone, Copy)]
pub(crate) struct BalanceInteraction {
    /// Temporary storage for redistributed percentages during user interaction.
    /// Set to `None` when there is no active interaction.
    preview_state: Signal<Option<HashMap<TaskID, f64>>>,
    /// Access to the current, persisted balance data.
    get_balance_data: Memo<BalanceData>,
    /// Callback triggered when a preview is committed to permanent storage.
    on_change: EventHandler<HashMap<TaskID, f64>>,
}

impl BalanceInteraction {
    /// Handles an incremental input change (e.g., while dragging a slider).
    ///
    /// This updates the `preview_state` by redistributing the total percentage
    /// across all tasks based on the `new_value` for the given `target_id`.
    pub(crate) fn handle_input(mut self, target_id: TaskID, new_value: f64) {
        let base_map = {
            let current_data = self.get_balance_data.read();
            let preview = self.preview_state.read();

            preview
                .clone()
                .unwrap_or_else(|| current_data.get_percentage_map())
        };

        let next_preview = redistribute_percentages(&base_map, &target_id, new_value);
        self.preview_state.set(Some(next_preview));
    }

    /// Handles a final change event (e.g., when a slider is released).
    ///
    /// This takes the current `preview_state`, calculates the new credit values
    /// for each task, and calls the `on_update` handler to persist the changes.
    /// Finally, it resets the `preview_state` to `None`.
    pub(crate) fn handle_change(mut self) {
        let preview_opt = { self.preview_state.read().clone() };
        if let Some(percentages) = preview_opt {
            let current_data = self.get_balance_data.read();
            let distribution = apply_redistribution_to_credits(&current_data, &percentages);
            self.on_change.call(distribution);
        }
        self.preview_state.set(None);
    }
}

/// Hook to manage the interaction state for the Balance View.
///
/// This hook provides:
/// 1. A list of [`BalanceItem`]s to render, which automatically reflect any active
///    previews during interaction.
/// 2. A [`BalanceInteraction`] handle to feed user input events back into the system.
///
/// # Arguments
/// * `on_change` - An event handler that will be called with the final credit adjustments
///   when an interaction is committed.
pub(crate) fn use_balance_interaction(
    on_change: EventHandler<HashMap<TaskID, f64>>,
) -> (Vec<BalanceItem>, BalanceInteraction) {
    let balance_data = use_balance_data();
    let preview_state = use_signal(|| None);

    let render_items = use_memo(move || {
        let data = balance_data.read();
        let preview = preview_state.read();
        data.apply_previews(&preview)
    });

    (
        render_items(),
        BalanceInteraction {
            preview_state,
            get_balance_data: balance_data,
            on_change,
        },
    )
}

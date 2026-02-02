//! Balance Data Hook
//!
//! Provides reactive balance data for the Balance View.

use dioxus::prelude::*;
use tasklens_core::get_balance_data;
use tasklens_core::types::BalanceData;

/// A hook that computes balance data from the current tunnel state.
///
/// This hook automatically recomputes when the underlying document changes.
/// It returns a memoized `BalanceData` containing:
/// - List of balance items (one per root goal, excluding Inbox)
/// - Total effective credits across all roots
///
/// Each balance item includes:
/// - Target percent (user's desired allocation)
/// - Actual percent (computed from effective credits)
/// - Starving flag (true if under-served)
pub fn use_balance_data() -> Memo<BalanceData> {
    let tunnel_state = crate::hooks::use_tunnel_state::use_tunnel_state();

    use_memo(move || {
        tracing::info!("use_balance_data: computing");
        let state = tunnel_state.read();
        get_balance_data(&state)
    })
}

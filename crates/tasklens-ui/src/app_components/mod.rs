//! App-specific UI components and composition helpers.
//!
//! This module contains UI elements that are specific to TaskLens app flows.
//! Shared and vendored component primitives remain under `components` and
//! `dioxus_components` during migration.

mod app_navbar;
mod back_button;
mod balance_slider;
mod date_time_inputs;
mod doc_id_manager;
mod empty_state;
mod load_error_view;
mod loading;
mod move_picker;
mod page_header;
mod priority_task_row;
mod search_panel;
mod sync_indicator;
mod task_editor;
mod task_input;
mod task_row;

#[allow(unreachable_pub)]
pub use app_navbar::AppNavBar;
#[allow(unreachable_pub)]
pub use back_button::BackButton;
#[allow(unreachable_pub)]
pub use balance_slider::BalanceSlider;
#[allow(unreachable_pub)]
pub use date_time_inputs::DateInput;
#[allow(unreachable_pub)]
pub use date_time_inputs::DateTimeInput;
#[allow(unreachable_pub)]
pub use doc_id_manager::DocIdManager;
#[allow(unreachable_pub)]
pub use empty_state::EmptyState;
#[allow(unreachable_pub)]
pub use load_error_view::LoadErrorView;
#[allow(unreachable_pub)]
pub use loading::Loading;
#[allow(unreachable_pub)]
pub use move_picker::MovePicker;
#[allow(unreachable_pub)]
pub use page_header::PageHeader;
#[allow(unreachable_pub)]
pub use priority_task_row::PriorityTaskRow;
#[allow(unreachable_pub)]
pub use search_panel::SearchPanel;
#[allow(unreachable_pub)]
pub use sync_indicator::SyncIndicator;
#[allow(unreachable_pub)]
pub use task_editor::TaskEditor;
#[allow(unreachable_pub)]
pub use task_input::TaskInput;
#[allow(unreachable_pub)]
pub use task_row::TaskRow;

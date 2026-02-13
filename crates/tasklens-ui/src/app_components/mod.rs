//! App-specific UI components and composition helpers.
//!
//! This module contains UI elements that are specific to TaskLens app flows.
//! Shared and vendored component primitives remain under `components` and
//! `dioxus_components` during migration.

pub mod app_navbar;
pub mod back_button;
pub mod balance_slider;
mod date_time_inputs;
pub mod doc_id_manager;
pub mod empty_state;
pub mod load_error_view;
pub mod loading;
pub mod move_picker;
pub mod page_header;
pub mod priority_task_row;
pub mod search_panel;
pub mod sync_indicator;
pub mod task_editor;
pub mod task_input;
pub mod task_row;

pub(crate) use back_button::BackButton;
pub(crate) use balance_slider::BalanceSlider;
pub(crate) use date_time_inputs::DateInput;
#[allow(unused_imports)]
pub(crate) use date_time_inputs::DateTimeInput;
pub(crate) use doc_id_manager::DocIdManager;
pub(crate) use empty_state::EmptyState;
pub use load_error_view::LoadErrorView;
#[allow(unused_imports)]
pub(crate) use move_picker::MovePicker;
pub(crate) use page_header::PageHeader;
#[allow(unused_imports)]
pub(crate) use priority_task_row::PriorityTaskRow;
#[allow(unused_imports)]
pub(crate) use sync_indicator::SyncIndicator;
pub(crate) use task_editor::TaskEditor;
pub(crate) use task_input::TaskInput;

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

pub(crate) use app_navbar::AppNavBar;
pub(crate) use back_button::BackButton;
pub(crate) use balance_slider::BalanceSlider;
pub(crate) use date_time_inputs::DateInput;
pub(crate) use doc_id_manager::DocIdManager;
pub(crate) use empty_state::EmptyState;
pub(crate) use load_error_view::LoadErrorView;
pub(crate) use loading::Loading;
pub(crate) use move_picker::MovePicker;
pub(crate) use page_header::PageHeader;
pub(crate) use priority_task_row::PriorityTaskRow;
pub(crate) use search_panel::SearchPanel;
pub(crate) use sync_indicator::SyncIndicator;
pub(crate) use task_editor::TaskEditor;
pub(crate) use task_input::TaskInput;
pub(crate) use task_row::TaskRow;

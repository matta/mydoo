//! Reusable UI components for the Todo application.
//!
//! This module contains generic, accessible, and stylable components used throughout the application.

pub mod alert;
pub mod back_button;
pub mod button;
pub mod checkbox;
pub mod input;
pub mod sync_indicator;

pub(crate) use alert::{Alert, AlertVariant};
pub(crate) use button::{Button, ButtonVariant};
pub(crate) use checkbox::Checkbox;
pub(crate) use input::Input;
pub(crate) use sync_indicator::SyncIndicator;
pub(crate) use task_input::TaskInput;
pub mod app_navbar;
pub mod calendar;
pub mod collapsible;
pub mod date_picker;
pub mod dialog;
pub mod doc_id_manager;
pub mod load_error_view;
pub mod loading;
pub mod navbar;
pub mod popover;
pub mod priority_task_row;
pub mod select;
pub mod slider;
pub mod task_editor;
pub mod task_input;
pub mod task_row;
pub(crate) use back_button::BackButton;
pub(crate) use date_picker::DatePicker;
pub(crate) use doc_id_manager::DocIdManager;
pub use load_error_view::LoadErrorView;
pub(crate) use select::Select;
pub(crate) use task_editor::TaskEditor;
pub mod move_picker;
pub(crate) use move_picker::MovePicker;
pub mod balance_slider;
pub(crate) use balance_slider::BalanceSlider;
pub mod empty_state;
pub(crate) use empty_state::EmptyState;
pub mod page_header;
pub(crate) use page_header::PageHeader;

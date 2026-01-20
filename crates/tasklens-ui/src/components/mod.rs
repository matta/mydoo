//! Reusable UI components for the Todo application.
//!
//! This module contains generic, accessible, and stylable components used throughout the application.

pub mod alert;
pub mod back_button;
pub mod button;
pub mod checkbox;
pub mod input;
pub mod sync_indicator;

pub use alert::{Alert, AlertVariant};
pub use button::{Button, ButtonVariant};
pub use checkbox::Checkbox;
pub use input::Input;
pub use sync_indicator::SyncIndicator;
pub use task_input::TaskInput;
pub mod app_navbar;
pub mod calendar;
pub mod collapsible;
pub mod date_picker;
pub mod dialog;
pub mod doc_id_manager;
pub mod loading;
pub mod navbar;
pub mod popover;
pub mod priority_task_row;
pub mod select;
pub mod slider;
pub mod task_editor;
pub mod task_input;
pub mod task_row;
pub use date_picker::DatePicker;
pub use doc_id_manager::DocIdManager;
pub use select::Select;
pub use task_editor::TaskEditor;
pub mod move_picker;
pub use move_picker::MovePicker;

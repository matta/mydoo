//! Compatibility shim for legacy `crate::components` imports.
//!
//! App-specific modules now live in `crate::app_components`.
//! This module keeps old import paths working during migration.

pub mod alert;
pub mod calendar;
pub mod collapsible;
pub mod dialog;
pub mod navbar;
pub mod popover;
pub mod slider;
pub use crate::dioxus_components::button;
pub use crate::dioxus_components::checkbox;
pub use crate::dioxus_components::input;
pub use crate::dioxus_components::select;

pub use crate::app_components::app_navbar;
pub use crate::app_components::back_button;
pub use crate::app_components::balance_slider;
pub use crate::app_components::doc_id_manager;
pub use crate::app_components::empty_state;
pub use crate::app_components::load_error_view;
pub use crate::app_components::loading;
pub use crate::app_components::move_picker;
pub use crate::app_components::page_header;
pub use crate::app_components::priority_task_row;
pub use crate::app_components::search_panel;
pub use crate::app_components::sync_indicator;
pub use crate::app_components::task_editor;
pub use crate::app_components::task_input;
pub use crate::app_components::task_row;

pub use crate::dioxus_components::button::{Button, ButtonVariant};
pub use crate::dioxus_components::checkbox::Checkbox;
pub use crate::dioxus_components::input::Input;
pub use crate::dioxus_components::select::{
    Select, SelectList, SelectOption, SelectTrigger, SelectValue,
};
pub(crate) use alert::{Alert, AlertVariant};

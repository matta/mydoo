//! Compatibility shim for legacy `crate::components` imports.
//!
//! App-specific modules now live in `crate::app_components`.
//! This module keeps old import paths working during migration.

pub mod alert;
pub mod button;
pub mod calendar;
pub mod checkbox;
pub mod collapsible;
pub mod date_picker;
pub mod dialog;
pub mod input;
pub mod navbar;
pub mod popover;
pub mod select;
pub mod slider;

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

pub(crate) use alert::{Alert, AlertVariant};
pub(crate) use button::{Button, ButtonVariant};
pub(crate) use checkbox::Checkbox;
pub(crate) use input::Input;

#[allow(unused_imports)]
pub(crate) use crate::app_components::back_button::BackButton;
#[allow(unused_imports)]
pub(crate) use crate::app_components::balance_slider::BalanceSlider;
#[allow(unused_imports)]
pub(crate) use crate::app_components::doc_id_manager::DocIdManager;
#[allow(unused_imports)]
pub(crate) use crate::app_components::empty_state::EmptyState;
pub use crate::app_components::load_error_view::LoadErrorView;
#[allow(unused_imports)]
pub(crate) use crate::app_components::move_picker::MovePicker;
#[allow(unused_imports)]
pub(crate) use crate::app_components::page_header::PageHeader;
pub(crate) use crate::app_components::sync_indicator::SyncIndicator;
#[allow(unused_imports)]
pub(crate) use crate::app_components::task_editor::TaskEditor;
#[allow(unused_imports)]
pub(crate) use crate::app_components::task_input::TaskInput;

pub(crate) use date_picker::DatePicker;
pub(crate) use select::Select;

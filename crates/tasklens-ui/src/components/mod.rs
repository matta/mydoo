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

pub use crate::app_components::{
    AppNavBar, BackButton, BalanceSlider, DocIdManager, EmptyState, LoadErrorView, Loading,
    MovePicker, PageHeader, PriorityTaskRow, SearchPanel, SyncIndicator, TaskEditor, TaskInput,
    TaskRow,
};

pub use crate::dioxus_components::button::{Button, ButtonVariant};
pub use crate::dioxus_components::checkbox::Checkbox;
pub use crate::dioxus_components::input::Input;
pub use crate::dioxus_components::select::{
    Select, SelectList, SelectOption, SelectTrigger, SelectValue,
};
pub(crate) use alert::{Alert, AlertVariant};

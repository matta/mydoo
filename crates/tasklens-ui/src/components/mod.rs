//! Compatibility shim for legacy `crate::components` imports.
//!
//! App-specific modules now live in `crate::app_components`.
//! This module keeps old import paths working during migration.

pub(crate) mod alert;
pub(crate) mod calendar;
pub(crate) mod collapsible;
pub(crate) mod dialog;
pub(crate) mod navbar;
pub(crate) mod popover;
pub(crate) mod slider;

pub(crate) use alert::{Alert, AlertVariant};

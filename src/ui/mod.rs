//! User-interface components and view composition.
//!
//! The implementation files are currently kept at the crate root while the
//! application is migrated incrementally. Keeping the public UI boundary here
//! lets the app stop depending on the eventual file layout all at once.

pub(crate) mod dashboard;
pub(crate) mod format;
pub(crate) mod history_chart;
pub(crate) mod power_flow;
pub(crate) mod settings;
pub(crate) mod shell;

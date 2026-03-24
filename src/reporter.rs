// reporter.rs — public re-export shim for the report module.
//
// The canonical implementation lives in `crate::report`.  This module is
// provided as an alternative entry-point so that callers can refer to either
// `git_ghosts::report` or `git_ghosts::reporter` interchangeably.
pub use crate::report::{format_report, render_report};

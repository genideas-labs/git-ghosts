pub mod cache;
pub mod cli;
pub mod detect;
pub mod detectors;
pub mod models;

pub use cache::{load_cache, save_cache, CacheError};
pub use detectors::detect_ghost_files;
pub use detectors::detect_orphan_commits;
pub use detectors::detect_zombie_branches;
pub use models::{GhostFile, OrphanCommit, ScanResults, ZombieBranch};
pub mod report;
pub use report::{format_report, render_report};
pub mod reporter;
pub use cli::{format_clean_dry_run, run_clean_dry_run};

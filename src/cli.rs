use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::{detect_ghost_files, detect_orphan_commits, detect_zombie_branches};
use crate::{save_cache, ScanResults};

#[path = "cli/report.rs"]
pub mod report;
pub use report::run_report;

#[path = "cli/clean.rs"]
pub mod clean;
pub use clean::{format_clean_dry_run, run_clean_dry_run};

/// git-ghosts: find deleted files, stale branches, and orphaned commits.
#[derive(Parser)]
#[command(name = "git-ghosts", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a git repository for ghosts, zombie branches, and orphan commits.
    Scan {
        /// Path to the git repository (defaults to the current directory).
        path: Option<PathBuf>,

        /// Minimum age in days for a branch to be considered a zombie (must be > 0).
        #[arg(long)]
        threshold: Option<u32>,
    },

    /// Display the most recent cached scan report without re-scanning.
    Report {
        /// Path to the git repository (defaults to the current directory).
        path: Option<PathBuf>,
    },

    /// Preview what `clean` would do without modifying the repository.
    /// The --dry-run flag is mandatory.
    Clean {
        /// Path to the git repository (defaults to the current directory).
        path: Option<PathBuf>,

        /// Required: preview actions without modifying the repository.
        #[arg(long)]
        dry_run: bool,
    },
}

/// Top-level entry point called by `main.rs`.  Signature must remain `pub fn run()`.
pub fn run() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan { path, threshold } => {
            let resolved = match resolve_path(path) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            };
            if let Err(e) = run_scan(&resolved, threshold) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Report { path } => {
            let resolved = match resolve_path(path) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            };
            if let Err(e) = run_report(&resolved) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Clean { path, dry_run } => {
            if !dry_run {
                eprintln!(
                    "error: --dry-run is required; refusing to modify the repository without it"
                );
                std::process::exit(1);
            }
            let resolved = match resolve_path(path) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            };
            if let Err(e) = run_clean_dry_run(&resolved) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// Resolve an optional path to a concrete `PathBuf`.
///
/// Returns the provided path unchanged, or falls back to `std::env::current_dir()`.
fn resolve_path(path: Option<PathBuf>) -> Result<PathBuf, String> {
    match path {
        Some(p) => Ok(p),
        None => std::env::current_dir()
            .map_err(|e| format!("could not determine current directory: {}", e)),
    }
}

/// Validate that `path` is a git repository, run all three detectors, aggregate
/// the results into a `ScanResults`, and persist them via `save_cache`.
///
/// Exposed publicly so integration tests can invoke scan logic without parsing
/// `std::env::args`.
pub fn run_scan(path: &Path, threshold: Option<u32>) -> Result<(), String> {
    // 1. Validate that the path is a git repository.
    git2::Repository::open(path)
        .map_err(|e| format!("'{}' is not a valid git repository: {}", path.display(), e))?;

    // 2. Reject threshold == 0 before calling the detector (which would also
    //    reject it, but we want a clear user-facing message).
    if threshold == Some(0) {
        return Err("threshold must be greater than 0; a value of 0 is not meaningful".to_string());
    }

    // 3. Run all three detectors, converting errors to String.
    let ghost_files =
        detect_ghost_files(path).map_err(|e| format!("ghost-file detection failed: {}", e))?;

    let zombie_branches = detect_zombie_branches(path, threshold.map(|t| t as i64))
        .map_err(|e| format!("zombie-branch detection failed: {}", e))?;

    let orphan_commits = detect_orphan_commits(path)
        .map_err(|e| format!("orphan-commit detection failed: {}", e))?;

    // 4. Assemble and persist the results.
    let results = ScanResults {
        ghost_files,
        zombie_branches,
        orphan_commits,
    };

    save_cache(path, &results).map_err(|e| format!("failed to write cache: {}", e))?;

    Ok(())
}

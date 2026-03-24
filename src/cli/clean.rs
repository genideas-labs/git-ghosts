use std::path::Path;

use colored::Colorize;

use crate::{load_cache, CacheError, ScanResults};

/// Format the dry-run clean plan for the given `ScanResults`.
///
/// Returns a `String` containing one line per zombie branch and one line per
/// orphan commit.  Ghost-file entries are intentionally excluded (no clean
/// action is defined for them).  Branch-deletion lines are rendered in
/// yellow/bold; orphan-removal lines in red/bold.
pub fn format_clean_dry_run(results: &ScanResults) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Zombie branches first — stable insertion order.
    for branch in &results.zombie_branches {
        lines.push(
            format!("[dry-run] would delete branch: {}", branch.branch_name)
                .yellow()
                .bold()
                .to_string(),
        );
    }

    // Orphan commits second — stable insertion order.
    for commit in &results.orphan_commits {
        lines.push(
            format!(
                "[dry-run] would remove orphan commit: {}",
                commit.commit_hash
            )
            .red()
            .bold()
            .to_string(),
        );
    }

    // ghost_files are intentionally skipped — no clean action defined for them.

    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n")
    }
}

/// Load the cached `ScanResults` from `<path>/.git/git-ghosts-cache.json` and
/// print the dry-run clean plan to stdout.
///
/// Returns `Err` with an actionable message if no cache exists or if the cache
/// cannot be read/parsed.  No git objects, refs, or files are modified.
pub fn run_clean_dry_run(path: &Path) -> Result<(), String> {
    match load_cache(path) {
        Ok(results) => {
            let output = format_clean_dry_run(&results);
            if !output.is_empty() {
                println!("{}", output);
            }
            Ok(())
        }
        Err(CacheError::NotFound) => {
            Err("No scan cache found. Run git-ghosts scan first.".to_string())
        }
        Err(CacheError::Io(e)) => Err(format!("Failed to read cache: {}", e)),
        Err(CacheError::Json(e)) => Err(format!("Failed to parse cache: {}", e)),
    }
}

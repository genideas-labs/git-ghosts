use std::path::Path;

use crate::{load_cache, render_report, CacheError};

/// Load the cached `ScanResults` from `<path>/.git/git-ghosts-cache.json` and
/// render them to stdout.
///
/// Returns `Err` with an actionable message if no cache exists or if the cache
/// cannot be read/parsed.  No detector function is invoked.
pub fn run_report(path: &Path) -> Result<(), String> {
    match load_cache(path) {
        Ok(results) => {
            render_report(&results);
            Ok(())
        }
        Err(CacheError::NotFound) => {
            Err("No scan cache found. Run git-ghosts scan first.".to_string())
        }
        Err(CacheError::Io(e)) => Err(format!("Failed to read cache: {}", e)),
        Err(CacheError::Json(e)) => Err(format!("Failed to parse cache: {}", e)),
    }
}

use std::io;
use std::path::Path;

use crate::models::ScanResults;

const CACHE_FILENAME: &str = "git-ghosts-cache.json";

/// Errors that can occur during cache operations.
#[derive(Debug)]
pub enum CacheError {
    /// The cache file does not exist yet.
    NotFound,
    /// An I/O error occurred while reading or writing the cache file.
    Io(io::Error),
    /// The cache file contained invalid JSON.
    Json(serde_json::Error),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::NotFound => write!(f, "no cache file found"),
            CacheError::Io(e) => write!(f, "I/O error: {}", e),
            CacheError::Json(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CacheError::Io(e) => Some(e),
            CacheError::Json(e) => Some(e),
            CacheError::NotFound => None,
        }
    }
}

fn cache_path(repo_path: &Path) -> std::path::PathBuf {
    repo_path.join(".git").join(CACHE_FILENAME)
}

/// Serialize `results` and write them to `<repo_path>/.git/git-ghosts-cache.json`.
pub fn save_cache(repo_path: &Path, results: &ScanResults) -> Result<(), CacheError> {
    let json = serde_json::to_string_pretty(results).map_err(CacheError::Json)?;
    std::fs::write(cache_path(repo_path), json).map_err(CacheError::Io)
}

/// Read and deserialize `<repo_path>/.git/git-ghosts-cache.json`.
///
/// Returns `CacheError::NotFound` when no cache file exists (cache miss).
pub fn load_cache(repo_path: &Path) -> Result<ScanResults, CacheError> {
    let path = cache_path(repo_path);
    let data = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            CacheError::NotFound
        } else {
            CacheError::Io(e)
        }
    })?;
    serde_json::from_str(&data).map_err(CacheError::Json)
}

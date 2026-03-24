use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use git_ghosts::cli::run_scan;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gg-scan-{}-{}", label, nanos))
}

fn init_repo(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).unwrap();
    let out = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git init failed: {:?}", out);
}

fn make_commit(dir: &std::path::Path) {
    // Configure a minimal identity so git commit works in CI.
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .unwrap();

    // Write a file, stage it, and commit.
    std::fs::write(dir.join("README.md"), b"hello").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .unwrap();
    let out = Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git commit failed: {:?}", out);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// `run_scan` on a fresh repo with one commit succeeds and writes the cache.
#[test]
fn scan_creates_cache_file() {
    let dir = temp_dir("creates");
    init_repo(&dir);
    make_commit(&dir);

    let result = run_scan(&dir, None);
    assert!(result.is_ok(), "expected Ok(()), got: {:?}", result);

    let cache = dir.join(".git").join("git-ghosts-cache.json");
    assert!(
        cache.exists(),
        "cache file should exist at .git/git-ghosts-cache.json"
    );
}

/// `run_scan` on a plain directory (no `.git`) returns an `Err`.
#[test]
fn scan_non_git_dir_errors() {
    let dir = temp_dir("nongit");
    std::fs::create_dir_all(&dir).unwrap();

    let result = run_scan(&dir, None);
    assert!(result.is_err(), "expected Err for non-git directory");
}

/// `run_scan` with `threshold = Some(0)` returns an `Err` mentioning threshold.
#[test]
fn scan_threshold_zero_errors() {
    let dir = temp_dir("thresh0");
    init_repo(&dir);
    make_commit(&dir);

    let result = run_scan(&dir, Some(0));
    assert!(result.is_err(), "expected Err for threshold 0");
    let msg = result.unwrap_err();
    assert!(
        msg.to_lowercase().contains("threshold"),
        "error should mention 'threshold', got: {}",
        msg
    );
}

/// `run_scan` with `threshold = Some(1)` (smallest positive value) succeeds.
#[test]
fn scan_threshold_one_accepted() {
    let dir = temp_dir("thresh1");
    init_repo(&dir);
    make_commit(&dir);

    let result = run_scan(&dir, Some(1));
    assert!(result.is_ok(), "expected Ok(()), got: {:?}", result);
}

/// Running `run_scan` twice on the same repo overwrites the cache and still succeeds.
#[test]
fn scan_overwrites_existing_cache() {
    let dir = temp_dir("overwrite");
    init_repo(&dir);
    make_commit(&dir);

    let first = run_scan(&dir, None);
    assert!(first.is_ok(), "first scan failed: {:?}", first);

    let second = run_scan(&dir, None);
    assert!(second.is_ok(), "second scan failed: {:?}", second);

    let cache = dir.join(".git").join("git-ghosts-cache.json");
    assert!(
        cache.exists(),
        "cache file should still exist after second scan"
    );
}

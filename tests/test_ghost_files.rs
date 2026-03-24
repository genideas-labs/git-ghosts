use git_ghosts::detect_ghost_files;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Creates a unique temp directory path.
fn temp_repo_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gg-{}-{}", label, nanos))
}

/// Runs a git command in `dir` and panics on failure.
fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run git {:?}: {}", args, e));
    if !status.status.success() {
        panic!(
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&status.stdout),
            String::from_utf8_lossy(&status.stderr),
        );
    }
}

/// Initialises a new git repo with a fixed identity.
fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init"]);
    git(dir, &["config", "user.email", "test@example.com"]);
    git(dir, &["config", "user.name", "TestUser"]);
}

// ---------------------------------------------------------------------------
// Basic deletion
// ---------------------------------------------------------------------------

#[test]
fn test_basic_deletion() {
    let dir = temp_repo_dir("basic");
    init_repo(&dir);

    std::fs::write(dir.join("ghost.txt"), "Hello ghost").unwrap(); // 11 bytes
    std::fs::write(dir.join("keep.txt"), "Hello keep").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add files"]);

    std::fs::remove_file(dir.join("ghost.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "delete ghost.txt"]);

    let ghosts = detect_ghost_files(&dir).unwrap();

    assert_eq!(ghosts.len(), 1, "expected exactly one ghost file");
    assert_eq!(ghosts[0].file_path, "ghost.txt");
    assert_eq!(ghosts[0].author, "TestUser");
    assert_eq!(ghosts[0].original_file_size_bytes, 11);
    assert!(!ghosts[0].deletion_commit_hash.is_empty());
    assert!(!ghosts.iter().any(|g| g.file_path == "keep.txt"));
}

// ---------------------------------------------------------------------------
// Deletion commit hash format
// ---------------------------------------------------------------------------

#[test]
fn test_deletion_commit_hash_is_40_hex_chars() {
    let dir = temp_repo_dir("hash");
    init_repo(&dir);

    std::fs::write(dir.join("a.txt"), "data").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add"]);

    std::fs::remove_file(dir.join("a.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "delete"]);

    let ghosts = detect_ghost_files(&dir).unwrap();
    assert_eq!(ghosts.len(), 1);
    let hash = &ghosts[0].deletion_commit_hash;
    assert_eq!(hash.len(), 40, "hash must be 40 chars, got: {}", hash);
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "hash must be all hex, got: {}",
        hash
    );
}

// ---------------------------------------------------------------------------
// Non-git directory → Err
// ---------------------------------------------------------------------------

#[test]
fn test_non_git_dir_returns_err() {
    let dir = temp_repo_dir("nongit");
    std::fs::create_dir_all(&dir).unwrap();
    let result = detect_ghost_files(&dir);
    assert!(result.is_err(), "expected Err for non-git directory");
}

// ---------------------------------------------------------------------------
// Non-existent path → Err
// ---------------------------------------------------------------------------

#[test]
fn test_nonexistent_path_returns_err() {
    let path = temp_repo_dir("nonexistent"); // never created
    let result = detect_ghost_files(&path);
    assert!(result.is_err(), "expected Err for nonexistent path");
}

// ---------------------------------------------------------------------------
// Empty repo (no commits) → Ok(vec![])
// ---------------------------------------------------------------------------

#[test]
fn test_empty_repo_returns_empty_vec() {
    let dir = temp_repo_dir("empty");
    init_repo(&dir);

    let ghosts = detect_ghost_files(&dir).unwrap();
    assert!(
        ghosts.is_empty(),
        "expected empty vec for repo with no commits"
    );
}

// ---------------------------------------------------------------------------
// Repo with commits but no deletions → Ok(vec![])
// ---------------------------------------------------------------------------

#[test]
fn test_no_deletions_returns_empty_vec() {
    let dir = temp_repo_dir("nodels");
    init_repo(&dir);

    std::fs::write(dir.join("a.txt"), "aaa").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add a"]);

    std::fs::write(dir.join("b.txt"), "bbb").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add b"]);

    let ghosts = detect_ghost_files(&dir).unwrap();
    assert!(
        ghosts.is_empty(),
        "expected empty vec when no files deleted"
    );
}

// ---------------------------------------------------------------------------
// Re-add-then-delete: second deletion wins
// ---------------------------------------------------------------------------

#[test]
fn test_readd_then_delete_shows_latest_deletion() {
    let dir = temp_repo_dir("readd");
    init_repo(&dir);

    // First version
    std::fs::write(dir.join("cycle.txt"), "version one").unwrap(); // 11 bytes
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add cycle.txt v1"]);

    // First deletion
    std::fs::remove_file(dir.join("cycle.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "delete cycle.txt first"]);

    // Re-add with different content
    std::fs::write(dir.join("cycle.txt"), "version two longer content").unwrap(); // 26 bytes
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "re-add cycle.txt v2"]);

    // Second deletion
    std::fs::remove_file(dir.join("cycle.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    let second_del = {
        let out = Command::new("git")
            .args(["commit", "-m", "delete cycle.txt second"])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(out.status.success());
        // Capture the commit hash after the commit
        let hash_out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        String::from_utf8_lossy(&hash_out.stdout).trim().to_string()
    };

    let ghosts = detect_ghost_files(&dir).unwrap();

    assert_eq!(ghosts.len(), 1, "expected exactly one entry for cycle.txt");
    assert_eq!(ghosts[0].file_path, "cycle.txt");
    assert_eq!(
        ghosts[0].deletion_commit_hash, second_del,
        "should point to most-recent deletion"
    );
    assert_eq!(
        ghosts[0].original_file_size_bytes, 26,
        "size should reflect blob at second deletion"
    );
}

// ---------------------------------------------------------------------------
// Re-add currently tracked: file not in result
// ---------------------------------------------------------------------------

#[test]
fn test_readd_currently_tracked_not_in_result() {
    let dir = temp_repo_dir("retrack");
    init_repo(&dir);

    std::fs::write(dir.join("phoenix.txt"), "born again").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add phoenix"]);

    std::fs::remove_file(dir.join("phoenix.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "delete phoenix"]);

    // Re-add — now currently tracked
    std::fs::write(dir.join("phoenix.txt"), "reborn").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "re-add phoenix"]);

    let ghosts = detect_ghost_files(&dir).unwrap();
    assert!(
        !ghosts.iter().any(|g| g.file_path == "phoenix.txt"),
        "currently-tracked file must not appear in ghosts"
    );
}

// ---------------------------------------------------------------------------
// Multiple files deleted in a single commit
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_files_deleted_in_one_commit() {
    let dir = temp_repo_dir("multi");
    init_repo(&dir);

    std::fs::write(dir.join("alpha.txt"), "aaa").unwrap();
    std::fs::write(dir.join("beta.txt"), "bbb").unwrap();
    std::fs::write(dir.join("gamma.txt"), "ccc").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add three files"]);

    std::fs::remove_file(dir.join("alpha.txt")).unwrap();
    std::fs::remove_file(dir.join("beta.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "delete alpha and beta"]);

    let ghosts = detect_ghost_files(&dir).unwrap();
    let paths: std::collections::HashSet<&str> =
        ghosts.iter().map(|g| g.file_path.as_str()).collect();

    assert_eq!(ghosts.len(), 2);
    assert!(paths.contains("alpha.txt"));
    assert!(paths.contains("beta.txt"));
    assert!(!paths.contains("gamma.txt"));
}

// ---------------------------------------------------------------------------
// Files with same basename but different paths are distinct
// ---------------------------------------------------------------------------

#[test]
fn test_same_basename_different_paths_distinct() {
    let dir = temp_repo_dir("samename");
    init_repo(&dir);

    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("file.txt"), "root").unwrap();
    std::fs::write(dir.join("sub").join("file.txt"), "sub").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add files"]);

    std::fs::remove_file(dir.join("file.txt")).unwrap();
    std::fs::remove_file(dir.join("sub").join("file.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "delete both file.txt"]);

    let ghosts = detect_ghost_files(&dir).unwrap();
    let paths: std::collections::HashSet<&str> =
        ghosts.iter().map(|g| g.file_path.as_str()).collect();

    assert_eq!(ghosts.len(), 2, "should have two distinct ghost entries");
    assert!(paths.contains("file.txt"));
    assert!(paths.contains("sub/file.txt"));
}

// ---------------------------------------------------------------------------
// Zero-byte file deletion
// ---------------------------------------------------------------------------

#[test]
fn test_zero_byte_file_deletion() {
    let dir = temp_repo_dir("zerobyte");
    init_repo(&dir);

    std::fs::write(dir.join("empty.txt"), "").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add empty file"]);

    std::fs::remove_file(dir.join("empty.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "delete empty file"]);

    let ghosts = detect_ghost_files(&dir).unwrap();
    assert_eq!(ghosts.len(), 1);
    assert_eq!(ghosts[0].file_path, "empty.txt");
    assert_eq!(ghosts[0].original_file_size_bytes, 0);
}

// ---------------------------------------------------------------------------
// Timestamp is a plausible Unix epoch (after year 2000)
// ---------------------------------------------------------------------------

#[test]
fn test_timestamp_is_plausible_unix_epoch() {
    let dir = temp_repo_dir("ts");
    init_repo(&dir);

    std::fs::write(dir.join("ts.txt"), "ts").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "add"]);

    std::fs::remove_file(dir.join("ts.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "delete"]);

    let ghosts = detect_ghost_files(&dir).unwrap();
    assert_eq!(ghosts.len(), 1);
    // Unix timestamp for 2000-01-01
    assert!(
        ghosts[0].timestamp > 946_684_800,
        "timestamp should be after year 2000"
    );
}

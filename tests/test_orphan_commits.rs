use git_ghosts::detect_orphan_commits;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_repo_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gg-oc-{}-{}", label, nanos))
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run git {:?}: {}", args, e));
    if !out.status.success() {
        panic!(
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn git_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> String {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dir);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("failed to run git {:?}: {}", args, e));
    if !out.status.success() {
        panic!(
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init"]);
    git(dir, &["config", "user.email", "test@example.com"]);
    git(dir, &["config", "user.name", "TestUser"]);
}

// ---------------------------------------------------------------------------
// Orphaned commit is detected with correct fields
// ---------------------------------------------------------------------------

#[test]
fn test_orphan_commit_detected() {
    let dir = temp_repo_dir("detected");
    init_repo(&dir);

    // Commit A — initial commit (will stay reachable)
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    git(&dir, &["add", "."]);
    git_env(
        &dir,
        &["commit", "-m", "initial commit"],
        &[
            ("GIT_AUTHOR_NAME", "TestUser"),
            ("GIT_COMMITTER_NAME", "TestUser"),
        ],
    );
    let hash_a = git(&dir, &["rev-parse", "HEAD"]);

    // Commit B — this will become the orphan
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    git(&dir, &["add", "."]);
    git_env(
        &dir,
        &["commit", "-m", "orphan message"],
        &[
            ("GIT_AUTHOR_NAME", "TestUser"),
            ("GIT_COMMITTER_NAME", "TestUser"),
        ],
    );
    let hash_b = git(&dir, &["rev-parse", "HEAD"]);

    // Reset back to A — B is now unreachable
    git(&dir, &["reset", "--hard", &hash_a]);

    let results = detect_orphan_commits(&dir).unwrap();

    // B must appear in results
    assert!(
        results
            .iter()
            .any(|c| c.commit_hash.to_lowercase() == hash_b.to_lowercase()),
        "orphaned commit B ({}) should be in results; got: {:?}",
        hash_b,
        results.iter().map(|c| &c.commit_hash).collect::<Vec<_>>()
    );

    // Verify fields on the orphaned commit
    let orphan = results
        .iter()
        .find(|c| c.commit_hash.to_lowercase() == hash_b.to_lowercase())
        .unwrap();

    assert_eq!(orphan.author, "TestUser");
    assert_eq!(orphan.message_summary, "orphan message");
    assert!(orphan.timestamp > 0, "timestamp must be positive");
    assert_eq!(
        orphan.commit_hash.len(),
        40,
        "commit_hash must be 40 hex chars"
    );
}

// ---------------------------------------------------------------------------
// Reachable commit (HEAD) is NOT included
// ---------------------------------------------------------------------------

#[test]
fn test_reachable_commit_excluded() {
    let dir = temp_repo_dir("reachable");
    init_repo(&dir);

    std::fs::write(dir.join("a.txt"), "a").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "reachable"]);
    let hash_head = git(&dir, &["rev-parse", "HEAD"]);

    // Make one orphan so the function has something to parse
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "orphan"]);
    let _hash_b = git(&dir, &["rev-parse", "HEAD"]);
    git(&dir, &["reset", "--hard", &hash_head]);

    let results = detect_orphan_commits(&dir).unwrap();

    assert!(
        !results
            .iter()
            .any(|c| c.commit_hash.to_lowercase() == hash_head.to_lowercase()),
        "reachable HEAD commit must not appear in orphan results"
    );
}

// ---------------------------------------------------------------------------
// No orphans → returns Ok(empty vec)
// ---------------------------------------------------------------------------

#[test]
fn test_no_orphans_returns_empty() {
    let dir = temp_repo_dir("empty");
    init_repo(&dir);

    std::fs::write(dir.join("f.txt"), "hello").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "first"]);

    let results = detect_orphan_commits(&dir).unwrap();

    assert!(
        !results
            .iter()
            .any(|c| c.commit_hash == git(&dir, &["rev-parse", "HEAD"])),
        "reachable commit must not appear"
    );
}

// ---------------------------------------------------------------------------
// Non-git directory returns Err
// ---------------------------------------------------------------------------

#[test]
fn test_non_git_dir_returns_err() {
    let dir = temp_repo_dir("nongit");
    std::fs::create_dir_all(&dir).unwrap();

    let result = detect_orphan_commits(&dir);
    assert!(result.is_err(), "expected Err for non-git directory");
}

// ---------------------------------------------------------------------------
// Nonexistent path returns Err
// ---------------------------------------------------------------------------

#[test]
fn test_nonexistent_path_returns_err() {
    let path = temp_repo_dir("nonexistent"); // never created
    let result = detect_orphan_commits(&path);
    assert!(result.is_err(), "expected Err for nonexistent path");
    let msg = result.unwrap_err().to_string();
    assert!(!msg.is_empty(), "error message should be non-empty");
}

// ---------------------------------------------------------------------------
// Multiple orphans are all returned
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_orphans_all_returned() {
    let dir = temp_repo_dir("multi");
    init_repo(&dir);

    // Commit A (initial, stays reachable)
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "initial"]);
    let hash_a = git(&dir, &["rev-parse", "HEAD"]);

    // Commit B
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "orphan B"]);
    let hash_b = git(&dir, &["rev-parse", "HEAD"]);

    // Commit C
    std::fs::write(dir.join("c.txt"), "c").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "orphan C"]);
    let hash_c = git(&dir, &["rev-parse", "HEAD"]);

    // Reset to A — both B and C are now unreachable
    git(&dir, &["reset", "--hard", &hash_a]);

    let results = detect_orphan_commits(&dir).unwrap();

    assert!(
        results
            .iter()
            .any(|c| c.commit_hash.to_lowercase() == hash_b.to_lowercase()),
        "orphan B ({}) must be in results",
        hash_b
    );
    assert!(
        results
            .iter()
            .any(|c| c.commit_hash.to_lowercase() == hash_c.to_lowercase()),
        "orphan C ({}) must be in results",
        hash_c
    );
}

// ---------------------------------------------------------------------------
// commit_hash is exactly 40 lowercase hex characters
// ---------------------------------------------------------------------------

#[test]
fn test_commit_hash_is_40_hex_lowercase() {
    let dir = temp_repo_dir("hashfmt");
    init_repo(&dir);

    std::fs::write(dir.join("a.txt"), "a").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "first"]);
    let hash_a = git(&dir, &["rev-parse", "HEAD"]);

    std::fs::write(dir.join("b.txt"), "b").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "second"]);
    git(&dir, &["reset", "--hard", &hash_a]);

    let results = detect_orphan_commits(&dir).unwrap();

    for orphan in &results {
        assert_eq!(
            orphan.commit_hash.len(),
            40,
            "commit_hash '{}' must be 40 chars",
            orphan.commit_hash
        );
        assert!(
            orphan.commit_hash.chars().all(|c| c.is_ascii_hexdigit()),
            "commit_hash '{}' must be hex",
            orphan.commit_hash
        );
        // git2's Oid::to_string() is always lowercase
        assert_eq!(
            orphan.commit_hash,
            orphan.commit_hash.to_lowercase(),
            "commit_hash must be lowercase"
        );
    }
}

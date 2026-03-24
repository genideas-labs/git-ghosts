use git_ghosts::detect_zombie_branches;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_repo_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gg-zb-{}-{}", label, nanos))
}

fn git(dir: &Path, args: &[&str]) {
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
}

fn git_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) {
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
}

fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init"]);
    git(dir, &["config", "user.email", "test@example.com"]);
    git(dir, &["config", "user.name", "TestUser"]);
}

// ---------------------------------------------------------------------------
// Stale branch detected, fresh branch excluded
// ---------------------------------------------------------------------------

#[test]
fn test_stale_branch_detected_fresh_excluded() {
    let dir = temp_repo_dir("stale");
    init_repo(&dir);

    // Create an initial commit on the default branch (fresh).
    std::fs::write(dir.join("readme.txt"), "init").unwrap();
    git(&dir, &["add", "."]);
    // Use a very recent date for the initial commit.
    let fresh_date = "2099-01-01T00:00:00+00:00";
    git_env(
        &dir,
        &["commit", "-m", "initial"],
        &[
            ("GIT_AUTHOR_DATE", fresh_date),
            ("GIT_COMMITTER_DATE", fresh_date),
        ],
    );

    // Create a stale branch with a commit dated far in the past.
    git(&dir, &["checkout", "-b", "old-feature"]);
    std::fs::write(dir.join("feature.txt"), "old work").unwrap();
    git(&dir, &["add", "."]);
    let old_date = "2020-01-01T00:00:00+00:00";
    git_env(
        &dir,
        &["commit", "-m", "old feature work"],
        &[
            ("GIT_AUTHOR_DATE", old_date),
            ("GIT_COMMITTER_DATE", old_date),
        ],
    );

    // Switch back to the default branch.
    git(&dir, &["checkout", "-"]);

    let zombies = detect_zombie_branches(&dir, None).unwrap();

    // Only old-feature should be a zombie (default branch has a 2099 commit).
    let names: Vec<&str> = zombies.iter().map(|z| z.branch_name.as_str()).collect();
    assert!(
        names.contains(&"old-feature"),
        "old-feature should be a zombie, got: {:?}",
        names
    );

    // Verify fields are populated correctly.
    let zombie = zombies
        .iter()
        .find(|z| z.branch_name == "old-feature")
        .unwrap();
    assert!(!zombie.last_commit_hash.is_empty());
    assert_eq!(zombie.last_commit_author, "TestUser");
    assert!(
        zombie.age_days >= 30,
        "age_days should be >= 30, got {}",
        zombie.age_days
    );
    assert!(zombie.last_commit_timestamp > 0);

    // Default branch must NOT be a zombie (fresh commit in 2099).
    for z in &zombies {
        assert_ne!(
            z.branch_name, "master",
            "fresh branch must not appear in zombies"
        );
        assert_ne!(
            z.branch_name, "main",
            "fresh branch must not appear in zombies"
        );
    }
}

// ---------------------------------------------------------------------------
// threshold_days defaults to 30 when None is passed
// ---------------------------------------------------------------------------

#[test]
fn test_none_threshold_defaults_to_30() {
    let dir = temp_repo_dir("default30");
    init_repo(&dir);

    std::fs::write(dir.join("f.txt"), "x").unwrap();
    git(&dir, &["add", "."]);
    // Commit dated 40 days ago (well over 30-day default).
    let old_date = "2020-06-01T00:00:00+00:00";
    git_env(
        &dir,
        &["commit", "-m", "old"],
        &[
            ("GIT_AUTHOR_DATE", old_date),
            ("GIT_COMMITTER_DATE", old_date),
        ],
    );

    let zombies = detect_zombie_branches(&dir, None).unwrap();
    assert!(
        !zombies.is_empty(),
        "should detect zombie with default 30-day threshold"
    );
}

// ---------------------------------------------------------------------------
// Non-positive threshold returns Err
// ---------------------------------------------------------------------------

#[test]
fn test_zero_threshold_returns_err() {
    let dir = temp_repo_dir("zerothresh");
    init_repo(&dir);

    let result = detect_zombie_branches(&dir, Some(0));
    assert!(result.is_err(), "expected Err for threshold_days=0");
    let msg = result.unwrap_err().to_string();
    assert!(!msg.is_empty(), "error message should be non-empty");
}

#[test]
fn test_negative_threshold_returns_err() {
    let dir = temp_repo_dir("negthresh");
    init_repo(&dir);

    let result = detect_zombie_branches(&dir, Some(-5));
    assert!(result.is_err(), "expected Err for threshold_days=-5");
}

// ---------------------------------------------------------------------------
// Non-git directory returns Err
// ---------------------------------------------------------------------------

#[test]
fn test_non_git_dir_returns_err() {
    let dir = temp_repo_dir("nongit");
    std::fs::create_dir_all(&dir).unwrap();

    let result = detect_zombie_branches(&dir, None);
    assert!(result.is_err(), "expected Err for non-git directory");
}

// ---------------------------------------------------------------------------
// age_days is calculated correctly
// ---------------------------------------------------------------------------

#[test]
fn test_age_days_reflects_commit_age() {
    let dir = temp_repo_dir("agedays");
    init_repo(&dir);

    std::fs::write(dir.join("x.txt"), "x").unwrap();
    git(&dir, &["add", "."]);
    // Commit from exactly 365 days ago (year 2024 in fixed past).
    let old_date = "2020-03-01T00:00:00+00:00";
    git_env(
        &dir,
        &["commit", "-m", "old"],
        &[
            ("GIT_AUTHOR_DATE", old_date),
            ("GIT_COMMITTER_DATE", old_date),
        ],
    );

    let zombies = detect_zombie_branches(&dir, Some(30)).unwrap();
    assert!(!zombies.is_empty());
    // The commit is from ~2020, so age_days should be well over 365.
    assert!(
        zombies[0].age_days > 365,
        "expected age > 365 days for a 2020 commit, got {}",
        zombies[0].age_days
    );
}

// ---------------------------------------------------------------------------
// i64::MAX threshold must not overflow or panic
// ---------------------------------------------------------------------------

#[test]
fn test_i64_max_threshold_no_overflow() {
    let dir = temp_repo_dir("maxthresh");
    init_repo(&dir);

    std::fs::write(dir.join("f.txt"), "x").unwrap();
    git(&dir, &["add", "."]);
    let old_date = "2020-01-01T00:00:00+00:00";
    git_env(
        &dir,
        &["commit", "-m", "old"],
        &[
            ("GIT_AUTHOR_DATE", old_date),
            ("GIT_COMMITTER_DATE", old_date),
        ],
    );

    // i64::MAX days should not overflow; no branch can be that old.
    let result = detect_zombie_branches(&dir, Some(i64::MAX));
    assert!(
        result.is_ok(),
        "i64::MAX threshold must not error, got: {:?}",
        result.err()
    );
    assert!(
        result.unwrap().is_empty(),
        "no branch can be i64::MAX days old"
    );
}

// ---------------------------------------------------------------------------
// i64::MIN threshold returns Err (negative value)
// ---------------------------------------------------------------------------

#[test]
fn test_i64_min_threshold_returns_err() {
    let dir = temp_repo_dir("minthresh");
    init_repo(&dir);

    let result = detect_zombie_branches(&dir, Some(i64::MIN));
    assert!(result.is_err(), "expected Err for threshold_days=i64::MIN");
    let msg = result.unwrap_err().to_string();
    assert!(!msg.is_empty(), "error message should be non-empty");
}

// ---------------------------------------------------------------------------
// Non-existent path returns Err
// ---------------------------------------------------------------------------

#[test]
fn test_nonexistent_path_returns_err() {
    let path = temp_repo_dir("nonexistent"); // never created
    let result = detect_zombie_branches(&path, None);
    assert!(result.is_err(), "expected Err for nonexistent path");
    let msg = result.unwrap_err().to_string();
    assert!(!msg.is_empty(), "error message should be non-empty");
}

// ---------------------------------------------------------------------------
// Empty repo (no commits) returns Ok(vec![])
// ---------------------------------------------------------------------------

#[test]
fn test_empty_repo_returns_ok_empty() {
    let dir = temp_repo_dir("emptyrepo");
    init_repo(&dir);
    // No commits — no branches exist.
    let result = detect_zombie_branches(&dir, None);
    assert!(
        result.is_ok(),
        "empty repo should return Ok, got: {:?}",
        result.err()
    );
    assert!(
        result.unwrap().is_empty(),
        "empty repo should return empty vec"
    );
}

// ---------------------------------------------------------------------------
// Boundary: commit exactly at threshold + 120 s IS included
// ---------------------------------------------------------------------------

#[test]
fn test_boundary_commit_is_included() {
    let dir = temp_repo_dir("boundary-in");
    init_repo(&dir);

    let threshold_days: i64 = 30;
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    // 120-second buffer beyond threshold to tolerate test-runner latency.
    let commit_ts = now_secs - (threshold_days * 86_400 + 120);
    let date_str = format!("@{}", commit_ts);

    std::fs::write(dir.join("b.txt"), "boundary").unwrap();
    git(&dir, &["add", "."]);
    git_env(
        &dir,
        &["commit", "-m", "boundary commit"],
        &[
            ("GIT_AUTHOR_DATE", &date_str),
            ("GIT_COMMITTER_DATE", &date_str),
        ],
    );

    let zombies = detect_zombie_branches(&dir, Some(threshold_days)).unwrap();
    assert!(
        !zombies.is_empty(),
        "commit at threshold+120s should be included as zombie"
    );
}

// ---------------------------------------------------------------------------
// Fresh commit (1 hour inside threshold) is NOT included
// ---------------------------------------------------------------------------

#[test]
fn test_fresh_commit_below_boundary_excluded() {
    let dir = temp_repo_dir("boundary-out");
    init_repo(&dir);

    let threshold_days: i64 = 30;
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    // 1 hour (3600 s) inside the threshold window → should NOT be zombie.
    let commit_ts = now_secs - (threshold_days * 86_400 - 3600);
    let date_str = format!("@{}", commit_ts);

    std::fs::write(dir.join("fresh.txt"), "fresh").unwrap();
    git(&dir, &["add", "."]);
    git_env(
        &dir,
        &["commit", "-m", "fresh commit"],
        &[
            ("GIT_AUTHOR_DATE", &date_str),
            ("GIT_COMMITTER_DATE", &date_str),
        ],
    );

    let zombies = detect_zombie_branches(&dir, Some(threshold_days)).unwrap();
    assert!(
        zombies.is_empty(),
        "commit 1 hour inside threshold must not be a zombie, got: {:?}",
        zombies.iter().map(|z| &z.branch_name).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Branch name containing slashes is correctly enumerated
// ---------------------------------------------------------------------------

#[test]
fn test_slashed_branch_name() {
    let dir = temp_repo_dir("slashedbranch");
    init_repo(&dir);

    // Create an initial commit on the default branch.
    std::fs::write(dir.join("init.txt"), "init").unwrap();
    git(&dir, &["add", "."]);
    let old_date = "2020-01-01T00:00:00+00:00";
    git_env(
        &dir,
        &["commit", "-m", "initial"],
        &[
            ("GIT_AUTHOR_DATE", old_date),
            ("GIT_COMMITTER_DATE", old_date),
        ],
    );

    // Create a nested branch name.
    git(&dir, &["checkout", "-b", "feature/foo/bar"]);
    std::fs::write(dir.join("x.txt"), "x").unwrap();
    git(&dir, &["add", "."]);
    git_env(
        &dir,
        &["commit", "-m", "feature work"],
        &[
            ("GIT_AUTHOR_DATE", old_date),
            ("GIT_COMMITTER_DATE", old_date),
        ],
    );
    git(&dir, &["checkout", "-"]);

    let zombies = detect_zombie_branches(&dir, Some(30)).unwrap();
    let names: Vec<&str> = zombies.iter().map(|z| z.branch_name.as_str()).collect();
    assert!(
        names.contains(&"feature/foo/bar"),
        "slashed branch name should be detected as zombie, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Future-dated commit (year 2099) does not panic and is excluded
// ---------------------------------------------------------------------------

#[test]
fn test_future_timestamp_no_panic() {
    let dir = temp_repo_dir("future");
    init_repo(&dir);

    std::fs::write(dir.join("f.txt"), "future").unwrap();
    git(&dir, &["add", "."]);
    let future_date = "2099-12-31T00:00:00+00:00";
    git_env(
        &dir,
        &["commit", "-m", "future commit"],
        &[
            ("GIT_AUTHOR_DATE", future_date),
            ("GIT_COMMITTER_DATE", future_date),
        ],
    );

    // Must not panic; future commit has negative age → not a zombie.
    let result = detect_zombie_branches(&dir, Some(30));
    assert!(
        result.is_ok(),
        "future-dated commit must not cause an error"
    );
    assert!(
        result.unwrap().is_empty(),
        "future-dated commit must not be classified as zombie"
    );
}

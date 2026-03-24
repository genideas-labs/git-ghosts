/// tests/test_clean_cmd.rs
///
/// Integration tests for the `git-ghosts clean --dry-run` subcommand handler.
/// All tests call `format_clean_dry_run` or `run_clean_dry_run` directly so no
/// CLI argument parsing is needed.
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use git_ghosts::cli::run_clean_dry_run;
use git_ghosts::{
    format_clean_dry_run, save_cache, GhostFile, OrphanCommit, ScanResults, ZombieBranch,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gg-clean-{}-{}", label, nanos))
}

/// Create a temp directory that has a `.git/` subdirectory (no real repo
/// needed for cache-only tests).
fn make_tempdir_with_git() -> PathBuf {
    let dir = temp_dir("witgit");
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    dir
}

/// Build a `ScanResults` fixture with the requested number of each item.
fn make_fixture_results(
    ghost_count: usize,
    zombie_count: usize,
    orphan_count: usize,
) -> ScanResults {
    let ghost_files = (0..ghost_count)
        .map(|i| GhostFile {
            file_path: format!("src/deleted_{i}.rs"),
            deletion_commit_hash: format!("{i:040x}"),
            author: "Alice".to_string(),
            timestamp: 1_700_000_000 + i as i64,
            original_file_size_bytes: 512 + i as u64,
        })
        .collect();

    let zombie_branches = (0..zombie_count)
        .map(|i| ZombieBranch {
            branch_name: format!("feature/old-{i}"),
            last_commit_hash: format!("{i:040x}"),
            last_commit_author: "Bob".to_string(),
            last_commit_timestamp: 1_600_000_000 + i as i64,
            age_days: 90 + i as u64,
        })
        .collect();

    let orphan_commits = (0..orphan_count)
        .map(|i| OrphanCommit {
            commit_hash: format!("{i:040x}"),
            author: "Carol".to_string(),
            timestamp: 1_500_000_000 + i as i64,
            message_summary: format!("stray work {i}"),
        })
        .collect();

    ScanResults {
        ghost_files,
        zombie_branches,
        orphan_commits,
    }
}

/// Strips ANSI escape sequences so assertions are colour-agnostic.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for ch in chars.by_ref() {
                if ch == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// `format_clean_dry_run` with 2 zombies + 1 orphan produces three expected
/// action lines.
#[test]
fn clean_dry_run_output_matches_fixture() {
    let dir = make_tempdir_with_git();
    save_cache(&dir, &make_fixture_results(0, 2, 1)).unwrap();

    // Load through the cache so the test exercises the same round-trip path.
    use git_ghosts::load_cache;
    let results = load_cache(&dir).expect("cache should be loadable");

    let plain = strip_ansi(&format_clean_dry_run(&results));

    assert!(
        plain.contains("would delete branch: feature/old-0"),
        "missing first zombie line; got:\n{}",
        plain
    );
    assert!(
        plain.contains("would delete branch: feature/old-1"),
        "missing second zombie line; got:\n{}",
        plain
    );
    assert!(
        plain.contains("would remove orphan commit: 0000000000000000000000000000000000000000"),
        "missing orphan line; got:\n{}",
        plain
    );
}

/// `run_clean_dry_run` without a cache file returns an `Err` containing the
/// canonical "no cache" message.
#[test]
fn clean_dry_run_no_cache_returns_error() {
    let dir = make_tempdir_with_git(); // no cache written

    let err = run_clean_dry_run(&dir).unwrap_err();
    assert!(
        err.contains("No scan cache found. Run git-ghosts scan first."),
        "unexpected error message: {}",
        err
    );
}

/// `format_clean_dry_run` on an all-empty `ScanResults` returns an empty string.
#[test]
fn clean_dry_run_empty_results_no_output() {
    let results = make_fixture_results(0, 0, 0);
    let output = format_clean_dry_run(&results);
    assert!(
        output.is_empty(),
        "expected empty output for empty results, got: {:?}",
        output
    );
}

/// `format_clean_dry_run` with only ghost files produces no output (ghost files
/// are excluded from the clean action).
#[test]
fn clean_dry_run_ghost_files_not_in_output() {
    let results = make_fixture_results(3, 0, 0);
    let output = strip_ansi(&format_clean_dry_run(&results));
    assert!(
        output.is_empty(),
        "expected empty output when only ghost_files present, got: {:?}",
        output
    );
}

/// All zombie-branch lines appear before all orphan-commit lines in the output.
#[test]
fn clean_dry_run_zombie_lines_before_orphan_lines() {
    let results = make_fixture_results(0, 1, 1);
    let plain = strip_ansi(&format_clean_dry_run(&results));
    let lines: Vec<&str> = plain.lines().collect();

    let branch_pos = lines
        .iter()
        .position(|l| l.contains("would delete branch:"))
        .expect("zombie line not found");
    let orphan_pos = lines
        .iter()
        .position(|l| l.contains("would remove orphan commit:"))
        .expect("orphan line not found");

    assert!(
        branch_pos < orphan_pos,
        "zombie line ({}) should appear before orphan line ({})",
        branch_pos,
        orphan_pos
    );
}

/// `run_clean_dry_run` returns `Ok(())` when a valid cache is present.
#[test]
fn clean_dry_run_loads_cache_ok() {
    let dir = make_tempdir_with_git();
    save_cache(&dir, &make_fixture_results(1, 1, 1)).unwrap();

    let result = run_clean_dry_run(&dir);
    assert_eq!(result, Ok(()), "expected Ok(()), got: {:?}", result);
}

/// Branch names with special characters (e.g. slashes and spaces) appear
/// verbatim in the output.
#[test]
fn clean_dry_run_special_branch_name() {
    let results = ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![ZombieBranch {
            branch_name: "refs/heads/fix/foo bar".to_string(),
            last_commit_hash: "a".repeat(40),
            last_commit_author: "Dev".to_string(),
            last_commit_timestamp: 1_600_000_000,
            age_days: 100,
        }],
        orphan_commits: vec![],
    };

    let plain = strip_ansi(&format_clean_dry_run(&results));
    assert!(
        plain.contains("would delete branch: refs/heads/fix/foo bar"),
        "special branch name not found verbatim; got:\n{}",
        plain
    );
}

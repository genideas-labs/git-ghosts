/// tests/test_report_cmd.rs
///
/// Integration tests for the `git-ghosts report` subcommand handler.
/// All tests call `run_report` directly so no CLI argument parsing is needed.
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use git_ghosts::cli::run_report;
use git_ghosts::{
    format_report, load_cache, save_cache, GhostFile, OrphanCommit, ScanResults, ZombieBranch,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gg-report-{}-{}", label, nanos))
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

/// Strips ANSI escape sequences so count/label assertions are colour-agnostic.
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

/// `run_report` on a directory with a valid cache succeeds and returns `Ok(())`.
#[test]
fn report_loads_cache_and_renders() {
    let dir = make_tempdir_with_git();
    save_cache(&dir, &make_fixture_results(1, 1, 1)).unwrap();

    let result = run_report(&dir);
    assert_eq!(result, Ok(()), "expected Ok(()), got: {:?}", result);
}

/// `run_report` without a cache file returns an `Err` containing the canonical
/// "no cache" message.
#[test]
fn report_no_cache_returns_error_message() {
    let dir = make_tempdir_with_git(); // no cache written

    let err = run_report(&dir).unwrap_err();
    assert!(
        err.contains("No scan cache found. Run git-ghosts scan first."),
        "unexpected error message: {}",
        err
    );
}

/// After saving a cache with 2 ghost files, 1 zombie branch, and 1 orphan
/// commit, `format_report` output (ANSI-stripped) shows those exact counts
/// adjacent to the correct category labels.
#[test]
fn report_output_contains_correct_counts_from_cache() {
    let dir = make_tempdir_with_git();
    save_cache(&dir, &make_fixture_results(2, 1, 1)).unwrap();

    let results = load_cache(&dir).expect("cache should be loadable");
    let plain = strip_ansi(&format_report(&results));
    let lines: Vec<&str> = plain.lines().collect();

    let ghost_line = lines
        .iter()
        .find(|l| l.contains("Ghost Files"))
        .expect("Ghost Files line missing from report");
    let zombie_line = lines
        .iter()
        .find(|l| l.contains("Zombie Branches"))
        .expect("Zombie Branches line missing from report");
    let orphan_line = lines
        .iter()
        .find(|l| l.contains("Orphan Commits"))
        .expect("Orphan Commits line missing from report");

    assert!(
        ghost_line.contains('2'),
        "expected ghost count 2 in line: {ghost_line}"
    );
    assert!(
        zombie_line.contains('1'),
        "expected zombie count 1 in line: {zombie_line}"
    );
    assert!(
        orphan_line.contains('1'),
        "expected orphan count 1 in line: {orphan_line}"
    );
}

/// The error message returned when no cache exists must not mention any
/// detector internals — confirming no detector was called.
#[test]
fn report_error_message_does_not_mention_detection() {
    let dir = make_tempdir_with_git(); // no cache

    let err = run_report(&dir).unwrap_err();
    let lower = err.to_lowercase();

    for forbidden in ["detect", "ghost_files", "zombie_branches", "orphan_commits"] {
        assert!(
            !lower.contains(forbidden),
            "error message must not mention '{}', got: {}",
            forbidden,
            err
        );
    }
}

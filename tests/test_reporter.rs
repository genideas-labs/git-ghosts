/// tests/test_reporter.rs
///
/// Integration tests for the reporter / report module.  All behavioural
/// assertions target `format_report` (returns `String`) rather than
/// `render_report` (stdout side-effect) so that tests remain deterministic
/// and environment-agnostic.
use git_ghosts::report::{format_report, render_report};
use git_ghosts::{GhostFile, OrphanCommit, ScanResults, ZombieBranch};

// ── fixture ──────────────────────────────────────────────────────────────────

fn make_results() -> ScanResults {
    ScanResults {
        ghost_files: vec![GhostFile {
            file_path: "src/old.rs".to_string(),
            deletion_commit_hash: "abc123".to_string(),
            author: "Alice".to_string(),
            timestamp: 1_700_000_000,
            original_file_size_bytes: 1024,
        }],
        zombie_branches: vec![ZombieBranch {
            branch_name: "feature/old".to_string(),
            last_commit_hash: "dead0001".to_string(),
            last_commit_author: "Bob".to_string(),
            last_commit_timestamp: 1_600_000_000,
            age_days: 180,
        }],
        orphan_commits: vec![OrphanCommit {
            commit_hash: "orphanff".to_string(),
            author: "Carol".to_string(),
            timestamp: 1_500_000_000,
            message_summary: "stray work".to_string(),
        }],
    }
}

fn empty_results() -> ScanResults {
    ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![],
        orphan_commits: vec![],
    }
}

/// Strips ANSI escape sequences (`ESC[...m`) so assertions are independent of
/// whether the `colored` crate emits colour codes in the test environment.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // consume everything up to and including the terminating 'm'
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

// ── label-presence tests ──────────────────────────────────────────────────────

#[test]
fn reporter_contains_all_category_labels() {
    let results = make_results();
    let raw = format_report(&results);
    let plain = strip_ansi(&raw);
    assert!(
        plain.contains("Ghost Files"),
        "missing 'Ghost Files': {plain}"
    );
    assert!(
        plain.contains("Zombie Branches"),
        "missing 'Zombie Branches': {plain}"
    );
    assert!(
        plain.contains("Orphan Commits"),
        "missing 'Orphan Commits': {plain}"
    );
}

#[test]
fn reporter_labels_are_non_empty_human_readable_strings() {
    // After stripping ANSI the three category names must be non-empty and
    // contain only printable characters (no raw escape bytes left over).
    let results = make_results();
    let plain = strip_ansi(&format_report(&results));
    for label in ["Ghost Files", "Zombie Branches", "Orphan Commits"] {
        assert!(!label.is_empty());
        assert!(plain.contains(label), "label '{label}' not found in output");
        assert!(
            label.chars().all(|c| !c.is_control()),
            "label '{label}' contains control characters"
        );
    }
}

// ── count-value tests ─────────────────────────────────────────────────────────

#[test]
fn reporter_shows_correct_counts_for_one_each() {
    let results = make_results();
    let plain = strip_ansi(&format_report(&results));
    let lines: Vec<&str> = plain.lines().collect();

    let ghost_line = lines
        .iter()
        .find(|l| l.contains("Ghost Files"))
        .expect("Ghost Files line missing");
    let zombie_line = lines
        .iter()
        .find(|l| l.contains("Zombie Branches"))
        .expect("Zombie Branches line missing");
    let orphan_line = lines
        .iter()
        .find(|l| l.contains("Orphan Commits"))
        .expect("Orphan Commits line missing");

    assert!(ghost_line.contains('1'), "ghost count wrong: {ghost_line}");
    assert!(
        zombie_line.contains('1'),
        "zombie count wrong: {zombie_line}"
    );
    assert!(
        orphan_line.contains('1'),
        "orphan count wrong: {orphan_line}"
    );
}

#[test]
fn reporter_zero_counts_on_empty_results() {
    let results = empty_results();
    let plain = strip_ansi(&format_report(&results));
    let lines: Vec<&str> = plain.lines().collect();

    // All three labels must still appear
    assert!(plain.contains("Ghost Files"));
    assert!(plain.contains("Zombie Branches"));
    assert!(plain.contains("Orphan Commits"));

    // Each row must show '0'
    let ghost_line = lines.iter().find(|l| l.contains("Ghost Files")).unwrap();
    let zombie_line = lines
        .iter()
        .find(|l| l.contains("Zombie Branches"))
        .unwrap();
    let orphan_line = lines.iter().find(|l| l.contains("Orphan Commits")).unwrap();

    assert!(
        ghost_line.contains('0'),
        "empty ghost count should be 0: {ghost_line}"
    );
    assert!(
        zombie_line.contains('0'),
        "empty zombie count should be 0: {zombie_line}"
    );
    assert!(
        orphan_line.contains('0'),
        "empty orphan count should be 0: {orphan_line}"
    );
}

#[test]
fn reporter_large_counts_do_not_panic() {
    // Build a ScanResults with many entries to exercise large count formatting.
    // We use a moderate large number (100_000) rather than usize::MAX to avoid
    // allocating an unreasonable amount of memory in CI.
    let n = 100_000_usize;
    let results = ScanResults {
        ghost_files: (0..n)
            .map(|i| GhostFile {
                file_path: format!("file_{i}.rs"),
                deletion_commit_hash: format!("{i:040x}"),
                author: "tester".to_string(),
                timestamp: i as i64,
                original_file_size_bytes: i as u64,
            })
            .collect(),
        zombie_branches: (0..n)
            .map(|i| ZombieBranch {
                branch_name: format!("branch_{i}"),
                last_commit_hash: format!("{i:040x}"),
                last_commit_author: "tester".to_string(),
                last_commit_timestamp: i as i64,
                age_days: i as u64,
            })
            .collect(),
        orphan_commits: (0..n)
            .map(|i| OrphanCommit {
                commit_hash: format!("{i:040x}"),
                author: "tester".to_string(),
                timestamp: i as i64,
                message_summary: format!("msg {i}"),
            })
            .collect(),
    };
    // Must not panic; we only assert the output is non-empty.
    let output = format_report(&results);
    assert!(
        !output.is_empty(),
        "format_report returned empty string for large input"
    );
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();
    let ghost_line = lines.iter().find(|l| l.contains("Ghost Files")).unwrap();
    assert!(
        ghost_line.contains(&n.to_string()),
        "expected count {n} in ghost line: {ghost_line}"
    );
}

// ── render (smoke) test ───────────────────────────────────────────────────────

#[test]
fn render_report_does_not_panic() {
    // Smoke-test: calling render_report must not panic for any valid input.
    render_report(&make_results());
    render_report(&empty_results());
}

// ── reporter shim re-export test ──────────────────────────────────────────────

#[test]
fn reporter_module_re_exports_format_and_render() {
    // git_ghosts::reporter re-exports the same functions; calling them must
    // produce the same output as calling them via git_ghosts::report.
    use git_ghosts::reporter::{format_report as fmt2, render_report as render2};
    let results = make_results();
    let via_report = format_report(&results);
    let via_reporter = fmt2(&results);
    assert_eq!(
        via_report, via_reporter,
        "reporter re-export must produce identical output to report module"
    );
    // smoke-test the render path through the shim
    render2(&results);
}

// ── top-level re-export test ──────────────────────────────────────────────────

#[test]
fn top_level_reexports_are_accessible() {
    // The acceptance criteria requires `pub use report::{format_report, render_report}`
    // to be available at the crate root.
    use git_ghosts::{format_report as top_fmt, render_report as top_render};
    let results = make_results();
    let output = top_fmt(&results);
    let plain = strip_ansi(&output);
    assert!(plain.contains("Ghost Files"));
    top_render(&results); // must not panic
}

use git_ghosts::report::format_report;
use git_ghosts::{GhostFile, OrphanCommit, ScanResults, ZombieBranch};

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

#[test]
fn report_contains_all_category_labels() {
    let results = make_results();
    let raw = format_report(&results);
    let plain = strip_ansi(&raw);
    assert!(
        plain.contains("Ghost Files"),
        "missing Ghost Files: {plain}"
    );
    assert!(
        plain.contains("Zombie Branches"),
        "missing Zombie Branches: {plain}"
    );
    assert!(
        plain.contains("Orphan Commits"),
        "missing Orphan Commits: {plain}"
    );
}

#[test]
fn report_shows_correct_counts() {
    let results = make_results();
    let raw = format_report(&results);
    let plain = strip_ansi(&raw);
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
fn report_empty_results_contains_labels() {
    let results = ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![],
        orphan_commits: vec![],
    };
    let raw = format_report(&results);
    let plain = strip_ansi(&raw);
    assert!(plain.contains("Ghost Files"));
    assert!(plain.contains("Zombie Branches"));
    assert!(plain.contains("Orphan Commits"));
    let lines: Vec<&str> = plain.lines().collect();
    let ghost_line = lines.iter().find(|l| l.contains("Ghost Files")).unwrap();
    assert!(
        ghost_line.contains('0'),
        "empty ghost count should be 0: {ghost_line}"
    );
}

#[test]
fn render_report_does_not_panic() {
    let results = make_results();
    git_ghosts::report::render_report(&results);
}

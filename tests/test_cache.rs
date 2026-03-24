use git_ghosts::cache::CacheError;
use git_ghosts::models::{GhostFile, OrphanCommit, ScanResults, ZombieBranch};
use git_ghosts::{load_cache, save_cache};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gg-cache-{}-{}", label, nanos))
}

fn init_repo(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .unwrap();
}

fn sample_results() -> ScanResults {
    ScanResults {
        ghost_files: vec![GhostFile {
            file_path: "src/old.rs".into(),
            deletion_commit_hash: "a".repeat(40),
            author: "Alice".into(),
            timestamp: 1_700_000_000,
            original_file_size_bytes: 42,
        }],
        zombie_branches: vec![ZombieBranch {
            branch_name: "feat/dead".into(),
            last_commit_hash: "b".repeat(40),
            last_commit_author: "Bob".into(),
            last_commit_timestamp: 1_600_000_000,
            age_days: 180,
        }],
        orphan_commits: vec![OrphanCommit {
            commit_hash: "c".repeat(40),
            author: "Carol".into(),
            timestamp: 1_500_000_000,
            message_summary: "wip".into(),
        }],
    }
}

/// Round-trip: save then load produces an equal ScanResults.
#[test]
fn test_cache_roundtrip() {
    let dir = temp_dir("roundtrip");
    init_repo(&dir);
    let original = sample_results();
    save_cache(&dir, &original).expect("save_cache failed");
    let loaded = load_cache(&dir).expect("load_cache failed");
    assert_eq!(loaded.ghost_files.len(), original.ghost_files.len());
    assert_eq!(
        loaded.ghost_files[0].file_path,
        original.ghost_files[0].file_path
    );
    assert_eq!(
        loaded.ghost_files[0].deletion_commit_hash,
        original.ghost_files[0].deletion_commit_hash
    );
    assert_eq!(loaded.ghost_files[0].author, original.ghost_files[0].author);
    assert_eq!(
        loaded.ghost_files[0].timestamp,
        original.ghost_files[0].timestamp
    );
    assert_eq!(
        loaded.ghost_files[0].original_file_size_bytes,
        original.ghost_files[0].original_file_size_bytes
    );
    assert_eq!(loaded.zombie_branches.len(), original.zombie_branches.len());
    assert_eq!(
        loaded.zombie_branches[0].branch_name,
        original.zombie_branches[0].branch_name
    );
    assert_eq!(
        loaded.zombie_branches[0].last_commit_hash,
        original.zombie_branches[0].last_commit_hash
    );
    assert_eq!(
        loaded.zombie_branches[0].last_commit_author,
        original.zombie_branches[0].last_commit_author
    );
    assert_eq!(
        loaded.zombie_branches[0].last_commit_timestamp,
        original.zombie_branches[0].last_commit_timestamp
    );
    assert_eq!(
        loaded.zombie_branches[0].age_days,
        original.zombie_branches[0].age_days
    );
    assert_eq!(loaded.orphan_commits.len(), original.orphan_commits.len());
    assert_eq!(
        loaded.orphan_commits[0].commit_hash,
        original.orphan_commits[0].commit_hash
    );
    assert_eq!(
        loaded.orphan_commits[0].author,
        original.orphan_commits[0].author
    );
    assert_eq!(
        loaded.orphan_commits[0].timestamp,
        original.orphan_commits[0].timestamp
    );
    assert_eq!(
        loaded.orphan_commits[0].message_summary,
        original.orphan_commits[0].message_summary
    );
}

/// load_cache returns CacheError::NotFound when no cache file exists.
#[test]
fn test_load_cache_not_found() {
    let dir = temp_dir("notfound");
    init_repo(&dir);
    let result = load_cache(&dir);
    assert!(
        matches!(result, Err(CacheError::NotFound)),
        "expected CacheError::NotFound, got: {:?}",
        result
    );
}

/// The cache file is written inside .git/, not the working tree.
#[test]
fn test_cache_file_is_inside_dot_git() {
    let dir = temp_dir("location");
    init_repo(&dir);
    let results = ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![],
        orphan_commits: vec![],
    };
    save_cache(&dir, &results).expect("save_cache failed");
    let cache_file = dir.join(".git").join("git-ghosts-cache.json");
    assert!(
        cache_file.exists(),
        "cache file must exist at .git/git-ghosts-cache.json"
    );
}

/// Empty ScanResults round-trips correctly.
#[test]
fn test_cache_roundtrip_empty() {
    let dir = temp_dir("empty");
    init_repo(&dir);
    let original = ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![],
        orphan_commits: vec![],
    };
    save_cache(&dir, &original).expect("save_cache failed");
    let loaded = load_cache(&dir).expect("load_cache failed");
    assert!(loaded.ghost_files.is_empty());
    assert!(loaded.zombie_branches.is_empty());
    assert!(loaded.orphan_commits.is_empty());
}

/// Saving again overwrites the previous cache.
#[test]
fn test_cache_overwrite() {
    let dir = temp_dir("overwrite");
    init_repo(&dir);
    save_cache(&dir, &sample_results()).expect("first save failed");
    let second = ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![],
        orphan_commits: vec![],
    };
    save_cache(&dir, &second).expect("second save failed");
    let loaded = load_cache(&dir).expect("load after overwrite failed");
    assert!(
        loaded.ghost_files.is_empty(),
        "cache should reflect second save"
    );
}

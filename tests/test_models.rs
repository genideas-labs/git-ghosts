use git_ghosts::{GhostFile, OrphanCommit, ScanResults, ZombieBranch};

// ── GhostFile ──────────────────────────────────────────────────────────────

#[test]
fn ghost_file_construction_and_field_access() {
    let gf = GhostFile {
        file_path: "src/main.rs".to_string(),
        deletion_commit_hash: "abc123".to_string(),
        author: "Alice".to_string(),
        timestamp: 1_700_000_000,
        original_file_size_bytes: 4096,
    };
    assert_eq!(gf.file_path, "src/main.rs");
    assert_eq!(gf.deletion_commit_hash, "abc123");
    assert_eq!(gf.author, "Alice");
    assert_eq!(gf.timestamp, 1_700_000_000);
    assert_eq!(gf.original_file_size_bytes, 4096);
}

#[test]
fn ghost_file_clone() {
    let gf = GhostFile {
        file_path: "a.rs".to_string(),
        deletion_commit_hash: "d1".to_string(),
        author: "Bob".to_string(),
        timestamp: 0,
        original_file_size_bytes: 1,
    };
    let cloned = gf.clone();
    assert_eq!(gf.file_path, cloned.file_path);
    assert_eq!(gf.deletion_commit_hash, cloned.deletion_commit_hash);
    assert_eq!(gf.author, cloned.author);
    assert_eq!(gf.timestamp, cloned.timestamp);
    assert_eq!(gf.original_file_size_bytes, cloned.original_file_size_bytes);
}

#[test]
fn ghost_file_debug_non_empty() {
    let gf = GhostFile {
        file_path: "x.rs".to_string(),
        deletion_commit_hash: "hash".to_string(),
        author: "Carol".to_string(),
        timestamp: 100,
        original_file_size_bytes: 0,
    };
    assert!(!format!("{gf:?}").is_empty());
}

#[test]
fn ghost_file_serde_roundtrip() {
    let gf = GhostFile {
        file_path: "lib.rs".to_string(),
        deletion_commit_hash: "beef".to_string(),
        author: "Dave".to_string(),
        timestamp: 1_609_459_200,
        original_file_size_bytes: 8192,
    };
    let json = serde_json::to_string(&gf).expect("serialize");
    let back: GhostFile = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(gf.file_path, back.file_path);
    assert_eq!(gf.deletion_commit_hash, back.deletion_commit_hash);
    assert_eq!(gf.author, back.author);
    assert_eq!(gf.timestamp, back.timestamp);
    assert_eq!(gf.original_file_size_bytes, back.original_file_size_bytes);
}

#[test]
fn ghost_file_negative_timestamp() {
    let gf = GhostFile {
        file_path: "old.rs".to_string(),
        deletion_commit_hash: "pre1970".to_string(),
        author: "Epoch".to_string(),
        timestamp: -1,
        original_file_size_bytes: 0,
    };
    assert_eq!(gf.timestamp, -1);
}

#[test]
fn ghost_file_zero_size() {
    let gf = GhostFile {
        file_path: "empty.rs".to_string(),
        deletion_commit_hash: "e0".to_string(),
        author: "Eve".to_string(),
        timestamp: 1,
        original_file_size_bytes: 0,
    };
    assert_eq!(gf.original_file_size_bytes, 0);
}

#[test]
fn ghost_file_unicode_fields() {
    let gf = GhostFile {
        file_path: "src/文件.rs".to_string(),
        deletion_commit_hash: "unicode_hash".to_string(),
        author: "作者".to_string(),
        timestamp: 42,
        original_file_size_bytes: 512,
    };
    assert_eq!(gf.file_path, "src/文件.rs");
    assert_eq!(gf.author, "作者");
}

// ── ZombieBranch ──────────────────────────────────────────────────────────

#[test]
fn zombie_branch_construction_and_field_access() {
    let zb = ZombieBranch {
        branch_name: "feature/old".to_string(),
        last_commit_hash: "dead0000".to_string(),
        last_commit_author: "Frank".to_string(),
        last_commit_timestamp: 1_600_000_000,
        age_days: 365,
    };
    assert_eq!(zb.branch_name, "feature/old");
    assert_eq!(zb.last_commit_hash, "dead0000");
    assert_eq!(zb.last_commit_author, "Frank");
    assert_eq!(zb.last_commit_timestamp, 1_600_000_000);
    assert_eq!(zb.age_days, 365);
}

#[test]
fn zombie_branch_clone() {
    let zb = ZombieBranch {
        branch_name: "old".to_string(),
        last_commit_hash: "h2".to_string(),
        last_commit_author: "Grace".to_string(),
        last_commit_timestamp: 0,
        age_days: 0,
    };
    let cloned = zb.clone();
    assert_eq!(zb.branch_name, cloned.branch_name);
    assert_eq!(zb.age_days, cloned.age_days);
}

#[test]
fn zombie_branch_debug_non_empty() {
    let zb = ZombieBranch {
        branch_name: "b".to_string(),
        last_commit_hash: "h".to_string(),
        last_commit_author: "H".to_string(),
        last_commit_timestamp: 1,
        age_days: 1,
    };
    assert!(!format!("{zb:?}").is_empty());
}

#[test]
fn zombie_branch_serde_roundtrip() {
    let zb = ZombieBranch {
        branch_name: "release/v1".to_string(),
        last_commit_hash: "cafe".to_string(),
        last_commit_author: "Ivan".to_string(),
        last_commit_timestamp: 1_700_000_000,
        age_days: 90,
    };
    let json = serde_json::to_string(&zb).expect("serialize");
    let back: ZombieBranch = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(zb.branch_name, back.branch_name);
    assert_eq!(zb.last_commit_hash, back.last_commit_hash);
    assert_eq!(zb.last_commit_author, back.last_commit_author);
    assert_eq!(zb.last_commit_timestamp, back.last_commit_timestamp);
    assert_eq!(zb.age_days, back.age_days);
}

#[test]
fn zombie_branch_slash_in_name() {
    let zb = ZombieBranch {
        branch_name: "feature/sub/topic".to_string(),
        last_commit_hash: "abcd".to_string(),
        last_commit_author: "Judy".to_string(),
        last_commit_timestamp: 0,
        age_days: 7,
    };
    assert_eq!(zb.branch_name, "feature/sub/topic");
}

#[test]
fn zombie_branch_zero_age() {
    let zb = ZombieBranch {
        branch_name: "fresh".to_string(),
        last_commit_hash: "new".to_string(),
        last_commit_author: "Kate".to_string(),
        last_commit_timestamp: 0,
        age_days: 0,
    };
    assert_eq!(zb.age_days, 0);
}

#[test]
fn zombie_branch_negative_timestamp() {
    let zb = ZombieBranch {
        branch_name: "ancient".to_string(),
        last_commit_hash: "old".to_string(),
        last_commit_author: "Leo".to_string(),
        last_commit_timestamp: -86400,
        age_days: 1,
    };
    assert_eq!(zb.last_commit_timestamp, -86400);
}

// ── OrphanCommit ──────────────────────────────────────────────────────────

#[test]
fn orphan_commit_construction_and_field_access() {
    let oc = OrphanCommit {
        commit_hash: "orphan42".to_string(),
        author: "Mallory".to_string(),
        timestamp: 1_500_000_000,
        message_summary: "Fix a bug".to_string(),
    };
    assert_eq!(oc.commit_hash, "orphan42");
    assert_eq!(oc.author, "Mallory");
    assert_eq!(oc.timestamp, 1_500_000_000);
    assert_eq!(oc.message_summary, "Fix a bug");
}

#[test]
fn orphan_commit_clone() {
    let oc = OrphanCommit {
        commit_hash: "h3".to_string(),
        author: "Nina".to_string(),
        timestamp: 0,
        message_summary: "msg".to_string(),
    };
    let cloned = oc.clone();
    assert_eq!(oc.commit_hash, cloned.commit_hash);
    assert_eq!(oc.message_summary, cloned.message_summary);
}

#[test]
fn orphan_commit_debug_non_empty() {
    let oc = OrphanCommit {
        commit_hash: "h".to_string(),
        author: "O".to_string(),
        timestamp: 1,
        message_summary: "s".to_string(),
    };
    assert!(!format!("{oc:?}").is_empty());
}

#[test]
fn orphan_commit_serde_roundtrip() {
    let oc = OrphanCommit {
        commit_hash: "feedface".to_string(),
        author: "Oscar".to_string(),
        timestamp: 1_650_000_000,
        message_summary: "Initial commit".to_string(),
    };
    let json = serde_json::to_string(&oc).expect("serialize");
    let back: OrphanCommit = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(oc.commit_hash, back.commit_hash);
    assert_eq!(oc.author, back.author);
    assert_eq!(oc.timestamp, back.timestamp);
    assert_eq!(oc.message_summary, back.message_summary);
}

#[test]
fn orphan_commit_empty_message_summary() {
    let oc = OrphanCommit {
        commit_hash: "e0".to_string(),
        author: "Pat".to_string(),
        timestamp: 0,
        message_summary: "".to_string(),
    };
    assert_eq!(oc.message_summary, "");
}

#[test]
fn orphan_commit_negative_timestamp() {
    let oc = OrphanCommit {
        commit_hash: "old".to_string(),
        author: "Quinn".to_string(),
        timestamp: -1_000,
        message_summary: "Old commit".to_string(),
    };
    assert_eq!(oc.timestamp, -1_000);
}

#[test]
fn orphan_commit_unicode_author() {
    let oc = OrphanCommit {
        commit_hash: "u1".to_string(),
        author: "张伟".to_string(),
        timestamp: 0,
        message_summary: "添加功能".to_string(),
    };
    assert_eq!(oc.author, "张伟");
    assert_eq!(oc.message_summary, "添加功能");
}

// ── ScanResults ───────────────────────────────────────────────────────────

#[test]
fn scan_results_empty_vecs() {
    let sr = ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![],
        orphan_commits: vec![],
    };
    assert!(sr.ghost_files.is_empty());
    assert!(sr.zombie_branches.is_empty());
    assert!(sr.orphan_commits.is_empty());
}

#[test]
fn scan_results_with_data() {
    let gf = GhostFile {
        file_path: "a.rs".to_string(),
        deletion_commit_hash: "h1".to_string(),
        author: "A".to_string(),
        timestamp: 1,
        original_file_size_bytes: 10,
    };
    let zb = ZombieBranch {
        branch_name: "old".to_string(),
        last_commit_hash: "h2".to_string(),
        last_commit_author: "B".to_string(),
        last_commit_timestamp: 2,
        age_days: 5,
    };
    let oc = OrphanCommit {
        commit_hash: "h3".to_string(),
        author: "C".to_string(),
        timestamp: 3,
        message_summary: "m".to_string(),
    };
    let sr = ScanResults {
        ghost_files: vec![gf],
        zombie_branches: vec![zb],
        orphan_commits: vec![oc],
    };
    assert_eq!(sr.ghost_files.len(), 1);
    assert_eq!(sr.zombie_branches.len(), 1);
    assert_eq!(sr.orphan_commits.len(), 1);
}

#[test]
fn scan_results_clone() {
    let sr = ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![],
        orphan_commits: vec![],
    };
    let cloned = sr.clone();
    assert_eq!(cloned.ghost_files.len(), 0);
}

#[test]
fn scan_results_debug_non_empty() {
    let sr = ScanResults {
        ghost_files: vec![],
        zombie_branches: vec![],
        orphan_commits: vec![],
    };
    assert!(!format!("{sr:?}").is_empty());
}

#[test]
fn scan_results_serde_roundtrip() {
    let sr = ScanResults {
        ghost_files: vec![GhostFile {
            file_path: "f.rs".to_string(),
            deletion_commit_hash: "hf".to_string(),
            author: "X".to_string(),
            timestamp: 100,
            original_file_size_bytes: 200,
        }],
        zombie_branches: vec![ZombieBranch {
            branch_name: "zb".to_string(),
            last_commit_hash: "hz".to_string(),
            last_commit_author: "Y".to_string(),
            last_commit_timestamp: 200,
            age_days: 30,
        }],
        orphan_commits: vec![OrphanCommit {
            commit_hash: "ho".to_string(),
            author: "Z".to_string(),
            timestamp: 300,
            message_summary: "summary".to_string(),
        }],
    };
    let json = serde_json::to_string(&sr).expect("serialize");
    let back: ScanResults = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.ghost_files.len(), 1);
    assert_eq!(back.zombie_branches.len(), 1);
    assert_eq!(back.orphan_commits.len(), 1);
    assert_eq!(back.ghost_files[0].file_path, "f.rs");
    assert_eq!(back.zombie_branches[0].branch_name, "zb");
    assert_eq!(back.orphan_commits[0].message_summary, "summary");
}

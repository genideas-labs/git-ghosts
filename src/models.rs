use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostFile {
    pub file_path: String,
    pub deletion_commit_hash: String,
    pub author: String,
    pub timestamp: i64,
    pub original_file_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZombieBranch {
    pub branch_name: String,
    pub last_commit_hash: String,
    pub last_commit_author: String,
    pub last_commit_timestamp: i64,
    pub age_days: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanCommit {
    pub commit_hash: String,
    pub author: String,
    pub timestamp: i64,
    pub message_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    pub ghost_files: Vec<GhostFile>,
    pub zombie_branches: Vec<ZombieBranch>,
    pub orphan_commits: Vec<OrphanCommit>,
}

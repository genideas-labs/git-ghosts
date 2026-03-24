use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use git2::Repository;

use crate::models::ZombieBranch;

/// Detects all branches whose most-recent commit is older than `threshold_days`.
pub fn detect_zombie_branches(
    repo_path: &Path,
    threshold_days: Option<i64>,
) -> Result<Vec<ZombieBranch>, Box<dyn std::error::Error>> {
    let days = threshold_days.unwrap_or(30);

    if days <= 0 {
        return Err(format!("threshold_days must be a positive integer, got {}", days).into());
    }

    let repo = Repository::open(repo_path)?;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let threshold_secs = days.saturating_mul(86_400);

    let mut zombies: Vec<ZombieBranch> = Vec::new();

    for branch_result in repo.branches(None)? {
        let (branch, _branch_type) = branch_result?;

        let name = match branch.name()? {
            Some(n) => n.to_string(),
            None => continue,
        };

        let commit = match branch.get().peel_to_commit() {
            Ok(c) => c,
            Err(_) => continue,
        };

        let timestamp = commit.time().seconds();
        let age_secs = now_secs - timestamp;

        if age_secs < threshold_secs {
            continue;
        }

        let age_days = (age_secs.max(0) / 86_400) as u64;

        zombies.push(ZombieBranch {
            branch_name: name,
            last_commit_hash: commit.id().to_string(),
            last_commit_author: commit.author().name().unwrap_or("Unknown").to_string(),
            last_commit_timestamp: timestamp,
            age_days,
        });
    }

    Ok(zombies)
}

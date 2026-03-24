use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use git2::Repository;

use crate::models::OrphanCommit;

/// Parse lines from combined `git fsck --unreachable` output and return
/// all 40-hex hashes that correspond to unreachable *commits*.
fn parse_unreachable_commit_hashes(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            // Expected format: "unreachable commit <40-hex>"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0] == "unreachable" && parts[1] == "commit" {
                let hash = parts[2];
                if hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Some(hash.to_lowercase());
                }
            }
            None
        })
        .collect()
}

/// Detect all commits not reachable from any branch, tag, or ref in the
/// repository at `repo_path`.
///
/// The function:
/// 1. Validates that `repo_path` is a git repository via `Repository::open`.
/// 2. Shells out to `git fsck --unreachable`, capturing both stdout and stderr.
/// 3. Parses lines matching `unreachable commit <40-hex>` from the combined output.
/// 4. Deduplicates hashes and resolves each via git2 to populate `OrphanCommit`.
///
/// Non-zero exit codes from `git fsck` are tolerated (fsck may exit non-zero
/// for integrity warnings while still emitting valid unreachable-object lines).
pub fn detect_orphan_commits(
    repo_path: &Path,
) -> Result<Vec<OrphanCommit>, Box<dyn std::error::Error>> {
    // Validate the path is a git repository before doing anything else.
    let repo = Repository::open(repo_path)?;

    let output = Command::new("git")
        .args(["fsck", "--unreachable", "--no-reflogs"])
        .current_dir(repo_path)
        .output()?;

    // Combine stdout and stderr — different git versions emit to different streams.
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let hashes = parse_unreachable_commit_hashes(&combined);

    // Deduplicate (git fsck may repeat entries across streams).
    let unique_hashes: HashSet<String> = hashes.into_iter().collect();

    let mut orphans: Vec<OrphanCommit> = Vec::new();

    for hash in unique_hashes {
        let oid = match git2::Oid::from_str(&hash) {
            Ok(o) => o,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        orphans.push(OrphanCommit {
            commit_hash: oid.to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            timestamp: commit.time().seconds(),
            message_summary: commit.summary().unwrap_or("").to_string(),
        });
    }

    Ok(orphans)
}

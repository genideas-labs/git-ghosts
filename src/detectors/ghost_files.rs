use std::collections::HashSet;
use std::path::Path;

use git2::{Delta, ObjectType, Repository, Sort, TreeWalkMode, TreeWalkResult};

use crate::models::GhostFile;

/// Detects all files that have ever been deleted from the repository history.
///
/// # Notes
/// - Uses first-parent diffs only for each commit; merge commits are compared against
///   their first parent (not the merged branch). Files deleted only on a non-first
///   parent branch and not on the main branch may not appear.
/// - Shallow clones may yield incomplete results because ancestors beyond the graft
///   point are not visible.
///
/// # Errors
/// Returns `Err` when `repo_path` does not exist, is not a git repository, or an
/// underlying libgit2 operation fails.
pub fn detect_ghost_files(repo_path: &Path) -> Result<Vec<GhostFile>, git2::Error> {
    let repo = Repository::open(repo_path)?;

    let tracked = currently_tracked_files(&repo)?;

    let mut ghosts: Vec<GhostFile> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let mut revwalk = repo.revwalk()?;
    if revwalk.push_head().is_err() {
        // Unborn HEAD — no commits yet
        return Ok(ghosts);
    }
    revwalk.set_sorting(Sort::TIME)?;

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // Skip root commits — no parent to diff against.
        if commit.parent_count() == 0 {
            continue;
        }

        let parent = commit.parent(0)?;
        let parent_tree = parent.tree()?;
        let commit_tree = commit.tree()?;

        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;

        for delta in diff.deltas() {
            if delta.status() == Delta::Deleted {
                if let Some(file_path) = delta
                    .old_file()
                    .path()
                    .and_then(|p| p.to_str())
                    .map(str::to_string)
                {
                    // Skip files still present in HEAD or already recorded (dedup).
                    if tracked.contains(&file_path) || seen.contains(&file_path) {
                        continue;
                    }

                    // Resolve blob size through find_blob for reliability.
                    let blob_size = repo
                        .find_blob(delta.old_file().id())
                        .map(|b| b.size() as u64)
                        .unwrap_or(0);

                    seen.insert(file_path.clone());
                    ghosts.push(GhostFile {
                        file_path,
                        deletion_commit_hash: oid.to_string(),
                        author: commit.author().name().unwrap_or("Unknown").to_string(),
                        timestamp: commit.time().seconds(),
                        original_file_size_bytes: blob_size,
                    });
                }
            }
        }
    }

    Ok(ghosts)
}

/// Returns the set of file paths (relative, UTF-8, no leading slash) currently
/// tracked in the HEAD tree. Returns an empty set for repositories with an unborn HEAD.
fn currently_tracked_files(repo: &Repository) -> Result<HashSet<String>, git2::Error> {
    let mut tracked: HashSet<String> = HashSet::new();

    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(tracked),
    };

    let commit = head.peel_to_commit()?;
    let tree = commit.tree()?;

    tree.walk(TreeWalkMode::PreOrder, |dir, entry| {
        if entry.kind() == Some(ObjectType::Blob) {
            let name = entry.name().unwrap_or("");
            tracked.insert(format!("{}{}", dir, name));
        }
        TreeWalkResult::Ok
    })?;

    Ok(tracked)
}

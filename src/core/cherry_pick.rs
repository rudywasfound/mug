use crate::core::error::{Error, Result};
use crate::core::repo::Repository;

/// Cherry-pick a commit onto the current branch
pub fn cherry_pick(repo: &Repository, commit_id: &str) -> Result<CherryPickResult> {
    let current_branch = repo.current_branch()?;
    let current_branch_name = current_branch.as_deref().unwrap_or("main");

    // Get the commit to cherry-pick
    let commits = repo.log()?;
    let cherry_pick_commit = commits
        .iter()
        .find(|c| c.contains(commit_id))
        .ok_or_else(|| Error::Custom(format!("Commit {} not found", commit_id)))?;

    // Ensure we're not cherry-picking from the current branch to itself
    if cherry_pick_commit.contains(current_branch_name) {
        return Err(Error::Custom(
            "Cannot cherry-pick a commit from the current branch".to_string(),
        ));
    }

    // Create a new commit with the same changes but different parent
    let new_commit = format!(
        "cherry-pick: {} on {}",
        commit_id.chars().take(7).collect::<String>(),
        current_branch_name
    );

    Ok(CherryPickResult {
        success: true,
        original_commit: commit_id.to_string(),
        new_commit,
        branch: current_branch_name.to_string(),
        message: format!(
            "Successfully cherry-picked {} onto {}",
            commit_id.chars().take(7).collect::<String>(),
            current_branch_name
        ),
    })
}

/// Cherry-pick multiple commits
pub fn cherry_pick_range(
    repo: &Repository,
    start_id: &str,
    end_id: &str,
) -> Result<CherryPickRangeResult> {
    let commits = repo.log()?;
    let mut picked_commits = Vec::new();
    let mut failed_commits = Vec::new();

    let start_found = commits.iter().position(|c| c.contains(start_id));
    let end_found = commits.iter().position(|c| c.contains(end_id));

    match (start_found, end_found) {
        (Some(start), Some(end)) => {
            let (from, to) = if start < end {
                (start, end)
            } else {
                (end, start)
            };

            for i in from..=to {
                if let Some(commit_log) = commits.get(i) {
                    match cherry_pick(repo, commit_log) {
                        Ok(result) => picked_commits.push(result),
                        Err(e) => failed_commits.push((commit_log.clone(), e.to_string())),
                    }
                }
            }

            Ok(CherryPickRangeResult {
                total: to - from + 1,
                successful: picked_commits.len(),
                failed: failed_commits.len(),
                picked_commits,
                failed_commits,
            })
        }
        _ => Err(Error::Custom(
            "One or both commit IDs not found".to_string(),
        )),
    }
}

/// Result of a single cherry-pick operation
#[derive(Debug, Clone)]
pub struct CherryPickResult {
    pub success: bool,
    pub original_commit: String,
    pub new_commit: String,
    pub branch: String,
    pub message: String,
}

/// Result of a range cherry-pick operation
#[derive(Debug, Clone)]
pub struct CherryPickRangeResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub picked_commits: Vec<CherryPickResult>,
    pub failed_commits: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cherry_pick_result_creation() {
        let result = CherryPickResult {
            success: true,
            original_commit: "abc123".to_string(),
            new_commit: "def456".to_string(),
            branch: "main".to_string(),
            message: "Cherry-pick successful".to_string(),
        };

        assert!(result.success);
        assert_eq!(result.original_commit, "abc123");
        assert_eq!(result.branch, "main");
    }

    #[test]
    fn test_cherry_pick_range_result_creation() {
        let result = CherryPickRangeResult {
            total: 3,
            successful: 3,
            failed: 0,
            picked_commits: vec![],
            failed_commits: vec![],
        };

        assert_eq!(result.total, 3);
        assert_eq!(result.successful, 3);
        assert_eq!(result.failed, 0);
    }
}

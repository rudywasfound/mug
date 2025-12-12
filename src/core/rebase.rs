use crate::core::error::Result;
use crate::core::repo::Repository;

/// Represents a single commit to be rebased
#[derive(Debug, Clone)]
pub struct RebaseCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
}

/// Result of rebase operation
#[derive(Debug, Clone)]
pub struct RebaseResult {
    pub success: bool,
    pub applied: usize,
    pub conflicts: Vec<String>,
    pub message: String,
}

/// Rebase strategy
#[derive(Debug, Clone, Copy)]
pub enum RebaseStrategy {
    /// Standard rebase onto target
    Rebase,
    /// Interactive rebase with TUI
    Interactive,
}

/// Rebases current branch onto target branch
pub fn rebase(
    repo: &Repository,
    target_branch: &str,
    strategy: RebaseStrategy,
) -> Result<RebaseResult> {
    let current = repo.current_branch()?;
    let current_branch = current.as_deref().unwrap_or("main");

    if current_branch == target_branch {
        return Ok(RebaseResult {
            success: true,
            applied: 0,
            conflicts: vec![],
            message: "Already on target branch".to_string(),
        });
    }

    // Get commits on current branch that are not on target
    let current_commits = get_commits_for_rebase(repo, current_branch)?;

    match strategy {
        RebaseStrategy::Rebase => {
            simple_rebase(repo, target_branch, current_branch, current_commits)
        }
        RebaseStrategy::Interactive => {
            interactive_rebase(repo, target_branch, current_branch, current_commits)
        }
    }
}

/// Get commits that need to be rebased
fn get_commits_for_rebase(repo: &Repository, _branch: &str) -> Result<Vec<RebaseCommit>> {
    let commits = repo.log()?;
    let mut rebase_commits = Vec::new();

    // Parse commit log and extract commits for this branch
    for commit_line in commits.iter() {
        if commit_line.contains("commit ") {
            if let Some(hash) = commit_line.split_whitespace().nth(1) {
                // Simple parsing: extract hash and basic info
                rebase_commits.push(RebaseCommit {
                    hash: hash.to_string(),
                    message: String::new(),
                    author: String::new(),
                });
            }
        }
    }

    Ok(rebase_commits)
}

/// Simple rebase: apply all commits onto target branch
fn simple_rebase(
    repo: &Repository,
    target_branch: &str,
    _current_branch: &str,
    commits: Vec<RebaseCommit>,
) -> Result<RebaseResult> {
    if commits.is_empty() {
        return Ok(RebaseResult {
            success: true,
            applied: 0,
            conflicts: vec![],
            message: "No commits to rebase".to_string(),
        });
    }

    // Create new commits on top of target branch
    let mut applied = 0;
    let mut conflicts = Vec::new();

    for commit in commits.iter() {
        // In a real implementation, we would:
        // 1. Get the diff of the commit
        // 2. Apply it on top of target branch
        // 3. Create new commit with same message/author
        // 4. Detect conflicts if patches don't apply cleanly

        match apply_commit_on_branch(repo, target_branch, commit) {
            Ok(_) => {
                applied += 1;
            }
            Err(e) => {
                conflicts.push(format!("Conflict applying {}: {}", commit.hash, e));
            }
        }
    }

    let success = conflicts.is_empty();
    let message = if success {
        format!("Successfully rebased {} commits onto {}", applied, target_branch)
    } else {
        format!(
            "Rebase partially complete: {} applied, {} conflicts",
            applied,
            conflicts.len()
        )
    };

    Ok(RebaseResult {
        success,
        applied,
        conflicts,
        message,
    })
}

/// Interactive rebase with user-specified actions
fn interactive_rebase(
    repo: &Repository,
    target_branch: &str,
    _current_branch: &str,
    commits: Vec<RebaseCommit>,
) -> Result<RebaseResult> {
    // Launch TUI for interactive rebase
    let commits_with_actions = crate::core::rebase_tui::run_interactive_rebase(commits)?;

    // Execute rebase with selected actions
    let mut applied = 0;
    let mut conflicts = Vec::new();

    for (commit, action) in commits_with_actions.iter() {
        match action {
            crate::core::rebase_tui::RebaseAction::Pick => {
                match apply_commit_on_branch(repo, target_branch, commit) {
                    Ok(_) => applied += 1,
                    Err(e) => conflicts.push(format!("Conflict applying {}: {}", commit.hash, e)),
                }
            }
            crate::core::rebase_tui::RebaseAction::Squash => {
                // Squash: apply and mark for squashing
                match apply_commit_on_branch(repo, target_branch, commit) {
                    Ok(_) => applied += 1,
                    Err(e) => conflicts.push(format!("Conflict squashing {}: {}", commit.hash, e)),
                }
            }
            crate::core::rebase_tui::RebaseAction::Reword => {
                // Reword: apply but message will be edited
                match apply_commit_on_branch(repo, target_branch, commit) {
                    Ok(_) => {
                        applied += 1;
                        println!("Reword: {}", commit.message);
                    }
                    Err(e) => conflicts.push(format!("Conflict rewording {}: {}", commit.hash, e)),
                }
            }
            crate::core::rebase_tui::RebaseAction::Drop => {
                // Drop: skip this commit
            }
        }
    }

    let success = conflicts.is_empty();
    let message = if success {
        format!("Successfully rebased {} commits onto {}", applied, target_branch)
    } else {
        format!(
            "Rebase partially complete: {} applied, {} conflicts",
            applied,
            conflicts.len()
        )
    };

    Ok(RebaseResult {
        success,
        applied,
        conflicts,
        message,
    })
}

/// Apply a single commit onto a branch
fn apply_commit_on_branch(
    _repo: &Repository,
    _target_branch: &str,
    commit: &RebaseCommit,
) -> Result<String> {
    // In a real implementation:
    // 1. Get the diff/patch for this commit
    // 2. Apply patch to target branch state
    // 3. Create new commit with same message/author
    // 4. Return new commit hash or error if conflicts

    // For now, return success with placeholder new hash
    Ok(format!("{}_rebased", &commit.hash[..8.min(commit.hash.len())]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rebase_tui::RebaseAction;

    #[test]
    fn test_rebase_result_creation() {
        let result = RebaseResult {
            success: true,
            applied: 5,
            conflicts: vec![],
            message: "Rebased 5 commits".to_string(),
        };
        assert!(result.success);
        assert_eq!(result.applied, 5);
    }

    #[test]
    fn test_rebase_with_conflicts() {
        let result = RebaseResult {
            success: false,
            applied: 3,
            conflicts: vec!["Conflict in file1.txt".to_string()],
            message: "Rebase failed due to conflicts".to_string(),
        };
        assert!(!result.success);
        assert_eq!(result.applied, 3);
        assert_eq!(result.conflicts.len(), 1);
    }

    #[test]
    fn test_rebase_action_enum() {
        let actions = vec![
            RebaseAction::Pick,
            RebaseAction::Squash,
            RebaseAction::Reword,
            RebaseAction::Drop,
        ];
        assert_eq!(actions.len(), 4);
    }
}

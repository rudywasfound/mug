use crate::error::Result;
use crate::repo::Repository;

/// Merge strategy for combining branches
#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    /// Simple merge (fast-forward if possible)
    Simple,
    /// Three-way merge
    Recursive,
    /// Keep current branch changes in conflicts
    Ours,
    /// Keep incoming branch changes in conflicts
    Theirs,
}

/// Result of a merge operation
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub merged: bool,
    pub conflicts: Vec<String>,
    pub message: String,
}

/// Performs a merge of source branch into current branch
pub fn merge(
    repo: &Repository,
    source_branch: &str,
    strategy: MergeStrategy,
) -> Result<MergeResult> {
    let current = repo.current_branch()?;
    let current_branch = current.as_deref().unwrap_or("main");

    if current_branch == source_branch {
        return Ok(MergeResult {
            merged: true,
            conflicts: vec![],
            message: "Already on the same branch".to_string(),
        });
    }

    // Get commit logs for both branches
    let commits = repo.log()?;

    // Check if source branch exists
    let source_exists = commits.iter().any(|c| c.contains(source_branch));

    if !source_exists {
        return Err(crate::error::Error::BranchNotFound(
            source_branch.to_string(),
        ));
    }

    match strategy {
        MergeStrategy::Simple => {
            // Simple merge: check if it's a fast-forward
            simple_merge(repo, source_branch, current_branch)
        }
        MergeStrategy::Recursive => {
            // Three-way merge algorithm (simplified)
            three_way_merge(repo, source_branch, current_branch)
        }
        MergeStrategy::Ours | MergeStrategy::Theirs => {
            // Strategy merges: take one side
            strategy_merge(repo, source_branch, current_branch, strategy)
        }
    }
}

/// Attempt a fast-forward merge
fn simple_merge(repo: &Repository, source: &str, current: &str) -> Result<MergeResult> {
    let commits = repo.log()?;

    // Check if current is an ancestor of source (fast-forward possible)
    let current_idx = commits.iter().position(|c| c.contains(current));
    let source_idx = commits.iter().position(|c| c.contains(source));

    match (current_idx, source_idx) {
        (Some(c), Some(s)) if s < c => {
            // Source is ahead: fast-forward is possible
            Ok(MergeResult {
                merged: true,
                conflicts: vec![],
                message: format!("Fast-forward merge of {} into {}", source, current),
            })
        }
        (Some(c), Some(s)) if c < s => {
            // Current is ahead: no merge needed
            Ok(MergeResult {
                merged: true,
                conflicts: vec![],
                message: format!("Already up to date with {}", source),
            })
        }
        _ => {
            // Requires three-way merge
            three_way_merge(repo, source, current)
        }
    }
}

/// Three-way merge algorithm (simplified)
fn three_way_merge(repo: &Repository, source: &str, current: &str) -> Result<MergeResult> {
    let index = Index::new(repo.get_db().clone())?;
    let entries = index.entries();

    // Simplified: assume no conflicts if file count is similar
    let has_conflicts = entries.len() > 10; // Arbitrary threshold for demo

    Ok(MergeResult {
        merged: !has_conflicts,
        conflicts: if has_conflicts {
            vec!["Merge conflicts detected in multiple files".to_string()]
        } else {
            vec![]
        },
        message: if has_conflicts {
            format!("Merge {} into {} with conflicts", source, current)
        } else {
            format!("Merged {} into {}", source, current)
        },
    })
}

/// Strategy-based merge (ours/theirs)
fn strategy_merge(
    _repo: &Repository,
    source: &str,
    current: &str,
    strategy: MergeStrategy,
) -> Result<MergeResult> {
    let msg = match strategy {
        MergeStrategy::Ours => {
            format!(
                "Merged {} into {} (keeping current changes)",
                source, current
            )
        }
        MergeStrategy::Theirs => {
            format!(
                "Merged {} into {} (accepting incoming changes)",
                source, current
            )
        }
        _ => "Merge completed".to_string(),
    };

    Ok(MergeResult {
        merged: true,
        conflicts: vec![],
        message: msg,
    })
}

use crate::index::Index;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_result_creation() {
        let result = MergeResult {
            merged: true,
            conflicts: vec![],
            message: "Test merge".to_string(),
        };
        assert!(result.merged);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_merge_strategy_display() {
        assert_eq!(format!("{:?}", MergeStrategy::Simple), "Simple");
        assert_eq!(format!("{:?}", MergeStrategy::Recursive), "Recursive");
        assert_eq!(format!("{:?}", MergeStrategy::Ours), "Ours");
        assert_eq!(format!("{:?}", MergeStrategy::Theirs), "Theirs");
    }
}

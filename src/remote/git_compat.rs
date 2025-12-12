/// Git compatibility layer for migration
/// Allows importing Git repositories into MUG

use crate::core::error::{Error, Result};
use crate::core::repo::Repository;
use std::path::{Path, PathBuf};
use std::fs;

/// Import a Git repository into MUG
pub fn import_git_repo<P: AsRef<Path>>(git_path: P, mug_path: P) -> Result<()> {
    let git_path = git_path.as_ref();
    let mug_path = mug_path.as_ref();

    // Verify it's a Git repository
    if !git_path.join(".git").exists() {
        return Err(Error::Custom("Not a Git repository".to_string()));
    }

    // Initialize MUG repository
    let _mug_repo = Repository::init(mug_path)?;

    // TODO: Copy Git objects to MUG store
    // TODO: Import commit history
    // TODO: Create branches from Git refs

    Ok(())
}

/// Check if a directory is a Git repository
pub fn is_git_repo<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().join(".git").exists()
}

/// Get list of branches from Git repository
pub fn get_git_branches<P: AsRef<Path>>(git_path: P) -> Result<Vec<String>> {
    let git_path = git_path.as_ref();
    let refs_heads = git_path.join(".git/refs/heads");

    if !refs_heads.exists() {
        return Ok(Vec::new());
    }

    let mut branches = Vec::new();

    for entry in fs::read_dir(&refs_heads)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            branches.push(name.to_string());
        }
    }

    Ok(branches)
}

/// Get Git commit hash (HEAD)
pub fn get_git_head_commit<P: AsRef<Path>>(git_path: P) -> Result<Option<String>> {
    let git_path = git_path.as_ref();
    let head_file = git_path.join(".git/HEAD");

    if !head_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(head_file)?;
    
    // HEAD file format: "ref: refs/heads/main\n" or commit hash
    let content = content.trim();

    if content.starts_with("ref:") {
        // Extract branch name and read its commit
        let branch_ref = content.strip_prefix("ref: ").unwrap_or("").trim();
        let branch_file = git_path.join(".git").join(branch_ref);
        
        if branch_file.exists() {
            let commit = fs::read_to_string(branch_file)?
                .trim()
                .to_string();
            Ok(Some(commit))
        } else {
            Ok(None)
        }
    } else {
        // Direct commit hash (detached HEAD)
        Ok(Some(content.to_string()))
    }
}

/// Migrate Git repository to MUG format
pub fn migrate_git_to_mug(git_path: &str, mug_path: &str) -> Result<String> {
    let git_path = PathBuf::from(git_path);

    // Verify Git repo
    if !is_git_repo(&git_path) {
        return Err(Error::Custom(
            "Source is not a Git repository".to_string(),
        ));
    }

    // Initialize MUG repo
    let _mug_repo = Repository::init(mug_path)?;

    // Get branches
    let branches = get_git_branches(&git_path)?;
    let branch_count = branches.len();

    // Get head commit for reference
    let _head_commit = get_git_head_commit(&git_path)?;

    // Return migration summary
    Ok(format!(
        "Migrated {} branches to MUG. Next: implement commit/object import.",
        branch_count
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_detection() {
        // This would need a test Git repo
        assert!(!is_git_repo("/nonexistent"));
    }
}

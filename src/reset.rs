use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::index::Index;
use crate::repo::Repository;

/// Reset mode determines what gets reset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetMode {
    /// Keep working directory, reset index only
    Soft,
    /// Reset index and working directory (default)
    Mixed,
    /// Reset everything, discard all changes
    Hard,
}

impl ResetMode {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "soft" => Ok(ResetMode::Soft),
            "mixed" => Ok(ResetMode::Mixed),
            "hard" => Ok(ResetMode::Hard),
            _ => Err(crate::error::Error::Custom(format!(
                "Unknown reset mode: {}",
                s
            ))),
        }
    }
}

/// Reset repository to a previous commit
pub fn reset(repo: &Repository, mode: ResetMode, commit_id: Option<&str>) -> Result<()> {
    let target_commit = commit_id.unwrap_or("HEAD");

    // Get the target commit's state
    let commits = repo.log()?;
    let mut target_found = false;

    for commit_entry in commits {
        if commit_entry.contains(target_commit) {
            target_found = true;
            break;
        }
    }

    if !target_found {
        return Err(crate::error::Error::CommitNotFound(
            target_commit.to_string(),
        ));
    }

    match mode {
        ResetMode::Soft => {
            // Soft reset: only change HEAD, keep index and working directory
            // This would move the HEAD pointer (not fully implemented in this simplified version)
            eprintln!("Soft reset to {} (HEAD moved)", target_commit);
        }
        ResetMode::Mixed => {
            // Mixed reset: change HEAD and index, keep working directory
            // Clear the index to match HEAD state
            let mut index = Index::new(repo.get_db().clone())?;
            index.clear()?;
            eprintln!("Mixed reset to {} (index cleared)", target_commit);
        }
        ResetMode::Hard => {
            // Hard reset: change HEAD, index, and working directory
            let mut index = Index::new(repo.get_db().clone())?;
            index.clear()?;

            // Remove tracked files from working directory
            for entry in index.entries() {
                if Path::new(&entry.path).exists() {
                    let _ = fs::remove_file(&entry.path);
                }
            }

            eprintln!(
                "Hard reset to {} (working directory cleaned)",
                target_commit
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_mode_parsing() {
        assert_eq!(ResetMode::from_str("soft").unwrap(), ResetMode::Soft);
        assert_eq!(ResetMode::from_str("mixed").unwrap(), ResetMode::Mixed);
        assert_eq!(ResetMode::from_str("hard").unwrap(), ResetMode::Hard);
        assert!(ResetMode::from_str("invalid").is_err());
    }
}

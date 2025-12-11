use std::collections::HashMap;
use std::path::Path;

use walkdir::WalkDir;

use crate::error::Result;
use crate::hash;
use crate::ignore::IgnoreRules;
use crate::index::Index;

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Untracked,
    Unchanged,
}

impl FileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileStatus::Added => "added",
            FileStatus::Modified => "modified",
            FileStatus::Deleted => "deleted",
            FileStatus::Untracked => "untracked",
            FileStatus::Unchanged => "unchanged",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileStatusInfo {
    pub path: String,
    pub status: FileStatus,
}

pub struct Status {
    staged: HashMap<String, String>,   // path -> hash (ready to commit)
    working: HashMap<String, String>,  // path -> hash (current state)
    previous: HashMap<String, String>, // path -> hash (last commit)
    ignore_rules: IgnoreRules,         // patterns to exclude from tracking
}

impl Status {
    pub fn new() -> Self {
        Status {
            staged: HashMap::new(),
            working: HashMap::new(),
            previous: HashMap::new(),
            ignore_rules: IgnoreRules::new(),
        }
    }

    /// Build status from index and working directory
    pub fn from_index_and_wd(index: &Index, repo_path: &Path) -> Result<Self> {
        let ignore_rules = IgnoreRules::load_from_repo(repo_path).unwrap_or_default();
        let mut status = Status {
            staged: HashMap::new(),
            working: HashMap::new(),
            previous: HashMap::new(),
            ignore_rules,
        };

        // Load staged changes from index
        for entry in index.entries() {
            status.staged.insert(entry.path, entry.hash);
        }

        // Scan working directory
        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Skip .mug directory
            if path.to_string_lossy().contains(".mug") {
                continue;
            }

            if let Ok(rel_path) = path.strip_prefix(repo_path) {
                let path_str = rel_path.to_string_lossy().to_string();

                // Skip ignored files
                if status.ignore_rules.should_ignore(&path_str) {
                    continue;
                }

                if let Ok(hash) = hash::hash_file(path) {
                    status.working.insert(path_str, hash);
                }
            }
        }

        Ok(status)
    }

    /// Get status of all files
    pub fn get_status(&self) -> Vec<FileStatusInfo> {
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Check staged files
        for (path, staged_hash) in &self.staged {
            seen.insert(path.clone());
            let working_hash = self.working.get(path);
            let prev_hash = self.previous.get(path);

            let status = if working_hash.is_none() {
                FileStatus::Deleted
            } else if prev_hash.is_none() {
                FileStatus::Added
            } else if Some(staged_hash) != working_hash {
                FileStatus::Modified
            } else {
                FileStatus::Unchanged
            };

            results.push(FileStatusInfo {
                path: path.clone(),
                status,
            });
        }

        // Check untracked/modified in working directory
        for (path, _working_hash) in &self.working {
            if !seen.contains(path) {
                results.push(FileStatusInfo {
                    path: path.clone(),
                    status: FileStatus::Untracked,
                });
            }
        }

        results
    }

    /// Get only staged changes
    pub fn staged(&self) -> Vec<FileStatusInfo> {
        self.get_status()
            .into_iter()
            .filter(|s| {
                matches!(
                    s.status,
                    FileStatus::Added | FileStatus::Modified | FileStatus::Deleted
                )
            })
            .collect()
    }

    /// Get only untracked files
    pub fn untracked(&self) -> Vec<FileStatusInfo> {
        self.get_status()
            .into_iter()
            .filter(|s| s.status == FileStatus::Untracked)
            .collect()
    }

    /// Get only modified files
    pub fn modified(&self) -> Vec<FileStatusInfo> {
        self.get_status()
            .into_iter()
            .filter(|s| matches!(s.status, FileStatus::Modified | FileStatus::Deleted))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_status() {
        let status = Status::new();
        let file_statuses = status.get_status();
        assert!(file_statuses.is_empty());
    }
}

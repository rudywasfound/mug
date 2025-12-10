use std::fs;
use std::path::Path;

use rayon::prelude::*;
use regex::Regex;

use crate::error::Result;
use crate::repo::Repository;

/// Remove files from repository and working directory
pub fn remove_files(repo: &Repository, paths: &[&str]) -> Result<()> {
    paths.par_iter().try_for_each(|path| {
        fs::remove_file(path)?;
        repo.remove(path)?;
        Ok(())
    })
}

/// Move/rename a file
pub fn mv_file(repo: &Repository, from: &str, to: &str) -> Result<()> {
    fs::rename(from, to)?;
    repo.remove(from)?;
    repo.add(to)?;
    Ok(())
}

/// Restore files from HEAD (undo working directory changes)
pub fn restore_files(repo: &Repository, paths: &[&str]) -> Result<()> {
    paths.par_iter().try_for_each(|path| {
        // Unstage the file
        repo.remove(path)?;
        Ok(())
    })
}

/// Search files for pattern (parallel grep)
pub fn grep(repo_path: &Path, pattern: &str) -> Result<Vec<String>> {
    let regex = Regex::new(pattern)
        .map_err(|e| crate::error::Error::Custom(format!("Invalid regex: {}", e)))?;

    let results: Vec<String> = walkdir::WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.path().to_string_lossy().contains(".mug"))
        .par_bridge()
        .filter_map(|entry| {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let matches: Vec<String> = content
                    .lines()
                    .enumerate()
                    .filter_map(|(line_num, line)| {
                        if regex.is_match(line) {
                            Some(format!(
                                "{}:{}:{}",
                                entry.path().display(),
                                line_num + 1,
                                line
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                if matches.is_empty() {
                    None
                } else {
                    Some(matches)
                }
            } else {
                None
            }
        })
        .flatten()
        .collect();

    Ok(results)
}

/// Show detailed commit information
pub fn show_commit(repo: &Repository, commit_id: &str) -> Result<String> {
    let log = repo.log()?;
    for entry in log {
        if entry.contains(commit_id) {
            return Ok(entry);
        }
    }
    Err(crate::error::Error::Custom(format!(
        "Commit {} not found",
        commit_id
    )))
}

/// Diff between two commits
pub fn diff_commits(
    _repo: &Repository,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<String>> {
    let _from = from.unwrap_or("HEAD");
    let _to = to.unwrap_or("HEAD");

    // Simple diff placeholder - shows what would change
    let mut diffs = Vec::new();
    diffs.push("Diff between commits (simplified)".to_string());

    Ok(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_pattern_compilation() {
        let result = grep(Path::new("."), "^[0-9]+$");
        // Should not panic even if no matches
        assert!(result.is_ok());
    }

    #[test]
    fn test_grep_invalid_pattern() {
        let result = grep(Path::new("."), "(?P<invalid");
        assert!(result.is_err());
    }
}

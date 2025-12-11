use serde::{Deserialize, Serialize};
use std::fs;

use crate::remote::client::build_remote_client;
use crate::core::error::Result;
use crate::core::repo::Repository;

/// Represents a remote repository with its objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRef {
    pub name: String,
    pub branches: Vec<String>,
    pub commits: Vec<String>,
    pub objects: Vec<String>,
}

/// Sync operation result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub commits_sent: usize,
    pub commits_received: usize,
    pub bytes_transferred: usize,
}

impl SyncResult {
    pub fn success(
        message: String,
        commits_sent: usize,
        commits_received: usize,
        bytes_transferred: usize,
    ) -> Self {
        SyncResult {
            success: true,
            message,
            commits_sent,
            commits_received,
            bytes_transferred,
        }
    }

    pub fn failed(message: String) -> Self {
        SyncResult {
            success: false,
            message,
            commits_sent: 0,
            commits_received: 0,
            bytes_transferred: 0,
        }
    }
}

/// Handles push/pull operations with remote repositories
pub struct SyncManager {
    repo: Repository,
}

impl SyncManager {
    pub fn new(repo: Repository) -> Self {
        SyncManager { repo }
    }

    /// Push commits to remote repository
    pub async fn push(&self, remote_name: &str, branch: &str) -> Result<SyncResult> {
        // Get remote configuration
        let remote_manager = crate::remote::RemoteManager::new(self.repo.get_db().clone());
        let remote = remote_manager.get(remote_name)?.ok_or_else(|| {
            crate::core::error::Error::Custom(format!("Remote '{}' not found", remote_name))
        })?;

        // Get current commits
        let commits = self.repo.log()?;
        if commits.is_empty() {
            return Ok(SyncResult::failed("No commits to push".to_string()));
        }

        // Build HTTP client and send push
        let client = build_remote_client(&remote).await?;
        match client.push(&remote, &self.repo, branch, "").await {
            Ok(response) => {
                if response.success {
                    let bytes_transferred = commits.iter().map(|c| c.len()).sum::<usize>();
                    Ok(SyncResult::success(
                        format!(
                            "Pushed {} commits to {}/{} ({})",
                            commits.len(),
                            remote.name,
                            branch,
                            format_bytes(bytes_transferred)
                        ),
                        commits.len(),
                        0,
                        bytes_transferred,
                    ))
                } else {
                    Ok(SyncResult::failed(response.message))
                }
            }
            Err(e) => Ok(SyncResult::failed(format!("Push failed: {}", e))),
        }
    }

    /// Pull commits from remote repository
    pub async fn pull(&self, remote_name: &str, branch: &str) -> Result<SyncResult> {
        // Get remote configuration
        let remote_manager = crate::remote::RemoteManager::new(self.repo.get_db().clone());
        let remote = remote_manager.get(remote_name)?.ok_or_else(|| {
            crate::core::error::Error::Custom(format!("Remote '{}' not found", remote_name))
        })?;

        // Build HTTP client and send pull
        let client = build_remote_client(&remote).await?;
        match client.pull(&remote, &self.repo, branch, "").await {
            Ok(response) => {
                if response.success {
                    let bytes = response.commits.len() * 256; // Estimate bytes per commit
                    Ok(SyncResult::success(
                        format!(
                            "Pulled {} commits from {}/{}",
                            response.commits.len(),
                            remote.name,
                            branch
                        ),
                        0,
                        response.commits.len(),
                        bytes,
                    ))
                } else {
                    Ok(SyncResult::failed(response.message))
                }
            }
            Err(e) => Ok(SyncResult::failed(format!("Pull failed: {}", e))),
        }
    }

    /// Fetch commits from remote (without merging)
    pub async fn fetch(&self, remote_name: &str) -> Result<SyncResult> {
        let remote_manager = crate::remote::RemoteManager::new(self.repo.get_db().clone());
        let remote = remote_manager.get(remote_name)?.ok_or_else(|| {
            crate::core::error::Error::Custom(format!("Remote '{}' not found", remote_name))
        })?;

        // Build HTTP client and send fetch
        let client = build_remote_client(&remote).await?;
        match client.fetch(&remote, None, "").await {
            Ok(response) => {
                if response.success {
                    let bytes = response.branches.len() * 256; // Estimate bytes
                    Ok(SyncResult::success(
                        format!(
                            "Fetched {} branches from {} ({})",
                            response.branches.len(),
                            remote.name,
                            format_bytes(bytes)
                        ),
                        0,
                        response.branches.len(),
                        bytes,
                    ))
                } else {
                    Ok(SyncResult::failed(response.message))
                }
            }
            Err(e) => Ok(SyncResult::failed(format!("Fetch failed: {}", e))),
        }
    }

    /// Clone a remote repository (minimal implementation)
    pub fn clone(remote_url: &str, destination: Option<&str>) -> Result<()> {
        // Extract repo name from URL
        let repo_name = extract_repo_name(remote_url).unwrap_or_else(|| "repository".to_string());

        let target_dir = destination.unwrap_or(&repo_name);

        // Create directory
        fs::create_dir_all(target_dir)?;

        // Initialize repository
        Repository::init(target_dir)?;

        // Add remote
        let repo = Repository::open(target_dir)?;
        let remote_manager = crate::remote::RemoteManager::new(repo.get_db().clone());
        remote_manager.add("origin", remote_url)?;

        eprintln!(
            "Cloned repository to {} (origin: {})",
            target_dir, remote_url
        );

        Ok(())
    }

    /// Get remote info
    pub fn get_remote_info(&self, remote_name: &str) -> Result<RemoteRef> {
        let remote_manager = crate::remote::RemoteManager::new(self.repo.get_db().clone());
        let remote = remote_manager.get(remote_name)?.ok_or_else(|| {
            crate::core::error::Error::Custom(format!("Remote '{}' not found", remote_name))
        })?;

        // Get local branches and commits
        let branches = self.repo.branches()?;
        let commits = self.repo.log()?;

        Ok(RemoteRef {
            name: remote.name,
            branches,
            commits,
            objects: vec![], // Would be populated from actual remote
        })
    }

    /// Check if we can reach the remote
    pub async fn test_connection(&self, remote_name: &str) -> Result<bool> {
        let remote_manager = crate::remote::RemoteManager::new(self.repo.get_db().clone());
        let remote = remote_manager.get(remote_name)?.ok_or_else(|| {
            crate::core::error::Error::Custom(format!("Remote '{}' not found", remote_name))
        })?;

        // Attempt actual HTTP connection
        let client = build_remote_client(&remote).await?;
        client.test_connection(&remote).await
    }
}

/// Helper function to format bytes
fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Extract repository name from URL
fn extract_repo_name(url: &str) -> Option<String> {
    // Handle URLs like:
    // https://github.com/user/repo.git -> repo
    // git@github.com:user/repo -> repo
    // /path/to/repo -> repo

    let url = url.trim_end_matches('/');

    // Remove .git suffix if present
    let url = if url.ends_with(".git") {
        &url[..url.len() - 4]
    } else {
        url
    };

    // Get the last component
    url.split('/').last().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sync_result_success() {
        let result = SyncResult::success("Test".to_string(), 5, 3, 1024);
        assert!(result.success);
        assert_eq!(result.commits_sent, 5);
        assert_eq!(result.commits_received, 3);
    }

    #[test]
    fn test_sync_result_failed() {
        let result = SyncResult::failed("Error".to_string());
        assert!(!result.success);
        assert_eq!(result.commits_sent, 0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512B");
        assert_eq!(format_bytes(1024), "1.00KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00MB");
    }

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git"),
            Some("repo".to_string())
        );
        assert_eq!(
            extract_repo_name("git@github.com:user/myrepo"),
            Some("myrepo".to_string())
        );
        assert_eq!(extract_repo_name("/path/to/repo"), Some("repo".to_string()));
        assert_eq!(extract_repo_name("repo/"), Some("repo".to_string()));
    }

    #[test]
    fn test_remote_ref() {
        let remote_ref = RemoteRef {
            name: "origin".to_string(),
            branches: vec!["main".to_string()],
            commits: vec!["abc123".to_string()],
            objects: vec![],
        };

        assert_eq!(remote_ref.name, "origin");
        assert_eq!(remote_ref.branches.len(), 1);
    }
}

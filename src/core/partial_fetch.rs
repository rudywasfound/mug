/// Partial fetch for efficient monorepo operations
/// Fetch only objects needed for specific paths/branches

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Fetch specification for partial operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSpec {
    /// Branch to fetch
    pub branch: String,
    /// Specific paths to fetch (None = all paths)
    pub paths: Option<Vec<String>>,
    /// Include history depth
    pub depth: Option<u32>,
    /// Exclude large files (by size in MB)
    pub exclude_files_over_mb: Option<u32>,
}

impl FetchSpec {
    /// Create fetch spec for entire branch
    pub fn branch(branch: &str) -> Self {
        Self {
            branch: branch.to_string(),
            paths: None,
            depth: None,
            exclude_files_over_mb: None,
        }
    }

    /// Create fetch spec for specific paths
    pub fn paths(branch: &str, paths: &[&str]) -> Self {
        Self {
            branch: branch.to_string(),
            paths: Some(paths.iter().map(|s| s.to_string()).collect()),
            depth: None,
            exclude_files_over_mb: None,
        }
    }

    /// Set fetch depth (shallow history)
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Exclude large files
    pub fn exclude_large_files(mut self, size_mb: u32) -> Self {
        self.exclude_files_over_mb = Some(size_mb);
        self
    }

    /// Check if path matches fetch spec
    pub fn includes_path(&self, path: &str) -> bool {
        match &self.paths {
            None => true, // Include all paths
            Some(paths) => {
                // Check if path starts with any included path
                paths.iter().any(|p| path.starts_with(p))
            }
        }
    }

    /// Check if file size should be excluded
    pub fn should_fetch_file(&self, size_bytes: u64) -> bool {
        match self.exclude_files_over_mb {
            None => true,
            Some(max_mb) => size_bytes <= (max_mb as u64 * 1024 * 1024),
        }
    }
}

/// Fetch statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchStats {
    /// Total objects fetched
    pub objects_fetched: u64,
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Paths included in fetch
    pub paths_included: Vec<String>,
    /// Number of commits fetched
    pub commits_fetched: u64,
}

impl FetchStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            objects_fetched: 0,
            bytes_transferred: 0,
            paths_included: vec![],
            commits_fetched: 0,
        }
    }

    /// Format bytes for display
    pub fn formatted_size(&self) -> String {
        let mb = self.bytes_transferred as f64 / (1024.0 * 1024.0);
        if mb > 1024.0 {
            format!("{:.2} GB", mb / 1024.0)
        } else {
            format!("{:.2} MB", mb)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_spec_branch() {
        let spec = FetchSpec::branch("main");
        assert_eq!(spec.branch, "main");
        assert!(spec.paths.is_none());
        assert!(spec.includes_path("anything"));
    }

    #[test]
    fn test_fetch_spec_paths() {
        let spec = FetchSpec::paths("main", &["src", "lib"]);
        assert!(spec.includes_path("src/main.rs"));
        assert!(spec.includes_path("lib/utils.rs"));
        assert!(!spec.includes_path("docs/readme.md"));
    }

    #[test]
    fn test_fetch_spec_with_depth() {
        let spec = FetchSpec::branch("main").with_depth(10);
        assert_eq!(spec.depth, Some(10));
    }

    #[test]
    fn test_fetch_spec_exclude_large_files() {
        let spec = FetchSpec::branch("main").exclude_large_files(50);
        assert!(spec.should_fetch_file(10 * 1024 * 1024)); // 10 MB
        assert!(!spec.should_fetch_file(100 * 1024 * 1024)); // 100 MB
    }

    #[test]
    fn test_fetch_stats_formatting() {
        let mut stats = FetchStats::new();
        stats.bytes_transferred = 500 * 1024 * 1024; // 500 MB
        assert!(stats.formatted_size().contains("MB"));

        stats.bytes_transferred = 2 * 1024 * 1024 * 1024; // 2 GB
        assert!(stats.formatted_size().contains("GB"));
    }
}

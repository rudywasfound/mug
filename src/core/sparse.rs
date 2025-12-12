/// Sparse checkout support for large monorepos
/// Allows cloning/checking out only specific directories

use crate::core::error::{Error, Result};
use crate::core::repo::Repository;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Sparse checkout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseConfig {
    /// Patterns to include (glob patterns)
    pub includes: Vec<String>,
    /// Patterns to exclude (glob patterns)
    pub excludes: Vec<String>,
    /// Whether to use cone mode (directory-based, faster)
    pub cone_mode: bool,
}

impl Default for SparseConfig {
    fn default() -> Self {
        Self {
            includes: vec!["*".to_string()],
            excludes: vec![],
            cone_mode: true,
        }
    }
}

impl SparseConfig {
    /// Create a sparse config for monorepo subset
    pub fn for_monorepo(paths: &[&str]) -> Self {
        Self {
            includes: paths.iter().map(|s| format!("{}/**", s)).collect(),
            excludes: vec![],
            cone_mode: true,
        }
    }

    /// Add include pattern
    pub fn add_include(&mut self, pattern: String) {
        self.includes.push(pattern);
    }

    /// Add exclude pattern
    pub fn add_exclude(&mut self, pattern: String) {
        self.excludes.push(pattern);
    }

    /// Save to .mug/sparse-checkout
    pub fn save(&self, repo: &Repository) -> Result<()> {
        let sparse_file = repo.root_path().join(".mug/sparse-checkout");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Custom(format!("Failed to serialize sparse config: {}", e)))?;
        fs::write(sparse_file, content)?;
        Ok(())
    }

    /// Load from .mug/sparse-checkout
    pub fn load(repo: &Repository) -> Result<Option<Self>> {
        let sparse_file = repo.root_path().join(".mug/sparse-checkout");
        if !sparse_file.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(sparse_file)?;
        let config = serde_json::from_str(&content)
            .map_err(|e| Error::Custom(format!("Failed to parse sparse config: {}", e)))?;
        Ok(Some(config))
    }
}

/// Sparse checkout manager
pub struct SparseCheckout {
    config: SparseConfig,
    repo: Repository,
}

impl SparseCheckout {
    /// Create new sparse checkout manager
    pub fn new(repo: Repository, config: SparseConfig) -> Self {
        Self { config, repo }
    }

    /// Check if path should be included in checkout
    pub fn should_include(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check excludes first (they take precedence)
        for exclude in &self.config.excludes {
            if self.matches_pattern(&path_str, exclude) {
                return false;
            }
        }

        // Check includes
        for include in &self.config.includes {
            if self.matches_pattern(&path_str, include) {
                return true;
            }
        }

        false
    }

    /// Simple glob pattern matching
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Handle common patterns
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with("/**") {
            // Match directory and all contents
            let dir = pattern.trim_end_matches("/**");
            return path.starts_with(dir);
        }

        if pattern.contains('*') {
            // Simple wildcard matching
            let pattern = pattern.replace("*", ".*");
            if let Ok(re) = regex::Regex::new(&format!("^{}$", pattern)) {
                return re.is_match(path);
            }
        }

        path == pattern
    }

    /// Apply sparse checkout - removes files not in sparse config
    pub fn apply(&self) -> Result<()> {
        let mut to_remove = Vec::new();

        // Find files to remove
        for entry in walkdir::WalkDir::new(self.repo.root_path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip .mug directory
            if path.components().any(|c| c.as_os_str() == ".mug") {
                continue;
            }

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Check if should be removed
            if !self.should_include(path) {
                to_remove.push(path.to_path_buf());
            }
        }

        // Remove files not in sparse config
        for path in to_remove {
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("Warning: Failed to remove {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Get current sparse config
    pub fn config(&self) -> &SparseConfig {
        &self.config
    }

    /// Update sparse config
    pub fn set_config(&mut self, config: SparseConfig) -> Result<()> {
        self.config = config;
        self.config.save(&self.repo)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_config_default() {
        let config = SparseConfig::default();
        assert_eq!(config.includes, vec!["*"]);
        assert!(config.excludes.is_empty());
        assert!(config.cone_mode);
    }

    #[test]
    fn test_sparse_config_monorepo() {
        let config = SparseConfig::for_monorepo(&["services", "libs"]);
        assert_eq!(config.includes, vec!["services/**", "libs/**"]);
        assert!(config.cone_mode);
    }

    #[test]
    fn test_pattern_matching() {
        let repo = Repository::open(".").unwrap_or_else(|_| {
            Repository::init(".mug_test").expect("Failed to create test repo")
        });
        let checkout = SparseCheckout::new(
            repo,
            SparseConfig::for_monorepo(&["src"]),
        );

        assert!(checkout.matches_pattern("src/main.rs", "src/**"));
        assert!(checkout.matches_pattern("src/lib/mod.rs", "src/**"));
        assert!(!checkout.matches_pattern("docs/readme.md", "src/**"));
    }
}

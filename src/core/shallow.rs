/// Shallow clone support for efficient partial history fetching
/// Allows cloning only recent commits instead of full history

use crate::core::error::{Error, Result};
use crate::core::repo::Repository;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Shallow clone configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShallowConfig {
    /// Maximum depth from HEAD (None = full history)
    pub depth: Option<u32>,
    /// Commit hash at shallow boundary
    pub shallow_commit: Option<String>,
    /// Whether this is a shallow clone
    pub is_shallow: bool,
}

impl Default for ShallowConfig {
    fn default() -> Self {
        Self {
            depth: None,
            shallow_commit: None,
            is_shallow: false,
        }
    }
}

impl ShallowConfig {
    /// Create shallow config with depth
    pub fn with_depth(depth: u32) -> Self {
        Self {
            depth: Some(depth),
            shallow_commit: None,
            is_shallow: true,
        }
    }

    /// Save to .mug/shallow
    pub fn save(&self, repo: &Repository) -> Result<()> {
        let shallow_file = repo.root_path().join(".mug/shallow");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Custom(format!("Failed to serialize shallow config: {}", e)))?;
        fs::write(shallow_file, content)?;
        Ok(())
    }

    /// Load from .mug/shallow
    pub fn load(repo: &Repository) -> Result<Option<Self>> {
        let shallow_file = repo.root_path().join(".mug/shallow");
        if !shallow_file.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(shallow_file)?;
        let config = serde_json::from_str(&content)
            .map_err(|e| Error::Custom(format!("Failed to parse shallow config: {}", e)))?;
        Ok(Some(config))
    }
}

/// Shallow clone manager
pub struct ShallowClone {
    config: ShallowConfig,
}

impl ShallowClone {
    /// Create new shallow clone manager
    pub fn new(config: ShallowConfig) -> Self {
        Self { config }
    }

    /// Create shallow clone with specified depth
    pub fn shallow_clone(repo: &Repository, depth: u32, _branch: &str) -> Result<ShallowConfig> {
        // Get commit log and truncate to depth
        let log = repo.log()?;
        let shallow_commit = log
            .get(depth.saturating_sub(1) as usize)
            .and_then(|l| l.lines().next())
            .map(|s| s.to_string());

        let config = ShallowConfig {
            depth: Some(depth),
            shallow_commit,
            is_shallow: true,
        };

        config.save(repo)?;
        Ok(config)
    }

    /// Unshallow - convert shallow clone to full clone
    pub fn unshallow(repo: &Repository) -> Result<()> {
        let shallow_file = repo.root_path().join(".mug/shallow");
        if shallow_file.exists() {
            fs::remove_file(shallow_file)?;
        }

        let config = ShallowConfig {
            depth: None,
            shallow_commit: None,
            is_shallow: false,
        };

        config.save(repo)?;
        Ok(())
    }

    /// Get depth limit
    pub fn depth(&self) -> Option<u32> {
        self.config.depth
    }

    /// Check if clone is shallow
    pub fn is_shallow(&self) -> bool {
        self.config.is_shallow
    }

    /// Get shallow commit boundary
    pub fn shallow_commit(&self) -> Option<&str> {
        self.config.shallow_commit.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shallow_config_default() {
        let config = ShallowConfig::default();
        assert!(config.depth.is_none());
        assert!(!config.is_shallow);
    }

    #[test]
    fn test_shallow_config_with_depth() {
        let config = ShallowConfig::with_depth(10);
        assert_eq!(config.depth, Some(10));
        assert!(config.is_shallow);
    }

    #[test]
    fn test_shallow_clone_manager() {
        let config = ShallowConfig::with_depth(5);
        let shallow = ShallowClone::new(config);

        assert!(shallow.is_shallow());
        assert_eq!(shallow.depth(), Some(5));
    }
}

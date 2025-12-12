/// Workspace mapping for monorepo support
/// Maps remote depot paths to local client paths (Perforce-style)

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Workspace view mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewMapping {
    /// Remote depot path pattern
    pub depot_path: String,
    /// Local client path pattern
    pub client_path: String,
    /// Exclude pattern (optional)
    pub exclude: Option<String>,
}

impl ViewMapping {
    /// Create new view mapping
    pub fn new(depot: &str, client: &str) -> Self {
        Self {
            depot_path: depot.to_string(),
            client_path: client.to_string(),
            exclude: None,
        }
    }

    /// Add exclusion pattern
    pub fn with_exclude(mut self, pattern: &str) -> Self {
        self.exclude = Some(pattern.to_string());
        self
    }

    /// Check if depot path matches this mapping
    pub fn matches_depot(&self, path: &str) -> bool {
        self.depot_path_matches(&self.depot_path, path)
    }

    /// Map depot path to client path
    pub fn map_to_client(&self, depot_path: &str) -> Option<PathBuf> {
        if !self.matches_depot(depot_path) {
            return None;
        }

        // Simple path mapping: replace depot prefix with client prefix
        let depot_prefix = self.depot_path.trim_end_matches("...");
        if depot_path.starts_with(depot_prefix) {
            let remainder = &depot_path[depot_prefix.len()..];
            let client_prefix = self.client_path.trim_end_matches("...");
            return Some(PathBuf::from(format!("{}{}", client_prefix, remainder)));
        }

        None
    }

    /// Check if path should be excluded
    pub fn is_excluded(&self, path: &str) -> bool {
        if let Some(exclude) = &self.exclude {
            // Extract filename for wildcard matching
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path);
            return self.matches_pattern(filename, exclude);
        }
        false
    }

    // Helper: match depot path pattern
    fn depot_path_matches(&self, pattern: &str, path: &str) -> bool {
        if pattern.ends_with("...") {
            let prefix = pattern.trim_end_matches("...");
            return path.starts_with(prefix);
        }
        pattern == path
    }

    // Helper: simple pattern matching
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.starts_with("*.") {
            // Match file extensions like *.tmp
            let ext = pattern.trim_start_matches('*');
            return path.ends_with(ext);
        }
        if pattern.ends_with("*") {
            return path.starts_with(pattern.trim_end_matches('*'));
        }
        path == pattern
    }
}

/// Workspace configuration (Perforce-style client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace name
    pub name: String,
    /// Root directory of workspace
    pub root: PathBuf,
    /// View mappings (depot paths -> local paths)
    pub view: Vec<ViewMapping>,
    /// Metadata
    pub description: Option<String>,
    pub owner: Option<String>,
}

impl Workspace {
    /// Create new workspace
    pub fn new(name: &str, root: &Path) -> Self {
        Self {
            name: name.to_string(),
            root: root.to_path_buf(),
            view: vec![],
            description: None,
            owner: None,
        }
    }

    /// Add view mapping
    pub fn add_view(&mut self, mapping: ViewMapping) {
        self.view.push(mapping);
    }

    /// Get all depot paths that should be synced to this workspace
    pub fn get_depot_paths(&self) -> Vec<String> {
        self.view
            .iter()
            .map(|v| v.depot_path.clone())
            .collect()
    }

    /// Map depot path to local client path
    pub fn map_to_local(&self, depot_path: &str) -> Option<PathBuf> {
        for mapping in &self.view {
            if let Some(local) = mapping.map_to_client(depot_path) {
                if !mapping.is_excluded(depot_path) {
                    return Some(local);
                }
            }
        }
        None
    }

    /// Check if depot path is in this workspace's view
    pub fn includes_path(&self, depot_path: &str) -> bool {
        for mapping in &self.view {
            if mapping.matches_depot(depot_path) && !mapping.is_excluded(depot_path) {
                return true;
            }
        }
        false
    }

    /// Save to .mug/workspace.json
    pub fn save(&self) -> Result<()> {
        let mug_dir = self.root.join(".mug");
        fs::create_dir_all(&mug_dir)?;

        let config_path = mug_dir.join("workspace.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Custom(format!("Failed to serialize workspace: {}", e)))?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// Load from .mug/workspace.json
    pub fn load(root: &Path) -> Result<Option<Self>> {
        let config_path = root.join(".mug/workspace.json");
        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(config_path)?;
        let workspace = serde_json::from_str(&content)
            .map_err(|e| Error::Custom(format!("Failed to parse workspace: {}", e)))?;
        Ok(Some(workspace))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_mapping_basic() {
        let mapping = ViewMapping::new("//depot/main/...", "//client/main/...");
        assert!(mapping.matches_depot("//depot/main/src/file.rs"));
        assert!(!mapping.matches_depot("//depot/other/file.rs"));
    }

    #[test]
    fn test_map_to_client() {
        let mapping = ViewMapping::new("//depot/src/...", "//client/src/...");
        let result = mapping.map_to_client("//depot/src/main.rs");
        assert_eq!(result, Some(PathBuf::from("//client/src/main.rs")));
    }

    #[test]
    fn test_exclude_pattern() {
        let mapping = ViewMapping::new("//depot/...", "//client/...")
            .with_exclude("*.tmp");

        assert!(!mapping.is_excluded("file.rs"));
        assert!(mapping.is_excluded("file.tmp"));
    }

    #[test]
    fn test_workspace_multiple_views() {
        let mut ws = Workspace::new("test", Path::new("/workspace"));
        ws.add_view(ViewMapping::new("//depot/services/...", "//client/services/..."));
        ws.add_view(ViewMapping::new("//depot/libs/...", "//client/libs/..."));

        assert!(ws.includes_path("//depot/services/api/main.rs"));
        assert!(ws.includes_path("//depot/libs/utils/main.rs"));
        assert!(!ws.includes_path("//depot/docs/readme.md"));
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::error::Result;

/// Repository configuration manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// User name for commits
    pub user_name: Option<String>,
    /// User email for commits
    pub user_email: Option<String>,
    /// Default branch name
    pub default_branch: Option<String>,
    /// Custom settings
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

impl Config {
    /// Creates a new empty configuration
    pub fn new() -> Self {
        Config {
            user_name: None,
            user_email: None,
            default_branch: Some("main".to_string()),
            custom: HashMap::new(),
        }
    }

    /// Loads configuration from .mug/config.json
    pub fn load(repo_root: &Path) -> Result<Self> {
        let config_path = repo_root.join(".mug").join("config.json");

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content)
                .map_err(|e| crate::core::error::Error::Custom(format!("Failed to parse config: {}", e)))
        } else {
            Ok(Config::new())
        }
    }

    /// Saves configuration to .mug/config.json
    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let config_path = repo_root.join(".mug").join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Sets user name
    pub fn set_user_name(&mut self, name: String) {
        self.user_name = Some(name);
    }

    /// Sets user email
    pub fn set_user_email(&mut self, email: String) {
        self.user_email = Some(email);
    }

    /// Sets default branch
    pub fn set_default_branch(&mut self, branch: String) {
        self.default_branch = Some(branch);
    }

    /// Sets a custom configuration value
    pub fn set(&mut self, key: String, value: String) {
        self.custom.insert(key, value);
    }

    /// Gets a custom configuration value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }

    /// Gets user name or uses default
    pub fn get_user_name(&self) -> String {
        self.user_name
            .clone()
            .unwrap_or_else(|| "MUG User".to_string())
    }

    /// Gets user email or uses default
    pub fn get_user_email(&self) -> String {
        self.user_email
            .clone()
            .unwrap_or_else(|| "user@local.mug".to_string())
    }

    /// Gets default branch
    pub fn get_default_branch(&self) -> String {
        self.default_branch
            .clone()
            .unwrap_or_else(|| "main".to_string())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_defaults() {
        let config = Config::new();
        assert_eq!(config.get_user_name(), "MUG User");
        assert_eq!(config.get_user_email(), "user@local.mug");
        assert_eq!(config.get_default_branch(), "main");
    }

    #[test]
    fn test_config_set_values() {
        let mut config = Config::new();
        config.set_user_name("John Doe".to_string());
        config.set_user_email("john@example.com".to_string());

        assert_eq!(config.get_user_name(), "John Doe");
        assert_eq!(config.get_user_email(), "john@example.com");
    }

    #[test]
    fn test_config_custom_values() {
        let mut config = Config::new();
        config.set("key1".to_string(), "value1".to_string());
        config.set("key2".to_string(), "value2".to_string());

        assert_eq!(config.get("key1"), Some(&"value1".to_string()));
        assert_eq!(config.get("key2"), Some(&"value2".to_string()));
        assert_eq!(config.get("key3"), None);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = TempDir::new().unwrap();
        let mug_dir = dir.path().join(".mug");
        fs::create_dir_all(&mug_dir).unwrap();

        let mut config = Config::new();
        config.set_user_name("Jane Doe".to_string());
        config.set_user_email("jane@example.com".to_string());
        config.save(dir.path()).unwrap();

        let loaded = Config::load(dir.path()).unwrap();
        assert_eq!(loaded.get_user_name(), "Jane Doe");
        assert_eq!(loaded.get_user_email(), "jane@example.com");
    }
}

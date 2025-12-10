use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::database::MugDb;

/// Remote configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub protocol: Protocol,
    pub fetch: bool,
    pub push: bool,
}

/// Protocol type for remote
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    Http,
    Https,
    Ssh,
}

impl Protocol {
    pub fn from_url(url: &str) -> Self {
        if url.starts_with("https://") {
            Protocol::Https
        } else if url.starts_with("http://") {
            Protocol::Http
        } else if url.contains("@") || url.starts_with("ssh://") {
            Protocol::Ssh
        } else {
            // Default to HTTPS for safety
            Protocol::Https
        }
    }
}

/// Remote manager - handles remote configuration
pub struct RemoteManager {
    db: MugDb,
}

impl RemoteManager {
    pub fn new(db: MugDb) -> Self {
        Self { db }
    }

    /// Add a new remote
    pub fn add(&self, name: &str, url: &str) -> Result<()> {
        if self.get(name)?.is_some() {
            return Err(crate::error::Error::Custom(format!(
                "Remote '{}' already exists",
                name
            )));
        }

        let protocol = Protocol::from_url(url);

        let remote = Remote {
            name: name.to_string(),
            url: url.to_string(),
            protocol,
            fetch: true,
            push: true,
        };

        let serialized = serde_json::to_vec(&remote)?;
        self.db.set("remotes", name, serialized)?;
        Ok(())
    }

    /// Remove a remote
    pub fn remove(&self, name: &str) -> Result<()> {
        self.db.delete("remotes", name)?;
        Ok(())
    }

    /// Get a remote by name
    pub fn get(&self, name: &str) -> Result<Option<Remote>> {
        match self.db.get("remotes", name)? {
            Some(data) => {
                let remote: Remote = serde_json::from_slice(&data)?;
                Ok(Some(remote))
            }
            None => Ok(None),
        }
    }

    /// List all remotes
    pub fn list(&self) -> Result<Vec<Remote>> {
        let entries = self.db.scan("remotes", "")?;
        let mut remotes = Vec::new();

        for (_, value) in entries {
            if let Ok(remote) = serde_json::from_slice::<Remote>(&value) {
                remotes.push(remote);
            }
        }

        remotes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(remotes)
    }

    /// Update a remote URL
    pub fn update_url(&self, name: &str, new_url: &str) -> Result<()> {
        let mut remote = self.get(name)?
            .ok_or_else(|| crate::error::Error::Custom(format!("Remote '{}' not found", name)))?;

        remote.url = new_url.to_string();
        remote.protocol = Protocol::from_url(new_url);

        let serialized = serde_json::to_vec(&remote)?;
        self.db.set("remotes", name, serialized)?;
        Ok(())
    }

    /// Set whether a remote supports fetch
    pub fn set_fetch(&self, name: &str, enabled: bool) -> Result<()> {
        let mut remote = self.get(name)?
            .ok_or_else(|| crate::error::Error::Custom(format!("Remote '{}' not found", name)))?;

        remote.fetch = enabled;

        let serialized = serde_json::to_vec(&remote)?;
        self.db.set("remotes", name, serialized)?;
        Ok(())
    }

    /// Set whether a remote supports push
    pub fn set_push(&self, name: &str, enabled: bool) -> Result<()> {
        let mut remote = self.get(name)?
            .ok_or_else(|| crate::error::Error::Custom(format!("Remote '{}' not found", name)))?;

        remote.push = enabled;

        let serialized = serde_json::to_vec(&remote)?;
        self.db.set("remotes", name, serialized)?;
        Ok(())
    }

    /// Set default remote (origin)
    pub fn set_default(&self, name: &str) -> Result<()> {
        if self.get(name)?.is_none() {
            return Err(crate::error::Error::Custom(format!("Remote '{}' not found", name)));
        }

        self.db.set("config", "default_remote", name.as_bytes())?;
        Ok(())
    }

    /// Get default remote
    pub fn get_default(&self) -> Result<Option<String>> {
        match self.db.get("config", "default_remote")? {
            Some(data) => Ok(Some(String::from_utf8(data)?)),
            None => Ok(None),
        }
    }

    /// Check if a remote supports fetch
    pub fn can_fetch(&self, name: &str) -> Result<bool> {
        Ok(self
            .get(name)?
            .map(|r| r.fetch)
            .unwrap_or(false))
    }

    /// Check if a remote supports push
    pub fn can_push(&self, name: &str) -> Result<bool> {
        Ok(self
            .get(name)?
            .map(|r| r.push)
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_protocol_detection() {
        assert_eq!(Protocol::from_url("https://github.com/user/repo"), Protocol::Https);
        assert_eq!(Protocol::from_url("http://localhost:3000/repo"), Protocol::Http);
        assert_eq!(Protocol::from_url("git@github.com:user/repo"), Protocol::Ssh);
        assert_eq!(Protocol::from_url("ssh://git@github.com/user/repo"), Protocol::Ssh);
    }

    #[test]
    fn test_remote_manager_add() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = RemoteManager::new(db);

        manager
            .add("origin", "https://github.com/user/repo.git")
            .unwrap();

        let remote = manager.get("origin").unwrap();
        assert!(remote.is_some());
        assert_eq!(remote.unwrap().url, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_remote_manager_duplicate_error() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = RemoteManager::new(db);

        manager
            .add("origin", "https://github.com/user/repo.git")
            .unwrap();

        let result = manager.add("origin", "https://github.com/other/repo.git");
        assert!(result.is_err());
    }

    #[test]
    fn test_remote_manager_list() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = RemoteManager::new(db);

        manager
            .add("origin", "https://github.com/user/repo.git")
            .unwrap();
        manager
            .add("upstream", "https://github.com/original/repo.git")
            .unwrap();

        let remotes = manager.list().unwrap();
        assert_eq!(remotes.len(), 2);
        assert_eq!(remotes[0].name, "origin");
        assert_eq!(remotes[1].name, "upstream");
    }

    #[test]
    fn test_remote_manager_update_url() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = RemoteManager::new(db);

        manager
            .add("origin", "https://github.com/user/repo.git")
            .unwrap();
        manager.update_url("origin", "https://github.com/newuser/repo.git").unwrap();

        let remote = manager.get("origin").unwrap().unwrap();
        assert_eq!(remote.url, "https://github.com/newuser/repo.git");
    }

    #[test]
    fn test_remote_manager_default() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = RemoteManager::new(db);

        manager
            .add("origin", "https://github.com/user/repo.git")
            .unwrap();
        manager.set_default("origin").unwrap();

        let default = manager.get_default().unwrap();
        assert_eq!(default, Some("origin".to_string()));
    }

    #[test]
    fn test_remote_manager_remove() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = RemoteManager::new(db);

        manager
            .add("origin", "https://github.com/user/repo.git")
            .unwrap();
        manager.remove("origin").unwrap();

        let remote = manager.get("origin").unwrap();
        assert!(remote.is_none());
    }

    #[test]
    fn test_remote_manager_fetch_push() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = RemoteManager::new(db);

        manager
            .add("origin", "https://github.com/user/repo.git")
            .unwrap();

        assert!(manager.can_fetch("origin").unwrap());
        assert!(manager.can_push("origin").unwrap());

        manager.set_fetch("origin", false).unwrap();
        assert!(!manager.can_fetch("origin").unwrap());

        manager.set_push("origin", false).unwrap();
        assert!(!manager.can_push("origin").unwrap());
    }
}

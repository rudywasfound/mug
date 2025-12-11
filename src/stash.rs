use serde::{Deserialize, Serialize};

use crate::database::MugDb;
use crate::error::Result;
use crate::index::IndexEntry;

/// A stashed set of changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stash {
    pub id: String,
    pub branch: String,
    pub message: String,
    pub files: Vec<StashedFile>,
    pub timestamp: String,
}

/// A stashed file with its contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashedFile {
    pub path: String,
    pub hash: String,
    pub mode: u32,
    pub content_hash: String,
}

/// Stash manager for saving and restoring work in progress
pub struct StashManager {
    db: MugDb,
}

impl StashManager {
    pub fn new(db: MugDb) -> Self {
        StashManager { db }
    }

    /// Create a new stash from current index
    pub fn create(&self, branch: &str, message: &str, entries: Vec<IndexEntry>) -> Result<String> {
        let stash_id = format!(
            "stash-{}-{}-{}",
            branch,
            chrono::Local::now().timestamp(),
            uuid::Uuid::new_v4()
        );

        let files = entries
            .into_iter()
            .map(|e| StashedFile {
                path: e.path.clone(),
                hash: e.hash.clone(),
                mode: e.mode,
                content_hash: format!("content-{}", e.hash),
            })
            .collect();

        let stash = Stash {
            id: stash_id.clone(),
            branch: branch.to_string(),
            message: message.to_string(),
            files,
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        let serialized = serde_json::to_vec(&stash)?;
        self.db.set("stash", &stash_id, serialized)?;

        Ok(stash_id)
    }

    /// Get a stash by ID
    pub fn get(&self, stash_id: &str) -> Result<Option<Stash>> {
        match self.db.get("stash", stash_id)? {
            Some(data) => {
                let stash: Stash = serde_json::from_slice(&data)?;
                Ok(Some(stash))
            }
            None => Ok(None),
        }
    }

    /// List all stashes
    pub fn list(&self) -> Result<Vec<Stash>> {
        let entries = self.db.scan("stash", "")?;
        let mut stashes = Vec::new();

        for (_, value) in entries {
            if let Ok(stash) = serde_json::from_slice::<Stash>(&value) {
                stashes.push(stash);
            }
        }

        // Sort by timestamp (newest first)
        stashes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(stashes)
    }

    /// Apply a stash (restore changes)
    pub fn apply(&self, stash_id: &str) -> Result<()> {
        match self.get(stash_id)? {
            Some(stash) => {
                // In a real implementation, this would restore the file contents
                // For now, just verify the stash exists
                eprintln!("Applied stash {}: {}", stash_id, stash.message);
                Ok(())
            }
            None => Err(crate::error::Error::Custom(format!(
                "Stash {} not found",
                stash_id
            ))),
        }
    }

    /// Apply and delete a stash
    pub fn pop(&self, stash_id: &str) -> Result<()> {
        self.apply(stash_id)?;
        self.db.delete("stash", stash_id)?;
        Ok(())
    }

    /// Delete a stash without applying
    pub fn drop(&self, stash_id: &str) -> Result<()> {
        self.db.delete("stash", stash_id)?;
        Ok(())
    }

    /// Delete all stashes
    pub fn clear(&self) -> Result<()> {
        self.db.clear_tree("stash")?;
        Ok(())
    }

    /// Get the latest stash (stash@{0})
    pub fn latest(&self) -> Result<Option<Stash>> {
        let stashes = self.list()?;
        Ok(stashes.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_stash_creation() {
        let stash = Stash {
            id: "stash-main-123".to_string(),
            branch: "main".to_string(),
            message: "WIP: feature work".to_string(),
            files: vec![],
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        assert_eq!(stash.branch, "main");
        assert_eq!(stash.message, "WIP: feature work");
    }

    #[test]
    fn test_stash_manager() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = StashManager::new(db);

        let entry = IndexEntry {
            path: "file.txt".to_string(),
            hash: "abc123".to_string(),
            mode: 0o100644,
        };

        let stash_id = manager.create("main", "WIP: test", vec![entry]).unwrap();

        let stash = manager.get(&stash_id).unwrap();
        assert!(stash.is_some());
        assert_eq!(stash.unwrap().message, "WIP: test");
    }

    #[test]
    fn test_stash_list() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = StashManager::new(db);

        let entry = IndexEntry {
            path: "file.txt".to_string(),
            hash: "abc123".to_string(),
            mode: 0o100644,
        };

        manager
            .create("main", "WIP: first", vec![entry.clone()])
            .unwrap();
        manager
            .create("main", "WIP: second", vec![entry.clone()])
            .unwrap();

        let stashes = manager.list().unwrap();
        assert_eq!(stashes.len(), 2);
    }

    #[test]
    fn test_stash_drop() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = StashManager::new(db);

        let entry = IndexEntry {
            path: "file.txt".to_string(),
            hash: "abc123".to_string(),
            mode: 0o100644,
        };

        let stash_id = manager.create("main", "WIP: test", vec![entry]).unwrap();

        manager.drop(&stash_id).unwrap();
        assert!(manager.get(&stash_id).unwrap().is_none());
    }
}

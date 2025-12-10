use serde::{Serialize, Deserialize};
use crate::database::MugDb;
use crate::error::Result;

/// A branch reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchRef {
    pub name: String,
    pub commit_id: String,
}

pub struct BranchManager {
    db: MugDb,
}

impl BranchManager {
    pub fn new(db: MugDb) -> Self {
        BranchManager { db }
    }

    /// Create a new branch pointing to a commit
    pub fn create_branch(&self, name: String, commit_id: String) -> Result<()> {
        let branch = BranchRef {
            name: name.clone(),
            commit_id,
        };
        let serialized = serde_json::to_vec(&branch)?;
        self.db.set("BRANCHES", &name, serialized)?;
        Ok(())
    }

    /// Get a branch by name
    pub fn get_branch(&self, name: &str) -> Result<Option<BranchRef>> {
        match self.db.get("BRANCHES", name)? {
            Some(data) => Ok(Some(serde_json::from_slice(&data)?)),
            None => Ok(None),
        }
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &str) -> Result<()> {
        self.db.delete("BRANCHES", name)?;
        Ok(())
    }

    /// List all branches
    pub fn list_branches(&self) -> Result<Vec<BranchRef>> {
        let entries = self.db.scan("BRANCHES", "")?;
        let mut branches = Vec::new();
        for (_name, data) in entries {
            if let Ok(branch) = serde_json::from_slice::<BranchRef>(&data) {
                branches.push(branch);
            }
        }
        Ok(branches)
    }

    /// Update a branch to point to a different commit
    pub fn update_branch(&self, name: &str, commit_id: String) -> Result<()> {
        let branch = BranchRef {
            name: name.to_string(),
            commit_id,
        };
        let serialized = serde_json::to_vec(&branch)?;
        self.db.set("BRANCHES", name, serialized)?;
        Ok(())
    }

    /// Get the HEAD reference
    pub fn get_head(&self) -> Result<Option<String>> {
        match self.db.get("HEAD", "HEAD")? {
            Some(data) => Ok(Some(String::from_utf8_lossy(&data).to_string())),
            None => Ok(None),
        }
    }

    /// Set the HEAD reference
    pub fn set_head(&self, ref_name: String) -> Result<()> {
        self.db.set("HEAD", "HEAD", ref_name)?;
        Ok(())
    }

    /// Detach HEAD to a specific commit
    pub fn detach_head(&self, commit_id: String) -> Result<()> {
        let detached_marker = format!("detached:{}", commit_id);
        self.db.set("HEAD", "HEAD", detached_marker)?;
        Ok(())
    }

    /// Check if HEAD is detached
    pub fn is_detached_head(&self) -> Result<bool> {
        match self.get_head()? {
            Some(head) => Ok(head.starts_with("detached:")),
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_get_branch() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = BranchManager::new(db);

        manager.create_branch("main".to_string(), "commit123".to_string()).unwrap();
        let branch = manager.get_branch("main").unwrap();
        assert!(branch.is_some());
        assert_eq!(branch.unwrap().commit_id, "commit123");
    }

    #[test]
    fn test_list_branches() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = BranchManager::new(db);

        manager.create_branch("main".to_string(), "commit1".to_string()).unwrap();
        manager.create_branch("dev".to_string(), "commit2".to_string()).unwrap();

        let branches = manager.list_branches().unwrap();
        assert_eq!(branches.len(), 2);
    }

    #[test]
    fn test_head_management() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = BranchManager::new(db);

        manager.set_head("main".to_string()).unwrap();
        assert_eq!(manager.get_head().unwrap(), Some("main".to_string()));
    }
}

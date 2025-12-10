use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::MugDb;
use crate::error::Result;

/// A commit in MUG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub tree_hash: String,
    pub parent: Option<String>,
    pub author: String,
    pub message: String,
    pub timestamp: String,
}

/// Commit metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMetadata {
    pub id: String,
    pub tree_hash: String,
    pub parent: Option<String>,
    pub author: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

pub struct CommitLog {
    db: MugDb,
}

impl CommitLog {
    pub fn new(db: MugDb) -> Self {
        CommitLog { db }
    }

    /// Create a new commit
    pub fn create_commit(
        &self,
        tree_hash: String,
        author: String,
        message: String,
        parent: Option<String>,
    ) -> Result<String> {
        let commit_id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now();

        let commit = CommitMetadata {
            id: commit_id.clone(),
            tree_hash,
            parent,
            author,
            message,
            timestamp,
        };

        let serialized = serde_json::to_vec(&commit)?;
        self.db.set("COMMITS", &commit_id, serialized)?;

        Ok(commit_id)
    }

    /// Get a commit by ID
    pub fn get_commit(&self, id: &str) -> Result<CommitMetadata> {
        let data = self.db.get("COMMITS", id)?
            .ok_or(crate::error::Error::CommitNotFound(id.to_string()))?;
        Ok(serde_json::from_slice(&data)?)
    }

    /// Get all commits in history (from head to root)
    pub fn history(&self, start_id: String) -> Result<Vec<CommitMetadata>> {
        let mut history = Vec::new();
        let mut current_id = Some(start_id);

        while let Some(id) = current_id {
            let commit = self.get_commit(&id)?;
            current_id = commit.parent.clone();
            history.push(commit);
        }

        Ok(history)
    }

    /// Get the parent of a commit
    pub fn parent(&self, id: &str) -> Result<Option<CommitMetadata>> {
        let commit = self.get_commit(id)?;
        if let Some(parent_id) = commit.parent {
            Ok(Some(self.get_commit(&parent_id)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_get_commit() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let log = CommitLog::new(db);

        let commit_id = log.create_commit(
            "tree123".to_string(),
            "Test User".to_string(),
            "Initial commit".to_string(),
            None,
        ).unwrap();

        let commit = log.get_commit(&commit_id).unwrap();
        assert_eq!(commit.message, "Initial commit");
        assert_eq!(commit.parent, None);
    }

    #[test]
    fn test_commit_history() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let log = CommitLog::new(db);

        let id1 = log.create_commit(
            "tree1".to_string(),
            "User".to_string(),
            "First".to_string(),
            None,
        ).unwrap();

        let id2 = log.create_commit(
            "tree2".to_string(),
            "User".to_string(),
            "Second".to_string(),
            Some(id1),
        ).unwrap();

        let history = log.history(id2).unwrap();
        assert_eq!(history.len(), 2);
    }
}

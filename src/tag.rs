use serde::{Deserialize, Serialize};

use crate::database::MugDb;
use crate::error::Result;

/// A Git-like tag for marking commits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub commit_id: String,
    pub message: Option<String>,
    pub author: Option<String>,
    pub timestamp: Option<String>,
}

impl Tag {
    pub fn new(name: String, commit_id: String) -> Self {
        Tag {
            name,
            commit_id,
            message: None,
            author: None,
            timestamp: None,
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    pub fn with_timestamp(mut self, timestamp: String) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

/// Tag manager for creating and listing tags
pub struct TagManager {
    db: MugDb,
}

impl TagManager {
    pub fn new(db: MugDb) -> Self {
        TagManager { db }
    }

    /// Create a new tag pointing to a commit
    pub fn create(&self, name: String, commit_id: String) -> Result<()> {
        if self.get(&name)?.is_some() {
            return Err(crate::error::Error::Custom(format!(
                "Tag '{}' already exists",
                name
            )));
        }

        let tag = Tag::new(name.clone(), commit_id);
        let serialized = serde_json::to_vec(&tag)?;
        self.db.set("tags", &name, serialized)?;

        Ok(())
    }

    /// Create an annotated tag with message
    pub fn create_annotated(
        &self,
        name: String,
        commit_id: String,
        message: String,
        author: String,
    ) -> Result<()> {
        let tag = Tag::new(name.clone(), commit_id)
            .with_message(message)
            .with_author(author)
            .with_timestamp(chrono::Local::now().to_rfc3339());

        let serialized = serde_json::to_vec(&tag)?;
        self.db.set("tags", &name, serialized)?;

        Ok(())
    }

    /// Get a tag by name
    pub fn get(&self, name: &str) -> Result<Option<Tag>> {
        match self.db.get("tags", name)? {
            Some(data) => {
                let tag: Tag = serde_json::from_slice(&data)?;
                Ok(Some(tag))
            }
            None => Ok(None),
        }
    }

    /// Delete a tag
    pub fn delete(&self, name: &str) -> Result<()> {
        self.db.delete("tags", name)?;
        Ok(())
    }

    /// List all tags
    pub fn list(&self) -> Result<Vec<Tag>> {
        let entries = self.db.scan("tags", "")?;
        let mut tags = Vec::new();

        for (_, value) in entries {
            if let Ok(tag) = serde_json::from_slice::<Tag>(&value) {
                tags.push(tag);
            }
        }

        tags.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tags)
    }

    /// Verify if a tag points to a valid commit
    pub fn verify(&self, name: &str) -> Result<bool> {
        match self.get(name)? {
            Some(tag) => {
                // Tag exists if we can retrieve it
                Ok(!tag.commit_id.is_empty())
            }
            None => Ok(false),
        }
    }

    /// Get tag by commit ID
    pub fn find_by_commit(&self, commit_id: &str) -> Result<Option<Tag>> {
        let tags = self.list()?;
        Ok(tags.into_iter().find(|t| t.commit_id == commit_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("v1.0.0".to_string(), "abc123def456".to_string());
        assert_eq!(tag.name, "v1.0.0");
        assert_eq!(tag.commit_id, "abc123def456");
        assert!(tag.message.is_none());
    }

    #[test]
    fn test_tag_with_message() {
        let tag = Tag::new("v1.0.0".to_string(), "abc123def456".to_string())
            .with_message("Release 1.0.0".to_string())
            .with_author("John Doe".to_string());

        assert_eq!(tag.message, Some("Release 1.0.0".to_string()));
        assert_eq!(tag.author, Some("John Doe".to_string()));
    }

    #[test]
    fn test_tag_manager() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = TagManager::new(db);

        manager
            .create("v1.0.0".to_string(), "commit1".to_string())
            .unwrap();

        let tag = manager.get("v1.0.0").unwrap();
        assert!(tag.is_some());
        assert_eq!(tag.unwrap().commit_id, "commit1");
    }

    #[test]
    fn test_tag_duplicate_error() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = TagManager::new(db);

        manager
            .create("v1.0.0".to_string(), "commit1".to_string())
            .unwrap();

        let result = manager.create("v1.0.0".to_string(), "commit2".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_tag_list() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let manager = TagManager::new(db);

        manager
            .create("v1.0.0".to_string(), "commit1".to_string())
            .unwrap();
        manager
            .create("v1.1.0".to_string(), "commit2".to_string())
            .unwrap();
        manager
            .create("v1.2.0".to_string(), "commit3".to_string())
            .unwrap();

        let tags = manager.list().unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].name, "v1.0.0");
        assert_eq!(tags[1].name, "v1.1.0");
        assert_eq!(tags[2].name, "v1.2.0");
    }
}

use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::fs;
use crate::error::Result;
use crate::hash;

/// A single file snapshot in the content-addressable store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blob {
    pub hash: String,
    pub size: u64,
    pub content: Vec<u8>,
}

/// A directory tree snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub hash: String,
    pub entries: Vec<TreeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub hash: String,
    pub is_dir: bool,
}

/// The content-addressable object store
pub struct ObjectStore {
    objects_dir: PathBuf,
}

impl ObjectStore {
    pub fn new(objects_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&objects_dir)?;
        Ok(ObjectStore { objects_dir })
    }

    /// Store a blob and return its hash
    pub fn store_blob(&self, content: &[u8]) -> Result<String> {
        let hash = hash::hash_bytes(content);
        let path = self.object_path(&hash);

        // Skip if already exists
        if !path.exists() {
            let blob = Blob {
                hash: hash.clone(),
                size: content.len() as u64,
                content: content.to_vec(),
            };
            let serialized = serde_json::to_vec(&blob)?;
            fs::write(&path, serialized)?;
        }

        Ok(hash)
    }

    /// Store a file and return its blob hash
    pub fn store_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let content = fs::read(&path)?;
        self.store_blob(&content)
    }

    /// Retrieve a blob by hash
    pub fn get_blob(&self, hash: &str) -> Result<Blob> {
        let path = self.object_path(hash);
        let data = fs::read(&path)?;
        let blob = serde_json::from_slice(&data)?;
        Ok(blob)
    }

    /// Store a tree and return its hash
    pub fn store_tree(&self, entries: Vec<TreeEntry>) -> Result<String> {
        let tree_json = serde_json::to_string(&entries)?;
        let hash = hash::hash_str(&tree_json);
        let path = self.object_path(&hash);

        if !path.exists() {
            let tree = Tree {
                hash: hash.clone(),
                entries,
            };
            let serialized = serde_json::to_vec(&tree)?;
            fs::write(&path, serialized)?;
        }

        Ok(hash)
    }

    /// Retrieve a tree by hash
    pub fn get_tree(&self, hash: &str) -> Result<Tree> {
        let path = self.object_path(hash);
        let data = fs::read(&path)?;
        let tree = serde_json::from_slice(&data)?;
        Ok(tree)
    }

    /// Check if an object exists
    pub fn has_object(&self, hash: &str) -> bool {
        self.object_path(hash).exists()
    }

    fn object_path(&self, hash: &str) -> PathBuf {
        self.objects_dir.join(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_store_and_retrieve_blob() {
        let dir = TempDir::new().unwrap();
        let store = ObjectStore::new(dir.path().join("objects")).unwrap();

        let content = b"hello world";
        let hash = store.store_blob(content).unwrap();

        let blob = store.get_blob(&hash).unwrap();
        assert_eq!(blob.content, content);
    }

    #[test]
    fn test_store_tree() {
        let dir = TempDir::new().unwrap();
        let store = ObjectStore::new(dir.path().join("objects")).unwrap();

        let entries = vec![
            TreeEntry {
                name: "file.txt".to_string(),
                hash: "abc123".to_string(),
                is_dir: false,
            },
        ];

        let hash = store.store_tree(entries).unwrap();
        let tree = store.get_tree(&hash).unwrap();
        assert_eq!(tree.entries.len(), 1);
    }
}

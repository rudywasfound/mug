use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::database::MugDb;
use crate::core::error::Result;

/// Represents a single entry in the git index (staging area)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexEntry {
    /// File path relative to repository root
    pub path: String,
    /// SHA-1 hash of file contents
    pub hash: String,
    /// File mode (e.g., 0o100644 for regular files, 0o100755 for executables)
    pub mode: u32,
}

/// Manages the git staging area (index) with persistence in the database
pub struct Index {
    db: MugDb,
    /// In-memory cache of index entries for quick access
    entries: HashMap<String, IndexEntry>,
}

impl Index {
    /// Creates or loads an existing index from the database
    pub fn new(db: MugDb) -> Result<Self> {
        let mut entries = HashMap::new();

        // Load all entries from the database
        let tree_entries = db.scan("INDEX", "")?;
        for (path_bytes, value_bytes) in tree_entries {
            let path = String::from_utf8_lossy(&path_bytes).to_string();
            if let Ok(entry) = serde_json::from_slice::<IndexEntry>(&value_bytes) {
                entries.insert(path, entry);
            }
        }

        Ok(Index { db, entries })
    }

    /// Stages a file by adding it to the index
    ///
    /// # Arguments
    /// * `path` - File path relative to repository root
    /// * `hash` - SHA-1 hash of the file contents
    ///
    /// # Returns
    /// Returns `Ok(())` on success, or an error if database operations fail
    pub fn add(&mut self, path: String, hash: String) -> Result<()> {
        // Validate inputs
        if path.is_empty() {
            return Err(crate::core::error::Error::Custom(
                "Path cannot be empty".to_string(),
            ));
        }
        if hash.is_empty() {
            return Err(crate::core::error::Error::Custom(
                "Hash cannot be empty".to_string(),
            ));
        }

        // Create the index entry
        let entry = IndexEntry {
            path: path.clone(),
            hash,
            mode: 0o100644, // Regular file mode
        };

        // Update in-memory cache
        self.entries.insert(path.clone(), entry.clone());

        // Persist to database
        let serialized = serde_json::to_vec(&entry)?;
        self.db.set("INDEX", &path, serialized)?;

        Ok(())
    }

    /// Adds an executable file to the index with executable mode
    pub fn add_executable(&mut self, path: String, hash: String) -> Result<()> {
        if path.is_empty() || hash.is_empty() {
            return Err(crate::core::error::Error::Custom(
                "Path and hash cannot be empty".to_string(),
            ));
        }

        let entry = IndexEntry {
            path: path.clone(),
            hash,
            mode: 0o100755, // Executable file mode
        };

        self.entries.insert(path.clone(), entry.clone());
        let serialized = serde_json::to_vec(&entry)?;
        self.db.set("INDEX", &path, serialized)?;

        Ok(())
    }

    /// Removes a file from the index (unstages it)
    ///
    /// # Arguments
    /// * `path` - File path relative to repository root
    pub fn remove(&mut self, path: &str) -> Result<()> {
        self.entries.remove(path);
        self.db.delete("INDEX", path)?;
        Ok(())
    }

    /// Retrieves an entry from the index
    ///
    /// # Arguments
    /// * `path` - File path to look up
    ///
    /// # Returns
    /// Returns a reference to the entry if found, or `None` if not staged
    pub fn get(&self, path: &str) -> Option<&IndexEntry> {
        self.entries.get(path)
    }

    /// Checks if a file is staged in the index
    pub fn contains(&self, path: &str) -> bool {
        self.entries.contains_key(path)
    }

    /// Returns all staged entries sorted by path
    pub fn entries(&self) -> Vec<IndexEntry> {
        let mut entries: Vec<_> = self.entries.values().cloned().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        entries
    }

    /// Returns all staged file paths sorted
    pub fn paths(&self) -> Vec<String> {
        let mut paths: Vec<_> = self.entries.keys().cloned().collect();
        paths.sort();
        paths
    }

    /// Clears all entries from the index (unstages everything)
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.db.clear_tree("INDEX")?;
        Ok(())
    }

    /// Returns `true` if the index contains no entries
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of staged entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Flushes the index to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()
    }

    /// Updates an entry's hash while preserving other properties
    pub fn update_hash(&mut self, path: &str, new_hash: String) -> Result<()> {
        if let Some(entry) = self.entries.get_mut(path) {
            entry.hash = new_hash.clone();
            let serialized = serde_json::to_vec(entry)?;
            self.db.set("INDEX", path, serialized)?;
        }
        Ok(())
    }

    /// Returns a list of entries matching a pattern (prefix search)
    pub fn find(&self, prefix: &str) -> Vec<IndexEntry> {
        self.entries
            .iter()
            .filter(|(path, _)| path.starts_with(prefix))
            .map(|(_, entry)| entry.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_add_and_get() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        index
            .add("test.txt".to_string(), "abc123".to_string())
            .unwrap();
        assert!(index.get("test.txt").is_some());
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_index_add_validates_inputs() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        // Empty path should fail
        assert!(index.add(String::new(), "abc123".to_string()).is_err());

        // Empty hash should fail
        assert!(index.add("test.txt".to_string(), String::new()).is_err());
    }

    #[test]
    fn test_index_remove() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        index
            .add("test.txt".to_string(), "abc123".to_string())
            .unwrap();
        assert_eq!(index.len(), 1);

        index.remove("test.txt").unwrap();
        assert!(index.get("test.txt").is_none());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_index_multiple_files() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        index
            .add("file1.txt".to_string(), "hash1".to_string())
            .unwrap();
        index
            .add("file2.txt".to_string(), "hash2".to_string())
            .unwrap();
        index
            .add("dir/file3.txt".to_string(), "hash3".to_string())
            .unwrap();

        assert_eq!(index.len(), 3);
        assert!(index.contains("file1.txt"));
        assert!(index.contains("file2.txt"));
        assert!(index.contains("dir/file3.txt"));
    }

    #[test]
    fn test_index_entries_sorted() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        index
            .add("z_file.txt".to_string(), "hash1".to_string())
            .unwrap();
        index
            .add("a_file.txt".to_string(), "hash2".to_string())
            .unwrap();
        index
            .add("m_file.txt".to_string(), "hash3".to_string())
            .unwrap();

        let entries = index.entries();
        assert_eq!(entries[0].path, "a_file.txt");
        assert_eq!(entries[1].path, "m_file.txt");
        assert_eq!(entries[2].path, "z_file.txt");
    }

    #[test]
    fn test_index_clear() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        index
            .add("test1.txt".to_string(), "hash1".to_string())
            .unwrap();
        index
            .add("test2.txt".to_string(), "hash2".to_string())
            .unwrap();

        assert_eq!(index.len(), 2);
        index.clear().unwrap();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_index_add_executable() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        index
            .add_executable("script.sh".to_string(), "abc123".to_string())
            .unwrap();

        let entry = index.get("script.sh").unwrap();
        assert_eq!(entry.mode, 0o100755);
    }

    #[test]
    fn test_index_update_hash() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        index
            .add("test.txt".to_string(), "hash1".to_string())
            .unwrap();
        index.update_hash("test.txt", "hash2".to_string()).unwrap();

        let entry = index.get("test.txt").unwrap();
        assert_eq!(entry.hash, "hash2");
    }

    #[test]
    fn test_index_find_prefix() {
        let dir = TempDir::new().unwrap();
        let db = MugDb::new(dir.path().join("db")).unwrap();
        let mut index = Index::new(db).unwrap();

        index
            .add("src/main.rs".to_string(), "hash1".to_string())
            .unwrap();
        index
            .add("src/lib.rs".to_string(), "hash2".to_string())
            .unwrap();
        index
            .add("tests/unit.rs".to_string(), "hash3".to_string())
            .unwrap();

        let src_files = index.find("src/");
        assert_eq!(src_files.len(), 2);
    }

    #[test]
    fn test_index_persistence() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("db");

        // Add files to index
        {
            let db = MugDb::new(db_path.clone()).unwrap();
            let mut index = Index::new(db).unwrap();
            index
                .add("file1.txt".to_string(), "hash1".to_string())
                .unwrap();
            index
                .add("file2.txt".to_string(), "hash2".to_string())
                .unwrap();
            index.flush().unwrap();
        }

        // Load index again and verify
        {
            let db = MugDb::new(db_path).unwrap();
            let index = Index::new(db).unwrap();
            assert_eq!(index.len(), 2);
            assert!(index.contains("file1.txt"));
            assert!(index.contains("file2.txt"));
        }
    }
}

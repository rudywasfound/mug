use sled::{Db, Tree};
use crate::error::{Error, Result};
use std::path::PathBuf;
use std::sync::Arc;

/// Lightweight embedded database wrapper around Sled
#[derive(Clone)]
pub struct MugDb {
    db: Arc<Db>,
}

impl MugDb {
    pub fn new(path: PathBuf) -> Result<Self> {
        let db = sled::open(&path)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(MugDb { db: Arc::new(db) })
    }

    /// Get the tree for storing HEAD ref
    pub fn head_tree(&self) -> Tree {
        self.db.open_tree("HEAD").unwrap()
    }

    /// Get the tree for storing branch refs
    pub fn branches_tree(&self) -> Tree {
        self.db.open_tree("BRANCHES").unwrap()
    }

    /// Get the tree for storing index/staging area
    pub fn index_tree(&self) -> Tree {
        self.db.open_tree("INDEX").unwrap()
    }

    /// Get the tree for storing commit metadata
    pub fn commits_tree(&self) -> Tree {
        self.db.open_tree("COMMITS").unwrap()
    }

    /// Flush database to disk
    pub fn flush(&self) -> Result<()> {
        self.db
            .flush()
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Get a value from a tree
    pub fn get<K: AsRef<[u8]>>(&self, tree_name: &str, key: K) -> Result<Option<Vec<u8>>> {
        let tree = self.db.open_tree(tree_name)
            .map_err(|e| Error::Database(e.to_string()))?;
        tree.get(key)
            .map_err(|e| Error::Database(e.to_string()))
            .map(|opt| opt.map(|v| v.to_vec()))
    }

    /// Set a value in a tree
    pub fn set<K: AsRef<[u8]>, V: AsRef<[u8]>>(&self, tree_name: &str, key: K, value: V) -> Result<()> {
        let tree = self.db.open_tree(tree_name)
            .map_err(|e| Error::Database(e.to_string()))?;
        tree.insert(key, value.as_ref())
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Delete a value from a tree
    pub fn delete<K: AsRef<[u8]>>(&self, tree_name: &str, key: K) -> Result<()> {
        let tree = self.db.open_tree(tree_name)
            .map_err(|e| Error::Database(e.to_string()))?;
        tree.remove(key)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Scan all entries in a tree
    pub fn scan<K: AsRef<[u8]>>(&self, tree_name: &str, prefix: K) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let tree = self.db.open_tree(tree_name)
            .map_err(|e| Error::Database(e.to_string()))?;
        let mut results = Vec::new();
        for item in tree.scan_prefix(prefix) {
            let (k, v) = item.map_err(|e| Error::Database(e.to_string()))?;
            results.push((k.to_vec(), v.to_vec()));
        }
        Ok(results)
    }

    /// Clear a tree
    pub fn clear_tree(&self, tree_name: &str) -> Result<()> {
        let tree = self.db.open_tree(tree_name)
            .map_err(|e| Error::Database(e.to_string()))?;
        tree.clear()
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }
}

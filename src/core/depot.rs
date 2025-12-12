/// Depot model - server-side repository structure for monorepo support
/// Follows Perforce depot model with integrated paths and revisions

use crate::core::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Depot revision info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepotRevision {
    /// Revision number
    pub revision: u64,
    /// Commit hash at this revision
    pub commit: String,
    /// Paths modified in this revision
    pub paths: Vec<String>,
    /// Author of changes
    pub author: String,
    /// Timestamp
    pub timestamp: String,
    /// Change description
    pub description: String,
}

/// Depot file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepotFile {
    /// Depot path
    pub path: String,
    /// Current revision
    pub revision: u64,
    /// File size in bytes
    pub size: u64,
    /// File hash
    pub hash: String,
    /// Last changed revision
    pub changed_revision: u64,
    /// Last changed by
    pub changed_by: String,
}

/// Depot integration point for path mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepotIntegration {
    /// Source depot path
    pub source: String,
    /// Target depot path
    pub target: String,
    /// Revision range
    pub revision_range: String,
}

/// Depot structure for monorepo
pub struct Depot {
    /// Depot name
    pub name: String,
    /// Depot type (local, stream, archive)
    pub depot_type: String,
    /// Map of path -> file info
    files: HashMap<String, DepotFile>,
    /// Revisions
    revisions: HashMap<u64, DepotRevision>,
}

impl Depot {
    /// Create new depot
    pub fn new(name: &str, depot_type: &str) -> Self {
        Self {
            name: name.to_string(),
            depot_type: depot_type.to_string(),
            files: HashMap::new(),
            revisions: HashMap::new(),
        }
    }

    /// Add file to depot
    pub fn add_file(&mut self, file: DepotFile) {
        self.files.insert(file.path.clone(), file);
    }

    /// Get file by depot path
    pub fn get_file(&self, path: &str) -> Option<&DepotFile> {
        self.files.get(path)
    }

    /// Get all files in path
    pub fn files_in_path(&self, depot_path: &str) -> Vec<&DepotFile> {
        self.files
            .values()
            .filter(|f| f.path.starts_with(depot_path))
            .collect()
    }

    /// Add revision
    pub fn add_revision(&mut self, revision: DepotRevision) {
        self.revisions.insert(revision.revision, revision);
    }

    /// Get revision
    pub fn get_revision(&self, revision: u64) -> Option<&DepotRevision> {
        self.revisions.get(&revision)
    }

    /// Get latest revision
    pub fn latest_revision(&self) -> Option<u64> {
        self.revisions.keys().max().copied()
    }

    /// Get revisions for path range
    pub fn revisions_for_path(&self, path: &str) -> Vec<&DepotRevision> {
        self.revisions
            .values()
            .filter(|r| r.paths.iter().any(|p| p.starts_with(path)))
            .collect()
    }

    /// Calculate depot statistics
    pub fn stats(&self) -> DepotStats {
        let total_size: u64 = self.files.values().map(|f| f.size).sum();
        let file_count = self.files.len();
        let revision_count = self.revisions.len();

        DepotStats {
            name: self.name.clone(),
            total_size,
            file_count,
            revision_count,
        }
    }
}

/// Depot statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepotStats {
    pub name: String,
    pub total_size: u64,
    pub file_count: usize,
    pub revision_count: usize,
}

impl DepotStats {
    /// Format size for display
    pub fn formatted_size(&self) -> String {
        let gb = self.total_size as f64 / (1024.0 * 1024.0 * 1024.0);
        if gb > 1.0 {
            format!("{:.2} GB", gb)
        } else {
            let mb = self.total_size as f64 / (1024.0 * 1024.0);
            format!("{:.2} MB", mb)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depot_creation() {
        let depot = Depot::new("main", "local");
        assert_eq!(depot.name, "main");
        assert_eq!(depot.depot_type, "local");
    }

    #[test]
    fn test_depot_add_file() {
        let mut depot = Depot::new("main", "local");
        let file = DepotFile {
            path: "//main/src/file.rs".to_string(),
            revision: 1,
            size: 1024,
            hash: "abc123".to_string(),
            changed_revision: 1,
            changed_by: "user".to_string(),
        };

        depot.add_file(file.clone());
        assert!(depot.get_file("//main/src/file.rs").is_some());
    }

    #[test]
    fn test_depot_files_in_path() {
        let mut depot = Depot::new("main", "local");

        for i in 1..5 {
            let file = DepotFile {
                path: format!("//main/src/file{}.rs", i),
                revision: 1,
                size: 1024,
                hash: format!("hash{}", i),
                changed_revision: 1,
                changed_by: "user".to_string(),
            };
            depot.add_file(file);
        }

        let files = depot.files_in_path("//main/src/");
        assert_eq!(files.len(), 4);
    }

    #[test]
    fn test_depot_stats() {
        let mut depot = Depot::new("main", "local");

        for i in 1..3 {
            let file = DepotFile {
                path: format!("//main/src/file{}.rs", i),
                revision: 1,
                size: 1024 * 1024,
                hash: format!("hash{}", i),
                changed_revision: 1,
                changed_by: "user".to_string(),
            };
            depot.add_file(file);
        }

        let stats = depot.stats();
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.total_size, 2 * 1024 * 1024);
    }
}

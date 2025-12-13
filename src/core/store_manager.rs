/// Hybrid store management - local files + centralized large file server
use crate::core::error::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration for object storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    /// Files larger than this threshold go to central server
    pub large_file_threshold_bytes: usize,
    /// Central server URL for large files
    pub central_server: Option<String>,
    /// Local cache directory for remote files
    pub cache_dir: PathBuf,
    /// Maximum cache size in bytes
    pub cache_size_bytes: usize,
    /// Cache policy: LRU, FIFO, or TTL
    pub cache_policy: CachePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CachePolicy {
    /// Least recently used
    LRU,
    /// First in, first out
    FIFO,
    /// Time to live (seconds)
    TTL(u64),
}

/// Object source location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectSource {
    /// Stored in local .mug/objects
    Local,
    /// Fetch from central server
    Central,
    /// Could be either
    Any,
}

/// Metadata about a stored object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    /// SHA256 hash of content
    pub hash: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Where it's stored
    pub source: ObjectSource,
    /// When it was last accessed
    pub last_accessed: String,
    /// Compression algorithm
    pub compression: Option<String>,
}

pub struct StoreManager {
    config: StoreConfig,
    cache_stats: CacheStats,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub size_bytes: usize,
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            large_file_threshold_bytes: 10 * 1024 * 1024, // 10MB default
            central_server: None,
            cache_dir: PathBuf::from(".mug/cache"),
            cache_size_bytes: 1024 * 1024 * 1024, // 1GB default
            cache_policy: CachePolicy::LRU,
        }
    }
}

impl StoreManager {
    pub fn new(config: StoreConfig) -> Self {
        StoreManager {
            config,
            cache_stats: CacheStats::default(),
        }
    }

    /// Determine where an object should be stored
    pub fn determine_source(&self, size_bytes: usize) -> ObjectSource {
        if size_bytes >= self.config.large_file_threshold_bytes
            && self.config.central_server.is_some()
        {
            ObjectSource::Central
        } else {
            ObjectSource::Local
        }
    }

    /// Check if an object exists locally
    pub fn exists_local(&self, hash: &str) -> Result<bool> {
        let obj_path = self.local_object_path(hash);
        Ok(obj_path.exists())
    }

    /// Check if an object exists in cache
    pub fn exists_cache(&self, hash: &str) -> Result<bool> {
        let cache_path = self.cache_path(hash);
        Ok(cache_path.exists())
    }

    /// Get local object path
    fn local_object_path(&self, hash: &str) -> PathBuf {
        let dir = &hash[..2];
        let file = &hash[2..];
        PathBuf::from(format!(".mug/objects/{}/{}", dir, file))
    }

    /// Get cache path for remote object
    fn cache_path(&self, hash: &str) -> PathBuf {
        self.config.cache_dir.join(hash)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> &CacheStats {
        &self.cache_stats
    }

    /// Evict oldest entry from cache (LRU policy)
    pub fn evict_lru(&mut self) -> Result<Option<String>> {
        // Would scan cache_dir, find oldest file by mtime, delete it
        // Return hash of evicted file
        Ok(None)
    }

    /// Clear entire cache
    pub fn clear_cache(&self) -> Result<()> {
        if self.config.cache_dir.exists() {
            std::fs::remove_dir_all(&self.config.cache_dir)?;
            std::fs::create_dir_all(&self.config.cache_dir)?;
        }
        Ok(())
    }

    /// Get current cache size
    pub fn cache_size(&self) -> Result<usize> {
        let mut total = 0;
        if self.config.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.config.cache_dir)? {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        total += metadata.len() as usize;
                    }
                }
            }
        }
        Ok(total)
    }

    /// Is cache full?
    pub fn is_cache_full(&self) -> Result<bool> {
        let size = self.cache_size()?;
        Ok(size >= self.config.cache_size_bytes)
    }

    /// Get centralized server URL
    pub fn central_server(&self) -> Option<&str> {
        self.config.central_server.as_deref()
    }

    /// Update central server URL
    pub fn set_central_server(&mut self, url: String) {
        self.config.central_server = Some(url);
    }

    /// Get large file threshold
    pub fn large_file_threshold(&self) -> usize {
        self.config.large_file_threshold_bytes
    }

    /// Set large file threshold
    pub fn set_large_file_threshold(&mut self, bytes: usize) {
        self.config.large_file_threshold_bytes = bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_source_local() {
        let mut config = StoreConfig::default();
        config.large_file_threshold_bytes = 1000;
        config.central_server = Some("https://server.example.com".to_string());

        let manager = StoreManager::new(config);

        // Small file -> local
        assert_eq!(manager.determine_source(500), ObjectSource::Local);
        // Large file -> central
        assert_eq!(manager.determine_source(2000), ObjectSource::Central);
    }

    #[test]
    fn test_determine_source_no_server() {
        let mut config = StoreConfig::default();
        config.large_file_threshold_bytes = 1000;
        config.central_server = None;

        let manager = StoreManager::new(config);

        // Even large files -> local if no server
        assert_eq!(manager.determine_source(2000), ObjectSource::Local);
    }

    #[test]
    fn test_cache_path() {
        let config = StoreConfig::default();
        let manager = StoreManager::new(config);

        let hash = "abc123def456";
        let path = manager.cache_path(hash);
        assert!(path.to_string_lossy().contains("abc123def456"));
    }
}

use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Content-addressed chunk with rolling hash deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub hash: String,
    pub size: usize,
    pub data: Vec<u8>,
}

impl Chunk {
    pub fn new(data: Vec<u8>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = format!("{:x}", hasher.finalize());
        
        Chunk {
            hash,
            size: data.len(),
            data,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self::new(data.to_vec())
    }
}

/// Content-addressed chunk store with global deduplication
pub struct ContentAddressedStore {
    chunks_dir: PathBuf,
    index: ChunkIndex,
}

impl ContentAddressedStore {
    pub fn new(base_path: &Path) -> std::io::Result<Self> {
        let chunks_dir = base_path.join("chunks");
        fs::create_dir_all(&chunks_dir)?;
        
        Ok(ContentAddressedStore {
            chunks_dir,
            index: ChunkIndex::new(),
        })
    }

    /// Store chunk and return hash
    pub fn store(&mut self, data: &[u8]) -> std::io::Result<String> {
        let chunk = Chunk::from_bytes(data);
        let path = self.chunks_dir.join(&chunk.hash);
        
        // Only write if not exists
        if !path.exists() {
            fs::write(&path, &chunk.data)?;
            self.index.add(chunk.hash.clone(), chunk.size);
        }
        
        Ok(chunk.hash)
    }

    /// Retrieve chunk by hash
    pub fn get(&self, hash: &str) -> std::io::Result<Vec<u8>> {
        let path = self.chunks_dir.join(hash);
        fs::read(path)
    }

    /// Dedup ratio: (deduplicated bytes / total bytes)
    pub fn dedup_ratio(&self) -> f64 {
        self.index.dedup_ratio()
    }

    pub fn total_size(&self) -> u64 {
        self.index.total_size()
    }

    pub fn chunk_count(&self) -> usize {
        self.index.chunk_count()
    }
}

/// Index tracking chunk references for deduplication analysis
pub struct ChunkIndex {
    chunks: HashMap<String, ChunkInfo>,
}

#[derive(Debug, Clone)]
struct ChunkInfo {
    size: usize,
    ref_count: usize,
}

impl ChunkIndex {
    pub fn new() -> Self {
        ChunkIndex {
            chunks: HashMap::new(),
        }
    }

    pub fn add(&mut self, hash: String, size: usize) {
        self.chunks
            .entry(hash)
            .and_modify(|info| info.ref_count += 1)
            .or_insert(ChunkInfo {
                size,
                ref_count: 1,
            });
    }

    /// Calculate deduplication ratio
    pub fn dedup_ratio(&self) -> f64 {
        let total_refs: usize = self.chunks.values().map(|c| c.ref_count).sum();
        let unique: usize = self.chunks.len();
        
        if total_refs == 0 {
            0.0
        } else {
            1.0 - (unique as f64 / total_refs as f64)
        }
    }

    pub fn total_size(&self) -> u64 {
        self.chunks.values().map(|c| c.size as u64).sum()
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let data = b"test data".to_vec();
        let chunk = Chunk::new(data);
        assert_eq!(chunk.size, 9);
        assert!(!chunk.hash.is_empty());
    }

    #[test]
    fn test_chunk_dedup() {
        let mut index = ChunkIndex::new();
        let hash = "abc123".to_string();
        
        index.add(hash.clone(), 100);
        index.add(hash.clone(), 100);
        index.add(hash.clone(), 100);
        
        assert_eq!(index.chunk_count(), 1);
        assert!(index.dedup_ratio() > 0.5);
    }
}

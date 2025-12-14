use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkMetadata {
    pub hash: String,
    pub size: u64,
    pub offset: u64,
    pub compressed_size: Option<u64>,
    pub compression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPackManifest {
    pub pack_id: String,
    pub version: String,
    pub created_at: String,
    pub total_size: u64,
    pub chunk_count: usize,
    pub chunks: Vec<ChunkMetadata>,
    pub checksums: HashMap<String, String>, // hash -> checksum
    pub metadata: PackMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackMetadata {
    pub source_branch: Option<String>,
    pub base_commit: Option<String>,
    pub commits_included: Vec<String>,
    pub author: Option<String>,
    pub timestamp: String,
}

impl ChunkPackManifest {
    pub fn new(pack_id: String) -> Self {
        ChunkPackManifest {
            pack_id,
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            total_size: 0,
            chunk_count: 0,
            chunks: Vec::new(),
            checksums: HashMap::new(),
            metadata: PackMetadata {
                source_branch: None,
                base_commit: None,
                commits_included: Vec::new(),
                author: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    pub fn add_chunk(
        &mut self,
        hash: String,
        size: u64,
        offset: u64,
        checksum: String,
    ) {
        let chunk = ChunkMetadata {
            hash: hash.clone(),
            size,
            offset,
            compressed_size: None,
            compression: None,
        };
        self.chunks.push(chunk);
        self.checksums.insert(hash, checksum);
        self.total_size += size;
        self.chunk_count = self.chunks.len();
    }

    pub fn add_chunk_compressed(
        &mut self,
        hash: String,
        original_size: u64,
        compressed_size: u64,
        offset: u64,
        checksum: String,
        compression: String,
    ) {
        let chunk = ChunkMetadata {
            hash: hash.clone(),
            size: original_size,
            offset,
            compressed_size: Some(compressed_size),
            compression: Some(compression),
        };
        self.chunks.push(chunk);
        self.checksums.insert(hash, checksum);
        self.total_size += compressed_size;
        self.chunk_count = self.chunks.len();
    }

    pub fn set_metadata(
        &mut self,
        source_branch: Option<String>,
        base_commit: Option<String>,
        commits: Vec<String>,
        author: Option<String>,
    ) {
        self.metadata.source_branch = source_branch;
        self.metadata.base_commit = base_commit;
        self.metadata.commits_included = commits;
        self.metadata.author = author;
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn verify_chunk(&self, hash: &str, actual_checksum: &str) -> bool {
        self.checksums
            .get(hash)
            .map(|expected| expected == actual_checksum)
            .unwrap_or(false)
    }

    pub fn get_chunk(&self, hash: &str) -> Option<&ChunkMetadata> {
        self.chunks.iter().find(|c| c.hash == hash)
    }

    pub fn get_download_size(&self) -> u64 {
        self.chunks
            .iter()
            .map(|c| c.compressed_size.unwrap_or(c.size))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_manifest_creation() {
        let manifest = ChunkPackManifest::new("pack-001".to_string());
        assert_eq!(manifest.pack_id, "pack-001");
        assert_eq!(manifest.chunk_count, 0);
        assert_eq!(manifest.total_size, 0);
    }

    #[test]
    fn test_add_chunk() {
        let mut manifest = ChunkPackManifest::new("pack-001".to_string());
        manifest.add_chunk(
            "hash1".to_string(),
            1024,
            0,
            "checksum1".to_string(),
        );
        assert_eq!(manifest.chunk_count, 1);
        assert_eq!(manifest.total_size, 1024);
    }

    #[test]
    fn test_add_multiple_chunks() {
        let mut manifest = ChunkPackManifest::new("pack-001".to_string());
        manifest.add_chunk("hash1".to_string(), 1024, 0, "checksum1".to_string());
        manifest.add_chunk("hash2".to_string(), 2048, 1024, "checksum2".to_string());
        assert_eq!(manifest.chunk_count, 2);
        assert_eq!(manifest.total_size, 3072);
    }

    #[test]
    fn test_verify_chunk() {
        let mut manifest = ChunkPackManifest::new("pack-001".to_string());
        manifest.add_chunk(
            "hash1".to_string(),
            1024,
            0,
            "abc123def456".to_string(),
        );

        assert!(manifest.verify_chunk("hash1", "abc123def456"));
        assert!(!manifest.verify_chunk("hash1", "wrong"));
        assert!(!manifest.verify_chunk("nonexistent", "abc123def456"));
    }

    #[test]
    fn test_manifest_serialization() {
        let mut manifest = ChunkPackManifest::new("pack-001".to_string());
        manifest.add_chunk("hash1".to_string(), 1024, 0, "checksum1".to_string());

        let json = manifest.to_json().unwrap();
        let restored = ChunkPackManifest::from_json(&json).unwrap();

        assert_eq!(restored.pack_id, manifest.pack_id);
        assert_eq!(restored.chunk_count, manifest.chunk_count);
    }

    #[test]
    fn test_compressed_chunk() {
        let mut manifest = ChunkPackManifest::new("pack-001".to_string());
        manifest.add_chunk_compressed(
            "hash1".to_string(),
            10240, // original
            2048,  // compressed
            0,
            "checksum1".to_string(),
            "zstd".to_string(),
        );

        assert_eq!(manifest.chunk_count, 1);
        assert_eq!(manifest.total_size, 2048); // Compressed size is stored
        let chunk = manifest.get_chunk("hash1").unwrap();
        assert_eq!(chunk.size, 10240);
        assert_eq!(chunk.compressed_size, Some(2048));
    }

    #[test]
    fn test_get_download_size() {
        let mut manifest = ChunkPackManifest::new("pack-001".to_string());
        manifest.add_chunk("hash1".to_string(), 1024, 0, "checksum1".to_string());
        manifest.add_chunk_compressed(
            "hash2".to_string(),
            2048,
            512,
            1024,
            "checksum2".to_string(),
            "zstd".to_string(),
        );

        let size = manifest.get_download_size();
        assert_eq!(size, 1024 + 512);
    }
}

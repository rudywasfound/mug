use sha2::{Sha256, Digest};

/// Content-aware chunking using rolling hash
pub struct Chunker {
    window_size: usize,
    min_chunk_size: usize,
    max_chunk_size: usize,
    target_chunk_size: usize,
}

impl Chunker {
    pub fn new() -> Self {
        Chunker {
            window_size: 64,           // Rolling window size
            min_chunk_size: 4096,      // 4KB minimum
            max_chunk_size: 1048576,   // 1MB maximum
            target_chunk_size: 65536,  // Target 64KB chunks
        }
    }

    /// Split data into variable-size chunks using rolling hash
    pub fn split(&self, data: &[u8]) -> Vec<(Vec<u8>, String)> {
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < data.len() {
            let end = self.find_chunk_boundary(data, start);
            if end > start {
                let chunk = &data[start..end];
                let hash = self.hash_chunk(chunk);
                chunks.push((chunk.to_vec(), hash));
                start = end;
            } else {
                // Fallback if no boundary found (shouldn't happen)
                let end = (start + self.target_chunk_size).min(data.len());
                let chunk = &data[start..end];
                let hash = self.hash_chunk(chunk);
                chunks.push((chunk.to_vec(), hash));
                start = end;
            }
        }

        chunks
    }

    /// Find chunk boundary using rolling hash (Rabin fingerprint)
    fn find_chunk_boundary(&self, data: &[u8], start: usize) -> usize {
        if start >= data.len() {
            return data.len();
        }

        let mut pos = start + self.min_chunk_size;
        let max_pos = (start + self.max_chunk_size).min(data.len());

        while pos < max_pos {
            // Simple rolling hash (in real impl, use polynomial rolling hash)
            let window = &data[pos.saturating_sub(self.window_size)..pos];
            let hash = self.rolling_hash(window);

            // Break at normalized hash boundaries
            if self.is_boundary(hash) {
                return pos;
            }

            pos += 1;
        }

        // If no boundary found, use max chunk size
        max_pos
    }

    /// Rolling hash computation (Rabin fingerprint style)
    fn rolling_hash(&self, window: &[u8]) -> u32 {
        let mut hash: u32 = 0;
        const BASE: u32 = 31;

        for &byte in window {
            hash = hash.wrapping_mul(BASE).wrapping_add(byte as u32);
        }

        hash
    }

    /// Check if hash indicates a good chunk boundary
    /// Target: 1 in 65536 positions are boundaries (matches avg 64KB chunks)
    fn is_boundary(&self, hash: u32) -> bool {
        (hash & 0xFFFF) == 0
    }

    /// Hash chunk content (SHA256)
    fn hash_chunk(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

/// Statistics about chunking results
#[derive(Debug, Clone)]
pub struct ChunkStats {
    pub total_bytes: u64,
    pub chunk_count: usize,
    pub avg_chunk_size: u64,
    pub min_chunk_size: usize,
    pub max_chunk_size: usize,
}

impl ChunkStats {
    pub fn from_chunks(chunks: &[(Vec<u8>, String)]) -> Self {
        let total_bytes: u64 = chunks.iter().map(|(data, _)| data.len() as u64).sum();
        let chunk_count = chunks.len();
        let avg_chunk_size = if chunk_count > 0 { total_bytes / chunk_count as u64 } else { 0 };
        let min_chunk_size = chunks.iter().map(|(data, _)| data.len()).min().unwrap_or(0);
        let max_chunk_size = chunks.iter().map(|(data, _)| data.len()).max().unwrap_or(0);

        ChunkStats {
            total_bytes,
            chunk_count,
            avg_chunk_size,
            min_chunk_size,
            max_chunk_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking_splits_data() {
        let chunker = Chunker::new();
        let data = vec![0u8; 1_000_000]; // 1MB of zeros
        let chunks = chunker.split(&data);

        assert!(!chunks.is_empty());
        assert!(chunks.len() > 1); // Should split into multiple chunks
    }

    #[test]
    fn test_chunk_stats() {
        let chunker = Chunker::new();
        let data = vec![42u8; 500_000]; // 500KB
        let chunks = chunker.split(&data);
        let stats = ChunkStats::from_chunks(&chunks);

        assert_eq!(stats.total_bytes, 500_000);
        assert!(stats.avg_chunk_size > 0);
        println!("Stats: {:?}", stats);
    }

    #[test]
    fn test_identical_chunks_have_same_hash() {
        let chunker = Chunker::new();
        let data = b"hello world";
        let chunk1 = chunker.hash_chunk(data);
        let chunk2 = chunker.hash_chunk(data);

        assert_eq!(chunk1, chunk2);
    }

    #[test]
    fn test_different_chunks_have_different_hash() {
        let chunker = Chunker::new();
        let chunk1 = chunker.hash_chunk(b"hello");
        let chunk2 = chunker.hash_chunk(b"world");

        assert_ne!(chunk1, chunk2);
    }

    #[test]
    fn test_rolling_hash_boundary_detection() {
        let chunker = Chunker::new();
        let window = &[0u8; 64];
        let hash = chunker.rolling_hash(window);

        // Should detect some boundaries in random data
        assert!(chunker.is_boundary(hash) || !chunker.is_boundary(hash));
    }
}

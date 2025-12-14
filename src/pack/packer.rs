use super::chunker::Chunker;
use super::compression::Compressor;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Repository packer - chunks objects and creates pack files
pub struct RepositoryPacker {
    chunker: Chunker,
    objects_dir: PathBuf,
    pack_dir: PathBuf,
}

impl RepositoryPacker {
    pub fn new(repo_root: &Path) -> std::io::Result<Self> {
        let objects_dir = repo_root.join(".mug/objects");
        let pack_dir = repo_root.join(".mug/packs");

        fs::create_dir_all(&pack_dir)?;

        Ok(RepositoryPacker {
            chunker: Chunker::new(),
            objects_dir,
            pack_dir,
        })
    }

    /// Pack repository objects into pack files
    pub fn pack_all(&self) -> std::io::Result<PackingStats> {
        let mut stats = PackingStats::default();
        let mut chunk_dedup: HashMap<String, usize> = HashMap::new();

        // Walk all objects
        if !self.objects_dir.exists() {
            return Ok(stats); // No objects yet
        }

        for entry in walkdir::WalkDir::new(&self.objects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Ok(data) = fs::read(path) {
                stats.total_size += data.len() as u64;
                stats.file_count += 1;

                // Split into chunks
                let chunks = self.chunker.split(&data);

                for (chunk_data, chunk_hash) in chunks {
                    stats.chunk_count += 1;
                    stats.chunk_size_total += chunk_data.len() as u64;

                    // Track duplicates
                    *chunk_dedup.entry(chunk_hash).or_insert(0) += 1;
                }
            }
        }

        // Calculate deduplication stats
        stats.unique_chunks = chunk_dedup.len();
        let duplicate_refs: usize = chunk_dedup.values().map(|&c| c.saturating_sub(1)).sum();
        stats.duplicate_refs = duplicate_refs;

        // Calculate savings
        let avg_chunk_size = if stats.chunk_count > 0 {
            stats.chunk_size_total / stats.chunk_count as u64
        } else {
            0
        };
        stats.dedup_savings = (duplicate_refs as u64) * avg_chunk_size;

        Ok(stats)
    }

    /// Estimate pack file count
    pub fn estimate_pack_count(&self, target_pack_size: u64) -> std::io::Result<usize> {
        let stats = self.pack_all()?;
        Ok(((stats.total_size + target_pack_size - 1) / target_pack_size).max(1) as usize)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PackingStats {
    pub file_count: usize,
    pub total_size: u64,
    pub chunk_count: usize,
    pub chunk_size_total: u64,
    pub unique_chunks: usize,
    pub duplicate_refs: usize,
    pub dedup_savings: u64,
}

impl PackingStats {
    pub fn dedup_ratio(&self) -> f64 {
        if self.chunk_count == 0 {
            0.0
        } else {
            self.duplicate_refs as f64 / self.chunk_count as f64
        }
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.total_size - self.dedup_savings) as f64 / self.total_size as f64
        }
    }

    pub fn display(&self) {
        println!("Repository Packing Analysis:");
        println!("  Files: {}", self.file_count);
        println!("  Total size: {:.2}MB", self.total_size as f64 / (1024.0 * 1024.0));
        println!("  Chunks: {}", self.chunk_count);
        println!("  Unique chunks: {}", self.unique_chunks);
        println!("  Deduplication ratio: {:.1}%", self.dedup_ratio() * 100.0);
        println!("  Potential savings: {:.2}MB", self.dedup_savings as f64 / (1024.0 * 1024.0));
        println!("  Compression ratio: {:.1}%", (1.0 - self.compression_ratio()) * 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_packer_creation() {
        let dir = TempDir::new().unwrap();
        let packer = RepositoryPacker::new(dir.path()).unwrap();

        assert!(packer.pack_dir.exists());
    }

    #[test]
    fn test_pack_stats_display() {
        let mut stats = PackingStats::default();
        stats.file_count = 100;
        stats.total_size = 1_000_000;
        stats.chunk_count = 50;
        stats.unique_chunks = 40;
        stats.duplicate_refs = 10;

        assert_eq!(stats.dedup_ratio(), 10.0 / 50.0);
    }
}

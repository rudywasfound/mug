use super::compression::{Compressor, ZstdCompressor};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::Path;
use sha2::{Sha256, Digest};

/// Pack file format: [HEADER][CHUNK_ENTRY]*[INDEX][FOOTER]
#[derive(Debug, Serialize, Deserialize)]
pub struct PackFile {
    pub id: String,
    pub entries: Vec<PackEntry>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackEntry {
    pub hash: String,
    pub size: usize,
    pub compressed_size: usize,
    pub offset: u64,
}

pub struct PackWriter {
    id: String,
    entries: Vec<PackEntry>,
    compressor: ZstdCompressor,
    buffer: BufWriter<File>,
    offset: u64,
}

impl PackWriter {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let buffer = BufWriter::new(file);
        let id = uuid::Uuid::new_v4().to_string();
        
        Ok(PackWriter {
            id,
            entries: Vec::new(),
            compressor: ZstdCompressor::fast(),
            buffer,
            offset: 0,
        })
    }

    /// Add chunk to pack
    pub fn add_chunk(&mut self, hash: &str, data: &[u8]) -> std::io::Result<()> {
        let compressed = self.compressor.compress(data)?;
        let compressed_size = compressed.len();
        
        self.buffer.write_all(&compressed)?;
        
        self.entries.push(PackEntry {
            hash: hash.to_string(),
            size: data.len(),
            compressed_size,
            offset: self.offset,
        });
        
        self.offset += compressed_size as u64;
        Ok(())
    }

    /// Finalize pack file and write index
    pub fn finish(mut self) -> std::io::Result<()> {
        let pack = PackFile {
            id: self.id,
            entries: self.entries,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        let index = serde_json::to_vec(&pack)?;
        self.buffer.write_all(&index)?;
        self.buffer.flush()?;
        
        Ok(())
    }
}

pub struct PackReader {
    compressor: ZstdCompressor,
    pack: PackFile,
}

impl PackReader {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents)?;
        
        // Parse index from end (simplified - in real impl would use proper serialization)
        if let Ok(pack) = serde_json::from_slice::<PackFile>(&contents) {
            Ok(PackReader {
                compressor: ZstdCompressor::default(),
                pack,
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid pack file",
            ))
        }
    }

    /// Get chunk by hash
    pub fn get_chunk(&self, hash: &str) -> Option<Vec<u8>> {
        self.pack
            .entries
            .iter()
            .find(|e| e.hash == hash)
            .and_then(|entry| {
                // In real impl, would seek to offset and read from file
                // For now, placeholder
                None
            })
    }

    pub fn stats(&self) -> PackStats {
        let total_uncompressed: u64 = self.pack.entries.iter().map(|e| e.size as u64).sum();
        let total_compressed: u64 = self.pack.entries.iter().map(|e| e.compressed_size as u64).sum();
        
        PackStats {
            chunk_count: self.pack.entries.len(),
            uncompressed_size: total_uncompressed,
            compressed_size: total_compressed,
            compression_ratio: total_compressed as f64 / total_uncompressed.max(1) as f64,
        }
    }
}

#[derive(Debug)]
pub struct PackStats {
    pub chunk_count: usize,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pack_write_read() {
        let dir = TempDir::new().unwrap();
        let pack_path = dir.path().join("test.pack");
        
        {
            let mut writer = PackWriter::new(&pack_path).unwrap();
            writer.add_chunk("hash1", b"hello world").unwrap();
            writer.add_chunk("hash2", b"test data").unwrap();
            writer.finish().unwrap();
        }
        
        let reader = PackReader::open(&pack_path).unwrap();
        let stats = reader.stats();
        
        assert_eq!(stats.chunk_count, 2);
        assert!(stats.compression_ratio < 1.0);
    }
}

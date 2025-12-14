use super::chunker::Chunker;
use super::compression::{ZstdCompressor, Compressor};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::collections::HashMap;
use sha2::{Sha256, Digest};

/// Builds pack files from repository objects with chunking and compression
pub struct PackBuilder {
    chunker: Chunker,
    compressor: ZstdCompressor,
    target_pack_size: u64,
    objects_dir: PathBuf,
}

impl PackBuilder {
    pub fn new(repo_root: &Path, target_pack_size: u64) -> std::io::Result<Self> {
        let objects_dir = repo_root.join(".mug/objects");
        
        Ok(PackBuilder {
            chunker: Chunker::new(),
            compressor: ZstdCompressor::fast(),
            target_pack_size,
            objects_dir,
        })
    }

    /// Build all packs and return manifest
    pub fn build_packs(&self, output_dir: &Path) -> std::io::Result<PackManifest> {
        fs::create_dir_all(output_dir)?;

        let mut manifest = PackManifest::new();
        let mut current_pack = PackBuffer::new(0);
        let mut chunk_registry: HashMap<String, ChunkLocation> = HashMap::new();

        // Walk all objects
        if !self.objects_dir.exists() {
            eprintln!("No objects directory found");
            return Ok(manifest);
        }

        let mut object_count = 0;
        for entry in walkdir::WalkDir::new(&self.objects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let object_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            if let Ok(data) = fs::read(path) {
                object_count += 1;

                // Chunk the object
                let chunks = self.chunker.split(&data);

                for (chunk_data, chunk_hash) in chunks {
                    // Compress chunk
                    let compressed = self.compressor.compress(&chunk_data)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                    // Check if starting new pack
                    if current_pack.size + compressed.len() as u64 > self.target_pack_size {
                        // Finalize current pack
                        let pack_info = self.write_pack(&current_pack, output_dir, manifest.packs.len())?;
                        manifest.packs.push(pack_info);

                        current_pack = PackBuffer::new(manifest.packs.len() as u32);
                    }

                    // Add to current pack
                    let offset = current_pack.size;
                    current_pack.chunks.push(ChunkEntry {
                        hash: chunk_hash.clone(),
                        offset,
                        size: compressed.len() as u32,
                        original_size: chunk_data.len() as u32,
                    });
                    current_pack.data.write_all(&compressed)?;
                    current_pack.size += compressed.len() as u64;

                    // Register chunk location
                    chunk_registry.insert(chunk_hash, ChunkLocation {
                        pack_id: current_pack.pack_id,
                        offset,
                    });
                }
            }
        }

        // Finalize last pack
        if !current_pack.chunks.is_empty() {
            let pack_info = self.write_pack(&current_pack, output_dir, manifest.packs.len())?;
            manifest.packs.push(pack_info);
        }

        manifest.object_count = object_count;
        manifest.chunk_registry = chunk_registry;
        manifest.created_at = chrono::Utc::now().to_rfc3339();

        Ok(manifest)
    }

    /// Write a single pack file with index
    fn write_pack(&self, buffer: &PackBuffer, output_dir: &Path, pack_num: usize) -> std::io::Result<PackInfo> {
        let pack_name = format!("pack-{:04}.mug", pack_num);
        let pack_path = output_dir.join(&pack_name);

        let mut file = fs::File::create(&pack_path)?;

        // Write magic header
        file.write_all(b"MUG1")?;

        // Write pack version
        file.write_all(&[1u8])?;

        // Write number of chunks
        file.write_all(&(buffer.chunks.len() as u32).to_le_bytes())?;

        // Write chunk entries and data
        let mut data_offset = 0u64;
        for chunk in &buffer.chunks {
            // Write entry header
            file.write_all(chunk.hash.as_bytes())?;
            file.write_all(&chunk.original_size.to_le_bytes())?;
            file.write_all(&chunk.size.to_le_bytes())?;
            file.write_all(&data_offset.to_le_bytes())?;

            data_offset += chunk.size as u64;
        }

        // Write all compressed data
        file.write_all(&buffer.data)?;

        let pack_info = PackInfo {
            id: buffer.pack_id,
            name: pack_name,
            size: pack_path.metadata()?.len(),
            chunk_count: buffer.chunks.len(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(pack_info)
    }
}

/// In-memory pack buffer
struct PackBuffer {
    pack_id: u32,
    chunks: Vec<ChunkEntry>,
    data: Vec<u8>,
    size: u64,
}

impl PackBuffer {
    fn new(pack_id: u32) -> Self {
        PackBuffer {
            pack_id,
            chunks: Vec::new(),
            data: Vec::new(),
            size: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct ChunkEntry {
    hash: String,
    offset: u64,
    size: u32,
    original_size: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkLocation {
    pub pack_id: u32,
    pub offset: u64,
}

/// Pack manifest for tracking all packs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackManifest {
    pub packs: Vec<PackInfo>,
    pub object_count: usize,
    pub chunk_registry: HashMap<String, ChunkLocation>,
    pub created_at: String,
}

impl PackManifest {
    pub fn new() -> Self {
        PackManifest {
            packs: Vec::new(),
            object_count: 0,
            chunk_registry: HashMap::new(),
            created_at: String::new(),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn total_size(&self) -> u64 {
        self.packs.iter().map(|p| p.size).sum()
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.packs.is_empty() {
            0.0
        } else {
            let total_compressed: u64 = self.packs.iter().map(|p| p.size).sum();
            let avg_chunk_size = 65536u64; // ~65KB average
            let total_uncompressed = self.chunk_registry.len() as u64 * avg_chunk_size;
            
            if total_uncompressed == 0 {
                0.0
            } else {
                total_compressed as f64 / total_uncompressed as f64
            }
        }
    }

    pub fn display(&self) {
        println!("Pack Manifest:");
        println!("  Packs: {}", self.packs.len());
        println!("  Total size: {:.2}MB", self.total_size() as f64 / (1024.0 * 1024.0));
        println!("  Objects: {}", self.object_count);
        println!("  Chunks: {}", self.chunk_registry.len());
        println!("  Compression ratio: {:.1}%", self.compression_ratio() * 100.0);
        println!("  Created: {}", self.created_at);
        
        for pack in &self.packs {
            println!("  {} - {:.2}MB ({} chunks)", 
                pack.name,
                pack.size as f64 / (1024.0 * 1024.0),
                pack.chunk_count
            );
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackInfo {
    pub id: u32,
    pub name: String,
    pub size: u64,
    pub chunk_count: usize,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pack_builder_creation() {
        let dir = TempDir::new().unwrap();
        let builder = PackBuilder::new(dir.path(), 1_000_000).unwrap();
        
        assert_eq!(builder.target_pack_size, 1_000_000);
    }

    #[test]
    fn test_manifest_creation() {
        let manifest = PackManifest::new();
        assert_eq!(manifest.packs.len(), 0);
        assert_eq!(manifest.object_count, 0);
    }
}

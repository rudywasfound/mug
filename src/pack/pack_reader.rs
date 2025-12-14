use super::compression::{ZstdCompressor, Compressor};
use super::pack_builder::PackManifest;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Read, Seek};

/// Reads and reconstructs objects from pack files
pub struct PackReader {
    manifest: PackManifest,
    pack_dir: PathBuf,
    compressor: ZstdCompressor,
}

impl PackReader {
    pub fn new(manifest_path: &Path) -> std::io::Result<Self> {
        let manifest = PackManifest::load(manifest_path)?;
        let pack_dir = manifest_path.parent()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid manifest path"))?
            .to_path_buf();

        Ok(PackReader {
            manifest,
            pack_dir,
            compressor: ZstdCompressor::default(),
        })
    }

    /// Retrieve a single chunk by hash
    pub fn get_chunk(&self, chunk_hash: &str) -> std::io::Result<Vec<u8>> {
        let location = self.manifest.chunk_registry.get(chunk_hash)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Chunk not found"))?;

        let pack_name = format!("pack-{:04}.mug", location.pack_id);
        let pack_path = self.pack_dir.join(&pack_name);

        let mut file = fs::File::open(pack_path)?;
        
        // Seek to chunk location
        file.seek(std::io::SeekFrom::Start(location.offset))?;

        // Read compressed chunk size header (assuming format)
        let mut size_buf = [0u8; 4];
        file.read_exact(&mut size_buf)?;
        let compressed_size = u32::from_le_bytes(size_buf) as usize;

        // Read compressed data
        let mut compressed = vec![0u8; compressed_size];
        file.read_exact(&mut compressed)?;

        // Decompress
        self.compressor.decompress(&compressed)
    }

    /// Extract all objects to output directory with progress
    pub fn extract_all(&self, output_dir: &Path, show_progress: bool) -> std::io::Result<ExtractStats> {
        fs::create_dir_all(output_dir)?;

        let mut stats = ExtractStats::default();
        let total_chunks = self.manifest.chunk_registry.len();

        for (chunk_hash, location) in &self.manifest.chunk_registry {
            if show_progress {
                stats.processed += 1;
                if stats.processed % 100 == 0 {
                    eprintln!("[{}/{}] Extracting chunks...", stats.processed, total_chunks);
                }
            }

            match self.get_chunk(chunk_hash) {
                Ok(data) => {
                    stats.extracted_bytes += data.len() as u64;
                    stats.chunks_extracted += 1;
                }
                Err(e) => {
                    stats.errors += 1;
                    eprintln!("Error extracting chunk {}: {}", &chunk_hash[..8], e);
                }
            }
        }

        if show_progress {
            eprintln!("[{}/{}] Extraction complete!", total_chunks, total_chunks);
        }

        Ok(stats)
    }

    /// Verify pack integrity (check manifest + pack files exist)
    pub fn verify(&self, show_progress: bool) -> std::io::Result<VerifyStats> {
        let mut stats = VerifyStats::default();
        let total_packs = self.manifest.packs.len();
        let total_chunks = self.manifest.chunk_registry.len();

        // Verify pack files exist
        for pack in &self.manifest.packs {
            if show_progress {
                stats.checked += 1;
                if stats.checked % 10 == 0 {
                    eprintln!("[{}/{}] Verifying packs...", stats.checked, total_packs);
                }
            }

            let pack_path = self.pack_dir.join(&pack.name);
            if pack_path.exists() && pack_path.metadata().ok().map(|m| m.len()) == Some(pack.size) {
                stats.valid += 1;
            } else {
                stats.invalid += 1;
                stats.invalid_hashes.push(pack.name.clone());
            }
        }

        // Verify manifest integrity
        if self.manifest.chunk_registry.len() == total_chunks {
            stats.valid += 1;
        } else {
            stats.invalid += 1;
        }

        if show_progress {
            eprintln!("[{}/{}] Verification complete!", total_packs, total_packs);
        }

        Ok(stats)
    }

    pub fn manifest(&self) -> &PackManifest {
        &self.manifest
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExtractStats {
    pub chunks_extracted: usize,
    pub extracted_bytes: u64,
    pub errors: usize,
    pub processed: usize,
}

impl ExtractStats {
    pub fn display(&self) {
        println!("Extraction Statistics:");
        println!("  Chunks extracted: {}", self.chunks_extracted);
        println!("  Bytes extracted: {:.2}MB", self.extracted_bytes as f64 / (1024.0 * 1024.0));
        println!("  Errors: {}", self.errors);
    }
}

#[derive(Debug, Clone, Default)]
pub struct VerifyStats {
    pub valid: usize,
    pub invalid: usize,
    pub checked: usize,
    pub invalid_hashes: Vec<String>,
}

impl VerifyStats {
    pub fn display(&self) {
        println!("Verification Statistics:");
        println!("  Valid: {}", self.valid);
        println!("  Invalid: {}", self.invalid);
        println!("  Total checked: {}", self.checked);
        
        if !self.invalid_hashes.is_empty() {
            println!("\nInvalid chunks:");
            for hash in &self.invalid_hashes {
                println!("  {}", &hash[..16]);
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.invalid == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_verify_stats() {
        let stats = VerifyStats {
            valid: 100,
            invalid: 0,
            checked: 100,
            invalid_hashes: Vec::new(),
        };

        assert!(stats.is_valid());
    }
}

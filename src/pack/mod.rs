pub mod chunk;
pub mod compression;
pub mod pack_file;

pub use chunk::{Chunk, ChunkIndex, ContentAddressedStore};
pub use compression::Compressor;
pub use pack_file::{PackFile, PackWriter, PackReader};

/// Pack metadata for tracking stored chunks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackMetadata {
    pub id: String,
    pub chunk_count: usize,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub created_at: String,
}

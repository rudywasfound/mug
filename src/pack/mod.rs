pub mod chunk;
pub mod compression;
pub mod pack_file;
pub mod chunker;
pub mod packer;
pub mod pack_builder;

pub use chunk::{Chunk, ChunkIndex, ContentAddressedStore};
pub use compression::Compressor;
pub use pack_file::{PackFile, PackWriter, PackReader};
pub use chunker::{Chunker, ChunkStats};
pub use packer::{RepositoryPacker, PackingStats};
pub use pack_builder::{PackBuilder, PackManifest, PackInfo};

/// Pack metadata for tracking stored chunks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackMetadata {
    pub id: String,
    pub chunk_count: usize,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub created_at: String,
}

use std::io::{Read, Write};

/// Compression codec abstraction
pub trait Compressor {
    fn compress(&self, data: &[u8]) -> std::io::Result<Vec<u8>>;
    fn decompress(&self, data: &[u8]) -> std::io::Result<Vec<u8>>;
}

/// Zstd compression (5-10x faster than zlib, better ratios)
pub struct ZstdCompressor {
    level: i32,
}

impl ZstdCompressor {
    pub fn new(level: i32) -> Self {
        ZstdCompressor { level }
    }

    pub fn default() -> Self {
        ZstdCompressor { level: 10 }
    }

    pub fn fast() -> Self {
        ZstdCompressor { level: 3 }
    }
}

impl Compressor for ZstdCompressor {
    fn compress(&self, data: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut encoder = zstd::Encoder::new(Vec::new(), self.level)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        encoder.write_all(data)?;
        encoder.finish()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn decompress(&self, data: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut decoder = zstd::Decoder::new(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)?;
        Ok(result)
    }
}

/// Flate2/zlib compression (backwards compatible with Git)
pub struct FlateCompressor;

impl Compressor for FlateCompressor {
    fn compress(&self, data: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data)?;
        encoder.finish()
    }

    fn decompress(&self, data: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_compression() {
        let compressor = ZstdCompressor::default();
        let data = b"hello world".repeat(100);
        
        let compressed = compressor.compress(&data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        
        assert_eq!(data.to_vec(), decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compression_ratio() {
        let compressor = ZstdCompressor::default();
        let data = vec![b'a'; 10000];
        
        let compressed = compressor.compress(&data).unwrap();
        let ratio = compressed.len() as f64 / data.len() as f64;
        
        // Highly repetitive data should compress < 1%
        assert!(ratio < 0.01);
    }
}

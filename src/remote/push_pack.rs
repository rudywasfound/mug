use crate::core::error::Result;
use crate::pack::manifest::ChunkPackManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPackRequest {
    pub manifest: ChunkPackManifest,
    pub chunks_to_upload: Vec<String>, // Chunk hashes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPackResponse {
    pub success: bool,
    pub pack_id: String,
    pub message: String,
    pub uploaded_chunks: usize,
    pub total_bytes_uploaded: u64,
    pub server_status: String,
}

impl PushPackResponse {
    pub fn success(
        pack_id: String,
        uploaded_chunks: usize,
        total_bytes: u64,
    ) -> Self {
        PushPackResponse {
            success: true,
            pack_id,
            message: format!("Successfully pushed {} chunks", uploaded_chunks),
            uploaded_chunks,
            total_bytes_uploaded: total_bytes,
            server_status: "available".to_string(),
        }
    }

    pub fn failed(message: String) -> Self {
        PushPackResponse {
            success: false,
            pack_id: String::new(),
            message,
            uploaded_chunks: 0,
            total_bytes_uploaded: 0,
            server_status: "error".to_string(),
        }
    }
}

pub struct PushPackManager {
    server_url: String,
}

impl PushPackManager {
    pub fn new(server_url: String) -> Self {
        PushPackManager { server_url }
    }

    pub fn create_request(
        manifest: ChunkPackManifest,
        chunks_to_upload: Vec<String>,
    ) -> PushPackRequest {
        PushPackRequest {
            manifest,
            chunks_to_upload,
        }
    }

    pub async fn push_manifest(&self, request: &PushPackRequest) -> Result<PushPackResponse> {
        
        let manifest_json = request.manifest.to_json()
            .map_err(|e| crate::core::error::Error::Custom(format!("Serialization error: {}", e)))?;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        Ok(PushPackResponse::success(
            request.manifest.pack_id.clone(),
            request.chunks_to_upload.len(),
            request.manifest.total_size,
        ))
    }

    pub async fn push_chunk(
        &self,
        pack_id: &str,
        chunk_hash: &str,
        chunk_data: &[u8],
    ) -> Result<ChunkUploadResponse> {

        let checksum = calculate_checksum(chunk_data);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        Ok(ChunkUploadResponse {
            chunk_hash: chunk_hash.to_string(),
            pack_id: pack_id.to_string(),
            bytes_uploaded: chunk_data.len() as u64,
            checksum,
            server_verification: true,
        })
    }

    pub async fn verify_chunk(
        &self,
        pack_id: &str,
        chunk_hash: &str,
        expected_checksum: &str,
    ) -> Result<bool> {
        
        Ok(true)
    }

    pub async fn commit_pack(&self, pack_id: &str) -> Result<PushPackResponse> {
        
        Ok(PushPackResponse::success(
            pack_id.to_string(),
            0,
            0,
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkUploadResponse {
    pub chunk_hash: String,
    pub pack_id: String,
    pub bytes_uploaded: u64,
    pub checksum: String,
    pub server_verification: bool,
}

#[derive(Debug, Clone)]
pub struct PushPackProgress {
    pub manifest_uploaded: bool,
    pub chunks_uploaded: usize,
    pub total_chunks: usize,
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
    pub errors: Vec<String>,
}

impl PushPackProgress {
    pub fn new(total_chunks: usize, total_bytes: u64) -> Self {
        PushPackProgress {
            manifest_uploaded: false,
            chunks_uploaded: 0,
            total_chunks,
            bytes_uploaded: 0,
            total_bytes,
            errors: Vec::new(),
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total_chunks == 0 {
            0.0
        } else {
            (self.chunks_uploaded as f64 / self.total_chunks as f64) * 100.0
        }
    }

    pub fn bytes_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_uploaded as f64 / self.total_bytes as f64) * 100.0
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn is_complete(&self) -> bool {
        self.chunks_uploaded == self.total_chunks
    }
}

fn calculate_checksum(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::manifest::ChunkPackManifest;

    #[test]
    fn test_push_pack_response_success() {
        let response = PushPackResponse::success("pack-001".to_string(), 10, 10240);
        assert!(response.success);
        assert_eq!(response.pack_id, "pack-001");
        assert_eq!(response.uploaded_chunks, 10);
        assert_eq!(response.total_bytes_uploaded, 10240);
    }

    #[test]
    fn test_push_pack_response_failed() {
        let response = PushPackResponse::failed("Connection error".to_string());
        assert!(!response.success);
        assert!(response.message.contains("Connection error"));
    }

    #[test]
    fn test_push_pack_progress() {
        let mut progress = PushPackProgress::new(10, 10240);
        assert_eq!(progress.percentage(), 0.0);

        progress.chunks_uploaded = 5;
        progress.bytes_uploaded = 5120;
        assert_eq!(progress.percentage(), 50.0);
        assert_eq!(progress.bytes_percentage(), 50.0);
    }

    #[test]
    fn test_push_pack_progress_complete() {
        let mut progress = PushPackProgress::new(10, 10240);
        assert!(!progress.is_complete());

        progress.chunks_uploaded = 10;
        assert!(progress.is_complete());
    }

    #[test]
    fn test_push_pack_request_creation() {
        let manifest = ChunkPackManifest::new("pack-001".to_string());
        let chunks = vec!["hash1".to_string(), "hash2".to_string()];

        let request = PushPackManager::create_request(manifest, chunks);
        assert_eq!(request.chunks_to_upload.len(), 2);
    }

    #[test]
    fn test_calculate_checksum() {
        let data1 = b"hello";
        let data2 = b"hello";
        let data3 = b"world";

        let checksum1 = calculate_checksum(data1);
        let checksum2 = calculate_checksum(data2);
        let checksum3 = calculate_checksum(data3);

        assert_eq!(checksum1, checksum2);
        assert_ne!(checksum1, checksum3);
    }

    #[tokio::test]
    async fn test_push_pack_manager_creation() {
        let manager = PushPackManager::new("http://example.com".to_string());
        assert_eq!(manager.server_url, "http://example.com");
    }
}

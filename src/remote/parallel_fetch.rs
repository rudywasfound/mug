use crate::core::error::Result;
use crate::pack::manifest::{ChunkMetadata, ChunkPackManifest};
use futures::future::join_all;
use std::sync::{Arc, Mutex};
use tokio::task;

#[derive(Debug, Clone)]
pub struct ParallelFetchConfig {
    pub max_concurrent_downloads: usize,
    pub chunk_timeout_secs: u64,
    pub retry_attempts: usize,
    pub verify_checksums: bool,
}

impl Default for ParallelFetchConfig {
    fn default() -> Self {
        ParallelFetchConfig {
            max_concurrent_downloads: 4,
            chunk_timeout_secs: 300,
            retry_attempts: 3,
            verify_checksums: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded_chunks: usize,
    pub total_chunks: usize,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub current_chunk: Option<String>,
}

impl DownloadProgress {
    pub fn new(total_chunks: usize, total_bytes: u64) -> Self {
        DownloadProgress {
            downloaded_chunks: 0,
            total_chunks,
            bytes_downloaded: 0,
            total_bytes,
            current_chunk: None,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total_chunks == 0 {
            0.0
        } else {
            (self.downloaded_chunks as f64 / self.total_chunks as f64) * 100.0
        }
    }

    pub fn bytes_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_downloaded as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkDownloadTask {
    pub chunk: ChunkMetadata,
    pub remote_url: String,
    pub local_path: String,
}

#[derive(Debug, Clone)]
pub struct ChunkDownloadResult {
    pub chunk_hash: String,
    pub success: bool,
    pub bytes_downloaded: u64,
    pub error: Option<String>,
}

pub struct ParallelChunkDownloader {
    config: ParallelFetchConfig,
    progress: Arc<Mutex<DownloadProgress>>,
}

impl ParallelChunkDownloader {
    pub fn new(config: ParallelFetchConfig, manifest: &ChunkPackManifest) -> Self {
        let progress = DownloadProgress::new(manifest.chunk_count, manifest.get_download_size());

        ParallelChunkDownloader {
            config,
            progress: Arc::new(Mutex::new(progress)),
        }
    }

    pub fn with_default_config(manifest: &ChunkPackManifest) -> Self {
        Self::new(ParallelFetchConfig::default(), manifest)
    }

    pub async fn download_chunks(
        &self,
        tasks: Vec<ChunkDownloadTask>,
    ) -> Result<Vec<ChunkDownloadResult>> {
        let config = self.config.clone();
        let progress = Arc::clone(&self.progress);

        let mut results = Vec::new();
        let mut current_batch = Vec::new();

        for task in tasks {
            current_batch.push(task);

            if current_batch.len() >= config.max_concurrent_downloads {
                let batch_results = self.process_batch(current_batch).await?;
                results.extend(batch_results);
                current_batch = Vec::new();
            }
        }

        if !current_batch.is_empty() {
            let batch_results = self.process_batch(current_batch).await?;
            results.extend(batch_results);
        }

        Ok(results)
    }

    async fn process_batch(&self, tasks: Vec<ChunkDownloadTask>) -> Result<Vec<ChunkDownloadResult>> {
        let config = self.config.clone();
        let progress = Arc::clone(&self.progress);

        let futures = tasks.into_iter().map(|task| {
            let config = config.clone();
            let progress = Arc::clone(&progress);

            task::spawn(async move {
                Self::download_chunk_with_retry(task, config, progress).await
            })
        });

        let mut results = Vec::new();
        for handle in join_all(futures).await {
            match handle {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => results.push(ChunkDownloadResult {
                    chunk_hash: "unknown".to_string(),
                    success: false,
                    bytes_downloaded: 0,
                    error: Some(e.to_string()),
                }),
                Err(e) => results.push(ChunkDownloadResult {
                    chunk_hash: "unknown".to_string(),
                    success: false,
                    bytes_downloaded: 0,
                    error: Some(format!("Task error: {}", e)),
                }),
            }
        }

        Ok(results)
    }

    async fn download_chunk_with_retry(
        task: ChunkDownloadTask,
        config: ParallelFetchConfig,
        progress: Arc<Mutex<DownloadProgress>>,
    ) -> Result<ChunkDownloadResult> {
        let mut last_error = None;

        for attempt in 0..config.retry_attempts {
            match Self::download_chunk_internal(&task).await {
                Ok(bytes) => {
                    if let Ok(mut prog) = progress.lock() {
                        prog.downloaded_chunks += 1;
                        prog.bytes_downloaded += bytes;
                        prog.current_chunk = None;
                    }

                    return Ok(ChunkDownloadResult {
                        chunk_hash: task.chunk.hash.clone(),
                        success: true,
                        bytes_downloaded: bytes,
                        error: None,
                    });
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < config.retry_attempts - 1 {
                        let delay = std::time::Duration::from_millis(100 * (2_u64.pow(attempt as u32)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Ok(ChunkDownloadResult {
            chunk_hash: task.chunk.hash.clone(),
            success: false,
            bytes_downloaded: 0,
            error: last_error.map(|e| e.to_string()),
        })
    }

    async fn download_chunk_internal(task: &ChunkDownloadTask) -> Result<u64> {
        let size = task.chunk.compressed_size.unwrap_or(task.chunk.size);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        Ok(size)
    }

    pub fn get_progress(&self) -> Result<DownloadProgress> {
        Ok(self.progress.lock().unwrap().clone())
    }
}

pub struct PackBatchDownloader {
    config: ParallelFetchConfig,
}

impl PackBatchDownloader {
    pub fn new(config: ParallelFetchConfig) -> Self {
        PackBatchDownloader { config }
    }

    pub async fn fetch_manifest(&self, remote_url: &str) -> Result<ChunkPackManifest> {
        Err(crate::core::error::Error::Custom(
            "Fetch manifest not yet implemented".to_string(),
        ))
    }

    pub async fn download_pack(
        &self,
        manifest: &ChunkPackManifest,
        remote_url: &str,
        local_dir: &str,
    ) -> Result<Vec<ChunkDownloadResult>> {
        let downloader = ParallelChunkDownloader::new(self.config.clone(), manifest);

        let tasks: Vec<ChunkDownloadTask> = manifest
            .chunks
            .iter()
            .map(|chunk| ChunkDownloadTask {
                chunk: chunk.clone(),
                remote_url: remote_url.to_string(),
                local_path: format!("{}/{}", local_dir, chunk.hash),
            })
            .collect();

        downloader.download_chunks(tasks).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::manifest::ChunkMetadata;

    #[test]
    fn test_download_progress_creation() {
        let progress = DownloadProgress::new(10, 1024 * 1024);
        assert_eq!(progress.downloaded_chunks, 0);
        assert_eq!(progress.total_chunks, 10);
        assert_eq!(progress.percentage(), 0.0);
    }

    #[test]
    fn test_download_progress_percentage() {
        let mut progress = DownloadProgress::new(10, 1024 * 1024);
        assert_eq!(progress.percentage(), 0.0);

        progress.downloaded_chunks = 5;
        assert_eq!(progress.percentage(), 50.0);

        progress.downloaded_chunks = 10;
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_download_progress_bytes_percentage() {
        let mut progress = DownloadProgress::new(10, 1024);
        assert_eq!(progress.bytes_percentage(), 0.0);

        progress.bytes_downloaded = 512;
        assert_eq!(progress.bytes_percentage(), 50.0);

        progress.bytes_downloaded = 1024;
        assert_eq!(progress.bytes_percentage(), 100.0);
    }

    #[test]
    fn test_chunk_download_result_success() {
        let result = ChunkDownloadResult {
            chunk_hash: "hash1".to_string(),
            success: true,
            bytes_downloaded: 1024,
            error: None,
        };
        assert!(result.success);
        assert_eq!(result.bytes_downloaded, 1024);
    }

    #[test]
    fn test_chunk_download_result_failure() {
        let result = ChunkDownloadResult {
            chunk_hash: "hash1".to_string(),
            success: false,
            bytes_downloaded: 0,
            error: Some("Connection failed".to_string()),
        };
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_parallel_fetch_config_default() {
        let config = ParallelFetchConfig::default();
        assert_eq!(config.max_concurrent_downloads, 4);
        assert_eq!(config.retry_attempts, 3);
        assert!(config.verify_checksums);
    }

    #[tokio::test]
    async fn test_parallel_downloader_creation() {
        let manifest = ChunkPackManifest::new("test-pack".to_string());
        let downloader = ParallelChunkDownloader::with_default_config(&manifest);
        
        let progress = downloader.get_progress().unwrap();
        assert_eq!(progress.total_chunks, 0);
    }
}

use std::sync::Arc;
use tokio::task::JoinHandle;
use futures::stream::{self, StreamExt};
use super::CloneConfig;

/// Parallel cloner for downloading pack files concurrently
pub struct ParallelCloner {
    config: CloneConfig,
}

impl ParallelCloner {
    pub fn new(config: CloneConfig) -> Self {
        ParallelCloner { config }
    }

    /// Clone repository with parallel pack file downloads
    pub async fn clone(&self) -> Result<(), String> {
        println!("Cloning {} to {}", self.config.url, self.config.path);
        
        // Step 1: Fetch pack manifest from server
        let packs = self.fetch_manifest().await?;
        println!("Found {} pack files", packs.len());
        
        // Step 2: Download packs in parallel
        self.download_packs_parallel(&packs).await?;
        
        println!("Clone complete!");
        Ok(())
    }

    /// Fetch list of available pack files from server
    async fn fetch_manifest(&self) -> Result<Vec<String>, String> {
        // In real impl: GET {url}/.mug/manifest.json
        // For now, placeholder
        Ok(vec![
            "pack-001.mug".to_string(),
            "pack-002.mug".to_string(),
        ])
    }

    /// Download multiple packs concurrently
    async fn download_packs_parallel(&self, packs: &[String]) -> Result<(), String> {
        let tasks: Vec<JoinHandle<Result<(), String>>> = packs
            .iter()
            .take(self.config.num_workers)
            .map(|pack| {
                let url = self.config.url.clone();
                let path = self.config.path.clone();
                let pack_name = pack.clone();
                
                tokio::spawn(async move {
                    Self::download_pack(&url, &path, &pack_name).await
                })
            })
            .collect();

        for task in tasks {
            task.await.map_err(|e| e.to_string())??;
        }

        Ok(())
    }

    /// Download single pack file
    async fn download_pack(
        url: &str,
        path: &str,
        pack_name: &str,
    ) -> Result<(), String> {
        let pack_url = format!("{}/{}", url, pack_name);
        println!("Downloading {}", pack_name);
        
        // In real impl: fetch from pack_url with progress
        // For now, placeholder
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clone_config() {
        let config = CloneConfig::new("https://example.com/repo", "/tmp/repo");
        assert_eq!(config.url, "https://example.com/repo");
        assert_eq!(config.num_workers, num_cpus::get());
    }
}

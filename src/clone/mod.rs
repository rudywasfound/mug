pub mod parallel;

pub use parallel::ParallelCloner;

/// Clone configuration
pub struct CloneConfig {
    pub url: String,
    pub path: String,
    pub num_workers: usize,
    pub chunk_size: usize,
}

impl CloneConfig {
    pub fn new(url: &str, path: &str) -> Self {
        CloneConfig {
            url: url.to_string(),
            path: path.to_string(),
            num_workers: num_cpus::get(),
            chunk_size: 64 * 1024, // 64KB chunks
        }
    }
}

use crate::commit::Commit;
use crate::store::{Blob, Tree};
use serde::{Deserialize, Serialize};

/// Unified remote protocol for HTTP/HTTPS/SSH
///
/// All transports use the same message format (JSON over HTTP/HTTPS, binary over SSH)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    /// Repository name
    pub repo: String,
    /// Branch name
    pub branch: String,
    /// Commit objects being pushed
    pub commits: Vec<Commit>,
    /// Blob objects being pushed
    pub blobs: Vec<Blob>,
    /// Tree objects being pushed
    pub trees: Vec<Tree>,
    /// Current branch head
    pub head: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    /// Success indicator
    pub success: bool,
    /// Status message
    pub message: String,
    /// New head after push
    pub head: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// Repository name
    pub repo: String,
    /// Branch name
    pub branch: String,
    /// Current known head
    pub current_head: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    /// Success indicator
    pub success: bool,
    /// Commit objects to apply
    pub commits: Vec<Commit>,
    /// Blob objects to apply
    pub blobs: Vec<Blob>,
    /// Tree objects to apply
    pub trees: Vec<Tree>,
    /// New head after pull
    pub head: String,
    /// Status message
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRequest {
    /// Repository name
    pub repo: String,
    /// Fetch all branches or specific branch
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResponse {
    /// Success indicator
    pub success: bool,
    /// All available branches and their heads
    pub branches: std::collections::HashMap<String, String>,
    /// Status message
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneRequest {
    /// Repository name
    pub repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneResponse {
    /// All commit objects
    pub commits: Vec<Commit>,
    /// All blob objects
    pub blobs: Vec<Blob>,
    /// All tree objects
    pub trees: Vec<Tree>,
    /// All branches with their heads
    pub branches: std::collections::HashMap<String, String>,
    /// Default branch
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

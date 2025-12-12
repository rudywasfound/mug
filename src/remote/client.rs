use crate::core::error::{Error, Result};
use crate::remote::protocol::{
    CloneRequest, CloneResponse, FetchRequest, FetchResponse, PullRequest, PullResponse,
    PushRequest, PushResponse,
};
use crate::remote::{Protocol, Remote};
use crate::core::repo::Repository;
use reqwest::Client;

/// Remote client for push/pull/fetch/clone operations with HTTP transport
pub struct RemoteClient {
    client: Client,
}

impl RemoteClient {
    /// Create a new remote client
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }

    /// Push to remote repository
    pub async fn push(
        &self,
        remote: &Remote,
        repo: &Repository,
        branch: &str,
        _token: &str,
    ) -> Result<PushResponse> {
        // Only HTTP(S) supported in this version
        if remote.protocol != Protocol::Http && remote.protocol != Protocol::Https {
            return Err(Error::Custom(
                "SSH transport not yet implemented".to_string(),
            ));
        }

        // Get commits to push
        let commits_str = repo.log()?;
        if commits_str.is_empty() {
            return Ok(PushResponse {
                success: false,
                message: "No commits to push".to_string(),
                head: None,
            });
        }

        // Convert string commit IDs to Commit objects (placeholder)
        let commits = commits_str
            .into_iter()
            .map(|id| crate::core::commit::Commit {
                id: id.clone(),
                tree_hash: String::new(),
                parent: None,
                author: String::new(),
                message: String::new(),
                timestamp: String::new(),
            })
            .collect();

        // Extract repo name from URL
        let repo_name = extract_repo_name(&remote.url).unwrap_or_else(|| "repo".to_string());

        // Gather blobs from repository
        let blobs = gather_repository_blobs(repo).unwrap_or_default();

        // Gather trees from repository
        let trees = gather_repository_trees(repo).unwrap_or_default();

        // Build request
        let request = PushRequest {
            repo: repo_name,
            branch: branch.to_string(),
            commits,
            blobs,
            trees,
            head: "HEAD".to_string(),
        };

        // Send push request
        let url = format!("{}/repo/push", remote.url.trim_end_matches('/'));
        match self.client.post(&url).json(&request).send().await {
            Ok(response) => match response.json::<PushResponse>().await {
                Ok(resp) => Ok(resp),
                Err(e) => Err(Error::Custom(format!(
                    "Failed to parse push response: {}",
                    e
                ))),
            },
            Err(e) => Err(Error::Custom(format!("Push failed: {}", e))),
        }
    }

    /// Pull from remote repository
    pub async fn pull(
        &self,
        remote: &Remote,
        _repo: &Repository,
        branch: &str,
        _token: &str,
    ) -> Result<PullResponse> {
        // Only HTTP(S) supported in this version
        if remote.protocol != Protocol::Http && remote.protocol != Protocol::Https {
            return Err(Error::Custom(
                "SSH transport not yet implemented".to_string(),
            ));
        }

        // Get current head (placeholder)
        let current_head = Some("HEAD".to_string());

        // Extract repo name
        let repo_name = extract_repo_name(&remote.url).unwrap_or_else(|| "repo".to_string());

        // Build request
        let request = PullRequest {
            repo: repo_name,
            branch: branch.to_string(),
            current_head,
        };

        // Send pull request
        let url = format!("{}/repo/pull", remote.url.trim_end_matches('/'));
        match self.client.get(&url).json(&request).send().await {
            Ok(response) => match response.json::<PullResponse>().await {
                Ok(resp) => Ok(resp),
                Err(e) => Err(Error::Custom(format!(
                    "Failed to parse pull response: {}",
                    e
                ))),
            },
            Err(e) => Err(Error::Custom(format!("Pull failed: {}", e))),
        }
    }

    /// Fetch from remote repository
    pub async fn fetch(
        &self,
        remote: &Remote,
        _branch: Option<&str>,
        _token: &str,
    ) -> Result<FetchResponse> {
        // Only HTTP(S) supported in this version
        if remote.protocol != Protocol::Http && remote.protocol != Protocol::Https {
            return Err(Error::Custom(
                "SSH transport not yet implemented".to_string(),
            ));
        }

        // Extract repo name
        let repo_name = extract_repo_name(&remote.url).unwrap_or_else(|| "repo".to_string());

        // Build request
        let request = FetchRequest {
            repo: repo_name,
            branch: _branch.map(|s| s.to_string()),
        };

        // Send fetch request
        let url = format!("{}/repo/fetch", remote.url.trim_end_matches('/'));
        match self.client.get(&url).json(&request).send().await {
            Ok(response) => match response.json::<FetchResponse>().await {
                Ok(resp) => Ok(resp),
                Err(e) => Err(Error::Custom(format!(
                    "Failed to parse fetch response: {}",
                    e
                ))),
            },
            Err(e) => Err(Error::Custom(format!("Fetch failed: {}", e))),
        }
    }

    /// Clone a repository
    pub async fn clone(&self, remote: &Remote, _dest: &str, _token: &str) -> Result<CloneResponse> {
        // Only HTTP(S) supported in this version
        if remote.protocol != Protocol::Http && remote.protocol != Protocol::Https {
            return Err(Error::Custom(
                "SSH transport not yet implemented".to_string(),
            ));
        }

        // Extract repo name
        let repo_name = extract_repo_name(&remote.url).unwrap_or_else(|| "repo".to_string());

        // Build request
        let request = CloneRequest { repo: repo_name };

        // Send clone request
        let url = format!("{}/repo/clone", remote.url.trim_end_matches('/'));
        match self.client.get(&url).json(&request).send().await {
            Ok(response) => match response.json::<CloneResponse>().await {
                Ok(resp) => Ok(resp),
                Err(e) => Err(Error::Custom(format!(
                    "Failed to parse clone response: {}",
                    e
                ))),
            },
            Err(e) => Err(Error::Custom(format!("Clone failed: {}", e))),
        }
    }

    /// Test connection to remote
    pub async fn test_connection(&self, remote: &Remote) -> Result<bool> {
        if remote.protocol != Protocol::Http && remote.protocol != Protocol::Https {
            return Err(Error::Custom(
                "SSH transport not yet implemented".to_string(),
            ));
        }

        let url = format!("{}/health", remote.url.trim_end_matches('/'));
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Build correct client based on protocol
pub async fn build_remote_client(remote: &Remote) -> Result<RemoteClient> {
    match remote.protocol {
        Protocol::Http | Protocol::Https => RemoteClient::new(),
        Protocol::Ssh => Err(Error::Custom("SSH support coming in v1.1.0".to_string())),
    }
}

/// Extract repository name from URL
fn extract_repo_name(url: &str) -> Option<String> {
    // Handle URLs like:
    // https://example.com/repo -> repo
    // https://example.com/repo/ -> repo
    // https://example.com/repo.git -> repo

    let url = url.trim_end_matches('/');

    // Remove .git suffix if present
    let url = if url.ends_with(".git") {
        &url[..url.len() - 4]
    } else {
        url
    };

    // Get the last component after the domain
    url.split('/').last().map(|s| s.to_string())
}

/// Gather all blobs from repository object store
fn gather_repository_blobs(_repo: &Repository) -> Result<Vec<crate::core::store::Blob>> {
    let blobs = Vec::new();

    // Iterate through all objects in store and collect blobs
    // For now, return empty vector - full implementation would require
    // iterating through the .mug/objects directory and deserializing stored blobs
    // This would require database iteration support or walking the filesystem
    
    Ok(blobs)
}

/// Gather all trees from repository object store
fn gather_repository_trees(_repo: &Repository) -> Result<Vec<crate::core::store::Tree>> {
    let trees = Vec::new();

    // Iterate through all objects in store and collect trees
    // Trees are stored in the object store, so we'd need to iterate through
    // the .mug/objects directory and deserialize tree objects
    // For now, return empty vector - requires database iteration support
    
    Ok(trees)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("https://example.com/repo"),
            Some("repo".to_string())
        );
        assert_eq!(
            extract_repo_name("https://example.com/repo/"),
            Some("repo".to_string())
        );
        assert_eq!(
            extract_repo_name("https://example.com/repo.git"),
            Some("repo".to_string())
        );
        assert_eq!(
            extract_repo_name("https://example.com/myorg/myrepo"),
            Some("myrepo".to_string())
        );
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::Result;
use crate::database::MugDb;

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Token/API key for server authentication
    pub token: String,
    /// Optional username for display
    pub username: Option<String>,
    /// Remote name these credentials are for
    pub remote: String,
}

/// Authentication manager
pub struct AuthManager {
    db: MugDb,
}

impl AuthManager {
    pub fn new(db: MugDb) -> Self {
        Self { db }
    }

    /// Store credentials for a remote
    pub fn save_credentials(&self, remote: &str, token: &str, username: Option<&str>) -> Result<()> {
        let creds = Credentials {
            token: token.to_string(),
            username: username.map(|u| u.to_string()),
            remote: remote.to_string(),
        };

        self.db.set(
            "auth",
            remote,
            &serde_json::to_string(&creds)?.into_bytes(),
        )?;
        Ok(())
    }

    /// Get credentials for a remote
    pub fn get_credentials(&self, remote: &str) -> Result<Option<Credentials>> {
        match self.db.get("auth", remote)? {
            Some(data) => {
                let creds: Credentials = serde_json::from_slice(&data)?;
                Ok(Some(creds))
            }
            None => Ok(None),
        }
    }

    /// Remove credentials for a remote
    pub fn delete_credentials(&self, remote: &str) -> Result<()> {
        self.db.delete("auth", remote)?;
        Ok(())
    }

    /// Generate a new API token (server-side)
    pub fn generate_token() -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }

    /// Validate token format
    pub fn validate_token_format(token: &str) -> bool {
        // Tokens must be non-empty and reasonable length
        !token.is_empty() && token.len() > 10
    }
}

/// Server-side auth store
pub struct ServerAuth {
    // Map of token -> (username, permissions)
    tokens: HashMap<String, TokenInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub username: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    Read(String),    // Read from repo
    Write(String),   // Write to repo
    Admin(String),   // Full access to repo
}

impl ServerAuth {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// Add a token
    pub fn add_token(&mut self, token: String, username: String, permissions: Vec<Permission>) {
        self.tokens.insert(token, TokenInfo { username, permissions });
    }

    /// Verify token and check permission
    pub fn verify(&self, token: &str, repo: &str, action: &str) -> Result<bool> {
        match self.tokens.get(token) {
            Some(info) => {
                let has_permission = info.permissions.iter().any(|p| {
                    match p {
                        Permission::Admin(r) => r == repo,
                        Permission::Write(r) if action == "write" => r == repo,
                        Permission::Read(r) if action == "read" => r == repo,
                        _ => false,
                    }
                });

                Ok(has_permission)
            }
            None => Ok(false),
        }
    }

    /// Get token info
    pub fn get_token_info(&self, token: &str) -> Option<TokenInfo> {
        self.tokens.get(token).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_token() {
        assert!(AuthManager::validate_token_format("valid_token_12345"));
        assert!(!AuthManager::validate_token_format("short"));
        assert!(!AuthManager::validate_token_format(""));
    }

    #[test]
    fn test_server_auth() {
        let mut auth = ServerAuth::new();
        let token = AuthManager::generate_token();
        
        auth.add_token(
            token.clone(),
            "testuser".to_string(),
            vec![Permission::Read("repo1".to_string())],
        );

        assert!(auth.verify(&token, "repo1", "read").unwrap());
        assert!(!auth.verify(&token, "repo1", "write").unwrap());
        assert!(!auth.verify(&token, "repo2", "read").unwrap());
    }
}

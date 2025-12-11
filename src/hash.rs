use crate::error::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Hash a byte slice using SHA256
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Hash a file's contents
pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let contents = fs::read(path)?;
    Ok(hash_bytes(&contents))
}

/// Hash a string
pub fn hash_str(s: &str) -> String {
    hash_bytes(s.as_bytes())
}

/// Shorten a hash to 7 characters (like git)
pub fn short_hash(hash: &str) -> String {
    hash.chars().take(7).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let hash = hash_bytes(b"test");
        assert_eq!(hash.len(), 64); // SHA256 is 64 hex chars
    }

    #[test]
    fn test_hash_str() {
        let hash = hash_str("test");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_short_hash() {
        let hash = hash_str("test");
        assert_eq!(short_hash(&hash).len(), 7);
    }
}

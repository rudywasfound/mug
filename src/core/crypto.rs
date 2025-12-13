/// Cryptographic signing and verification for commits
use crate::core::error::Result;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoKey {
    /// Base64-encoded public key for verification
    pub public_key: String,
    /// Base64-encoded seed (only stored locally)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedCommit {
    pub id: String,
    pub tree_hash: String,
    pub parent: Option<String>,
    pub author: String,
    pub message: String,
    pub timestamp: String,
    /// Base64-encoded Ed25519 signature
    pub signature: String,
    /// Base64-encoded public key of signer
    pub signer_key: String,
}

impl CryptoKey {
    /// Generate a new keypair
    pub fn generate() -> Result<(CryptoKey, String)> {
        use rand::RngCore;
        let mut seed = [0u8; 32];
        thread_rng().fill_bytes(&mut seed);
        
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let public_bytes = verifying_key.to_bytes();

        Ok((
            CryptoKey {
                public_key: base64::encode(public_bytes),
                seed: Some(base64::encode(&seed)),
            },
            base64::encode(public_bytes),
        ))
    }

    /// Import from seed
    pub fn from_seed(seed: &str) -> Result<Self> {
        let seed_bytes = base64::decode(seed)
            .map_err(|e| crate::core::error::Error::Custom(format!("Invalid seed: {}", e)))?;
        
        if seed_bytes.len() != 32 {
            return Err(crate::core::error::Error::Custom(
                "Seed must be 32 bytes".to_string(),
            ));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&seed_bytes);
        let signing_key = SigningKey::from_bytes(&array);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let public_bytes = verifying_key.to_bytes();

        Ok(CryptoKey {
            public_key: base64::encode(public_bytes),
            seed: Some(seed.to_string()),
        })
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<String> {
        let seed = self
            .seed
            .as_ref()
            .ok_or(crate::core::error::Error::Custom(
                "Cannot sign without seed".to_string(),
            ))?;

        let seed_bytes = base64::decode(seed)
            .map_err(|e| crate::core::error::Error::Custom(format!("Invalid seed: {}", e)))?;

        let mut array = [0u8; 32];
        array.copy_from_slice(&seed_bytes);
        let signing_key = SigningKey::from_bytes(&array);

        let signature = signing_key.sign(message);
        Ok(base64::encode(signature.to_bytes()))
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &str) -> Result<bool> {
        let public_bytes_vec = base64::decode(&self.public_key)
            .map_err(|e| crate::core::error::Error::Custom(format!("Invalid public key: {}", e)))?;

        let sig_bytes = base64::decode(signature)
            .map_err(|e| crate::core::error::Error::Custom(format!("Invalid signature: {}", e)))?;

        if sig_bytes.len() != 64 {
            return Ok(false);
        }

        let mut public_bytes = [0u8; 32];
        public_bytes.copy_from_slice(&public_bytes_vec);
        
        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);

        let verifying_key = VerifyingKey::from_bytes(&public_bytes)
            .map_err(|e| crate::core::error::Error::Custom(format!("Invalid public key: {}", e)))?;

        let sig = Signature::from_bytes(&sig_array);

        Ok(verifying_key.verify(message, &sig).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let (key, _pub) = CryptoKey::generate().unwrap();
        assert!(!key.public_key.is_empty());
        assert!(key.seed.is_some());
    }

    #[test]
    fn test_sign_and_verify() {
        let (key, _) = CryptoKey::generate().unwrap();
        let message = b"hello world";

        let signature = key.sign(message).unwrap();
        let verified = key.verify(message, &signature).unwrap();

        assert!(verified);
    }

    #[test]
    fn test_verify_invalid_signature() {
        let (key, _) = CryptoKey::generate().unwrap();
        let message = b"hello world";
        let wrong_message = b"goodbye world";

        let signature = key.sign(message).unwrap();
        let verified = key.verify(wrong_message, &signature).unwrap();

        assert!(!verified);
    }
}

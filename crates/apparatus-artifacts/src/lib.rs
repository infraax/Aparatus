//! Immutable artifact management.
//!
//! Provides the canonical artifact concepts and pure helpers only.
//! It does not write to the CAS filesystem directly yet.

use apparatus_crypto::{CryptoError, Sha256Digest};
use apparatus_types::ObjectHeader;
use std::path::PathBuf;
use thiserror::Error;

/// Artifact-related errors.
#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
}

/// Immutable descriptor for an artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactDescriptor {
    /// Standard metadata for this artifact.
    pub header: ObjectHeader,
    /// The SHA-256 digest of the artifact's raw content.
    pub digest: Sha256Digest,
    /// The size in bytes of the artifact.
    pub size_bytes: u64,
}

/// Calculates the safe, relative CAS path from a digest.
///
/// Follows the format: `sha256/ab/cd/<full-lowercase-hash>`
pub fn derive_cas_path(digest: &Sha256Digest) -> PathBuf {
    let hex = format!("{:x}", digest);
    let mut path = PathBuf::from("sha256");
    path.push(&hex[0..2]);
    path.push(&hex[2..4]);
    path.push(hex);
    path
}

/// Validates a CAS path and extracts the hash.
pub fn validate_and_extract_hash_from_cas_path(path: &str) -> Result<Sha256Digest, ArtifactError> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 4 || parts[0] != "sha256" {
        return Err(ArtifactError::Crypto(CryptoError::InvalidLength { expected: 4, actual: parts.len() })); // roughly indicating format error
    }

    let hex = parts[3];
    if hex.len() != 64 || &hex[0..2] != parts[1] || &hex[2..4] != parts[2] {
        return Err(ArtifactError::Crypto(CryptoError::InvalidHex));
    }

    Ok(Sha256Digest::from_hex(hex)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cas_path_derivation() {
        // "hello world" hash
        let digest = Sha256Digest::from_hex("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9").unwrap();
        let path = derive_cas_path(&digest);
        assert_eq!(path.to_str().unwrap(), "sha256/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_cas_path_validation() {
        let valid_path = "sha256/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let digest = validate_and_extract_hash_from_cas_path(valid_path).unwrap();
        assert_eq!(format!("{:x}", digest), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");

        assert!(validate_and_extract_hash_from_cas_path("sha256/b9/4e/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9").is_err());
        assert!(validate_and_extract_hash_from_cas_path("sha256/b9/4d/short").is_err());
        assert!(validate_and_extract_hash_from_cas_path("md5/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9").is_err());
    }
}

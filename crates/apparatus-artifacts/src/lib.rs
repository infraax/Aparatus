//! Immutable artifact management.
//!
//! Provides the canonical artifact concepts and pure helpers only.
//! It does not write to the CAS filesystem directly yet.

use apparatus_crypto::{CryptoError, Sha256Digest};
use apparatus_types::ObjectHeader;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

/// Artifact-related errors.
#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("Artifact path is absolute; must be relative")]
    AbsolutePath,
    #[error("Artifact path contains unsupported directory separators")]
    InvalidSeparator,
    #[error("Artifact path component count mismatch, expected {0}")]
    InvalidComponentCount(usize),
    #[error("Artifact path has incorrect prefix, expected {expected}")]
    InvalidPrefix { expected: String },
    #[error("Artifact path shard mismatch")]
    ShardMismatch,
    #[error("Artifact path contains traversals like '.' or '..'")]
    PathTraversal,
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
    if path.contains('\\') {
        return Err(ArtifactError::InvalidSeparator);
    }
    if path.starts_with('/') {
        return Err(ArtifactError::AbsolutePath);
    }
    if path.contains("/../")
        || path.contains("/./")
        || path.ends_with("/..")
        || path.ends_with("/.")
        || path.starts_with("../")
        || path.starts_with("./")
    {
        return Err(ArtifactError::PathTraversal);
    }

    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 4 {
        return Err(ArtifactError::InvalidComponentCount(4));
    }
    if parts[0] != "sha256" {
        return Err(ArtifactError::InvalidPrefix {
            expected: "sha256".to_string(),
        });
    }

    let hex = parts[3];
    let digest = Sha256Digest::from_str(hex)?;

    if &hex[0..2] != parts[1] || &hex[2..4] != parts[2] {
        return Err(ArtifactError::ShardMismatch);
    }

    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cas_path_derivation() {
        let digest = Sha256Digest::from_str(
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        )
        .unwrap();
        let path = derive_cas_path(&digest);
        assert_eq!(
            path.to_str().unwrap(),
            "sha256/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_cas_path_validation() {
        let valid_path =
            "sha256/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let digest = validate_and_extract_hash_from_cas_path(valid_path).unwrap();
        assert_eq!(
            format!("{:x}", digest),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        assert!(matches!(
            validate_and_extract_hash_from_cas_path(
                "/sha256/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            ),
            Err(ArtifactError::AbsolutePath)
        ));
        assert!(matches!(
            validate_and_extract_hash_from_cas_path(
                "sha256\\b9\\4d\\b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            ),
            Err(ArtifactError::InvalidSeparator)
        ));
        assert!(matches!(validate_and_extract_hash_from_cas_path("sha256/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9/extra"), Err(ArtifactError::InvalidComponentCount(_))));
        assert!(matches!(
            validate_and_extract_hash_from_cas_path(
                "sha256/../b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            ),
            Err(ArtifactError::PathTraversal)
        ));
        assert!(matches!(
            validate_and_extract_hash_from_cas_path(
                "sha256/b9/4e/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            ),
            Err(ArtifactError::ShardMismatch)
        ));
        assert!(matches!(
            validate_and_extract_hash_from_cas_path("sha256/b9/4d/short"),
            Err(ArtifactError::Crypto(CryptoError::InvalidLength { .. }))
        ));
        assert!(matches!(
            validate_and_extract_hash_from_cas_path(
                "md5/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            ),
            Err(ArtifactError::InvalidPrefix { .. })
        ));
    }
}

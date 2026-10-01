//! Immutable artifact management.
//!
//! Provides the canonical artifact concepts, CAS path helpers, and
//! `FsArtifactStore`: a write-once filesystem content-addressed store.

use apparatus_crypto::{CryptoError, Sha256Digest};
use apparatus_store::{ArtifactStore, StoreError};
use apparatus_types::{ArtifactId, ObjectHeader};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
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
        return Err(ArtifactError::Crypto(CryptoError::InvalidLength {
            expected: 4,
            actual: parts.len(),
        })); // roughly indicating format error
    }

    let hex = parts[3];
    if hex.len() != 64 || &hex[0..2] != parts[1] || &hex[2..4] != parts[2] {
        return Err(ArtifactError::Crypto(CryptoError::InvalidHex));
    }

    Ok(Sha256Digest::from_hex(hex)?)
}

/// Write-once content-addressed store rooted at a directory (normally `.apparatus/cas`).
///
/// Bytes live at `<root>/sha256/ab/cd/<hex>`; the id→digest pointer lives at
/// `<root>/ids/<artifact-id>`. Neither is ever overwritten.
#[derive(Debug, Clone)]
pub struct FsArtifactStore {
    root: PathBuf,
}

impl FsArtifactStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Absolute path of the blob for `digest`.
    pub fn blob_path(&self, digest: &Sha256Digest) -> PathBuf {
        self.root.join(derive_cas_path(digest))
    }

    fn pointer_path(&self, id: ArtifactId) -> PathBuf {
        self.root.join("ids").join(id.to_string())
    }

    /// Store bytes and return their digest. Fails if the id or the digest exists.
    pub fn put(&self, id: ArtifactId, data: &[u8]) -> Result<Sha256Digest, StoreError> {
        let digest = Sha256Digest::compute(data);
        let pointer = self.pointer_path(id);
        if pointer.exists() {
            return Err(StoreError::ArtifactAlreadyExists(id));
        }
        let blob = self.blob_path(&digest);
        if blob.exists() {
            return Err(StoreError::DigestAlreadyExists(format!("{digest:x}")));
        }
        write_new(&blob, data)?;
        write_new(&pointer, format!("{digest:x}\n").as_bytes())?;
        Ok(digest)
    }

    /// Digest recorded for `id`.
    pub fn digest_of(&self, id: ArtifactId) -> Result<Sha256Digest, StoreError> {
        let pointer = self.pointer_path(id);
        if !pointer.exists() {
            return Err(StoreError::ArtifactNotFound(id));
        }
        let hex = fs::read_to_string(pointer)?;
        Sha256Digest::from_hex(hex.trim()).map_err(|e| StoreError::Backend(e.to_string()))
    }

    /// True when the blob for `digest` exists and its bytes hash to `digest`.
    pub fn verify(&self, digest: &Sha256Digest) -> Result<bool, StoreError> {
        let blob = self.blob_path(digest);
        if !blob.exists() {
            return Ok(false);
        }
        Ok(Sha256Digest::compute(&fs::read(blob)?) == *digest)
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

impl ArtifactStore for FsArtifactStore {
    fn store(&self, id: ArtifactId, data: &[u8]) -> Result<(), StoreError> {
        self.put(id, data).map(|_| ())
    }

    fn load(&self, id: ArtifactId) -> Result<Vec<u8>, StoreError> {
        let digest = self.digest_of(id)?;
        let bytes = fs::read(self.blob_path(&digest))?;
        if Sha256Digest::compute(&bytes) != digest {
            return Err(StoreError::Backend(format!(
                "blob for {id} fails its digest"
            )));
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apparatus_types::ObjectId;

    #[test]
    fn fs_store_is_write_once_by_id_and_digest() {
        let root = std::env::temp_dir().join(format!("apparatus-cas-{}", ObjectId::new_v7()));
        let store = FsArtifactStore::new(&root);
        let id = ArtifactId::new();
        store.store(id, b"hello world").unwrap();
        assert_eq!(store.load(id).unwrap(), b"hello world");
        assert!(store
            .blob_path(&Sha256Digest::compute(b"hello world"))
            .ends_with(
                "sha256/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            ));

        // Same id again: refused.
        assert!(matches!(
            store.store(id, b"other"),
            Err(StoreError::ArtifactAlreadyExists(_))
        ));
        // Same bytes under a new id: refused.
        assert!(matches!(
            store.store(ArtifactId::new(), b"hello world"),
            Err(StoreError::DigestAlreadyExists(_))
        ));
        assert!(store
            .verify(&Sha256Digest::compute(b"hello world"))
            .unwrap());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn test_cas_path_derivation() {
        // "hello world" hash
        let digest = Sha256Digest::from_hex(
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

        assert!(validate_and_extract_hash_from_cas_path(
            "sha256/b9/4e/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        )
        .is_err());
        assert!(validate_and_extract_hash_from_cas_path("sha256/b9/4d/short").is_err());
        assert!(validate_and_extract_hash_from_cas_path(
            "md5/b9/4d/b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        )
        .is_err());
    }
}

//! Core persistence traits for Apparatus.
//!
//! Note: This crate defines narrow traits. The concrete implementations
//! (e.g. SQLite through `rusqlite` or direct CAS filesystem access) are
//! deferred to future phases to keep the foundation minimal.

use apparatus_types::{ArtifactId, ReceiptId};
use thiserror::Error;

/// Storage-related errors.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Attempted to write an artifact that already exists.
    #[error("Artifact {0:?} already exists")]
    ArtifactAlreadyExists(ArtifactId),
    /// Attempted to read an artifact that does not exist.
    #[error("Artifact {0:?} not found")]
    ArtifactNotFound(ArtifactId),
    /// A generic backend error.
    #[error("Backend error: {0}")]
    Backend(String),
    /// Content with the same digest is already stored (immutable, no overwrite).
    #[error("Content {0} already exists")]
    DigestAlreadyExists(String),
    /// Append refused because the receipt does not extend the current head.
    #[error("Chain mismatch: expected previous {expected}, got {actual}")]
    ChainMismatch { expected: String, actual: String },
    /// The requested receipt does not exist.
    #[error("Receipt {0} not found")]
    ReceiptNotFound(String),
    /// Filesystem failure.
    #[error("I/O error: {0}")]
    Io(String),
    /// Another writer holds the store lock.
    #[error("Locked: {0}")]
    Locked(String),
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e.to_string())
    }
}

/// A transactional boundary representing a single unit of atomic work.
pub trait Transaction {
    /// Commit the transaction.
    fn commit(self) -> Result<(), StoreError>;

    /// Rollback the transaction.
    fn rollback(self) -> Result<(), StoreError>;
}

/// A read-only/append-only abstraction over the raw artifact store.
///
/// M1 implementation: `apparatus_artifacts::FsArtifactStore` (filesystem CAS).
pub trait ArtifactStore {
    /// Store bytes under a specific ID. Must fail if the artifact already exists (immutable).
    fn store(&self, id: ArtifactId, data: &[u8]) -> Result<(), StoreError>;

    /// Load an artifact by ID.
    fn load(&self, id: ArtifactId) -> Result<Vec<u8>, StoreError>;
}

/// A ledger storing cryptographic receipts for operations.
///
/// M1 implementation: `apparatus_ledger::FileLedger` (append-only JSONL + HEAD).
/// `append` must fail when the receipt does not extend the current head.
pub trait ReceiptLedger {
    /// Append a receipt to the ledger.
    fn append(&self, id: ReceiptId, receipt_bytes: &[u8]) -> Result<(), StoreError>;

    /// Retrieve a receipt by its ID.
    fn get(&self, id: ReceiptId) -> Result<Vec<u8>, StoreError>;
}

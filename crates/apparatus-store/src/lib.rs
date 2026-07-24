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
/// Future concrete implementation will likely write directly to a local CAS filesystem.
pub trait ArtifactStore {
    /// Store bytes under a specific ID. Must fail if the artifact already exists (immutable).
    fn store(&self, id: ArtifactId, data: &[u8]) -> Result<(), StoreError>;

    /// Load an artifact by ID.
    fn load(&self, id: ArtifactId) -> Result<Vec<u8>, StoreError>;
}

/// A ledger storing cryptographic receipts for operations.
///
/// Future concrete implementation will write to a SQLite table mapping receipts to operations.
pub trait ReceiptLedger {
    /// Append a receipt to the ledger.
    fn append(&self, id: ReceiptId, receipt_bytes: &[u8]) -> Result<(), StoreError>;

    /// Retrieve a receipt by its ID.
    fn get(&self, id: ReceiptId) -> Result<Vec<u8>, StoreError>;
}

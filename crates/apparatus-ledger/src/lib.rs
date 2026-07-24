//! Append-only receipt ledger for Apparatus.
//!
//! Provides receipt domain types and a deterministic receipt-hash input function.
//! Note: cryptographic chain verification is not complete as storage is not yet implemented.

use apparatus_crypto::Sha256Digest;
use apparatus_types::{ObjectHeader, ReceiptId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Ledger-related errors.
#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A receipt recording a mutation or state transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    /// Standard object header identifying this receipt.
    pub header: ObjectHeader,
    /// The ID of the receipt that immediately precedes this one (if any).
    pub previous_receipt_id: Option<ReceiptId>,
    /// The cryptographic hash of the prior receipt, forming a chain.
    pub previous_hash: Option<Sha256Digest>,
    /// Opaque JSON describing the mutation applied.
    pub operation_payload: serde_json::Value,
}

impl Receipt {
    /// Deterministically hash the receipt by serializing its fields.
    pub fn compute_hash(&self) -> Result<Sha256Digest, LedgerError> {
        // Serde JSON's to_vec does not guarantee strict canonicalization (e.g., key ordering)
        // by default unless a BTreeMap is used at the struct level, but for this foundational
        // phase we assume stable serialization for the struct fields and rely on payload stability.
        // For a full system, a proper canonical JSON encoding should be used.
        let bytes = serde_json::to_vec(self)?;
        Ok(Sha256Digest::compute(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apparatus_types::{Classification, ObjectId, ProjectId};

    #[test]
    fn test_deterministic_receipt_hash() {
        let header = ObjectHeader {
            id: ObjectId::new_v7(),
            created_at: 1000,
            project_id: ProjectId::new(),
            classification: Classification::Public,
            provenance: None,
            correlation_id: None,
        };

        let receipt1 = Receipt {
            header: header.clone(),
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload: serde_json::json!({ "type": "create", "target": "artifact" }),
        };

        let receipt2 = Receipt {
            header: header.clone(),
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload: serde_json::json!({ "type": "create", "target": "artifact" }),
        };

        // Same struct content should produce identical hashes.
        assert_eq!(
            receipt1.compute_hash().unwrap(),
            receipt2.compute_hash().unwrap()
        );
    }
}

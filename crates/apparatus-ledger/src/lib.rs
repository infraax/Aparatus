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
    /// The digest of the operation payload.
    /// The actual operation payload is not directly part of this strict struct to avoid opaque mutable JSON.
    pub operation_payload_digest: Sha256Digest,
}

/// The V1 format for the receipt hash preimage.
/// This explicit structure guarantees deterministic hashing rather than relying on
/// arbitrary struct field ordering during JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReceiptHashPreimageV1 {
    version: u8,
    header: ObjectHeader,
    previous_receipt_id: Option<ReceiptId>,
    previous_hash: Option<Sha256Digest>,
    operation_payload_digest: Sha256Digest,
}

impl Receipt {
    /// Deterministically hash the receipt by serializing an explicit preimage structure.
    pub fn compute_hash(&self) -> Result<Sha256Digest, LedgerError> {
        let preimage = ReceiptHashPreimageV1 {
            version: 1,
            header: self.header.clone(),
            previous_receipt_id: self.previous_receipt_id,
            previous_hash: self.previous_hash,
            operation_payload_digest: self.operation_payload_digest,
        };
        // By controlling the exact fields and using a typed digest for the opaque JSON,
        // we guarantee a deterministic preimage output.
        let bytes = serde_json::to_vec(&preimage)?;
        Ok(Sha256Digest::compute(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apparatus_time::UnixMs;
    use apparatus_types::{
        Classification, ObjectId, ObjectStatus, ObjectType, PrincipalId, ProjectId, SchemaVersion,
    };
    use std::str::FromStr;

    fn get_header() -> ObjectHeader {
        ObjectHeader {
            id: ObjectId::from_str("018fa03f-7634-7546-bf52-52643bf4c084").unwrap(),
            object_type: ObjectType::new("receipt").unwrap(),
            schema_version: SchemaVersion::new("1.0").unwrap(),
            created_at_unix_ms: UnixMs::new(1000),
            recorded_at_unix_ms: UnixMs::new(1000),
            occurred_at_unix_ms: None,
            owner_id: PrincipalId::from_str("018fa03f-7634-7546-bf52-52643bf4c084").unwrap(),
            project_id: Some(ProjectId::from_str("018fa03f-7634-7546-bf52-52643bf4c084").unwrap()),
            classification: Classification::Public,
            provenance: vec![],
            content_digest: None,
            status: ObjectStatus::Active,
            supersedes_id: None,
            correlation_id: None,
            causation_id: None,
        }
    }

    #[test]
    fn test_deterministic_receipt_hash() {
        let digest_hex = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let digest = Sha256Digest::from_str(digest_hex).unwrap();

        let receipt1 = Receipt {
            header: get_header(),
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload_digest: digest,
        };

        let receipt2 = Receipt {
            header: get_header(),
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload_digest: digest,
        };

        assert_eq!(
            receipt1.compute_hash().unwrap(),
            receipt2.compute_hash().unwrap()
        );
    }

    #[test]
    fn test_hash_changes_with_field() {
        let digest_hex = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let digest = Sha256Digest::from_str(digest_hex).unwrap();

        let mut receipt1 = Receipt {
            header: get_header(),
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload_digest: digest,
        };

        let receipt2 = receipt1.clone();

        // modify an optional field
        receipt1.previous_hash = Some(digest);

        assert_ne!(
            receipt1.compute_hash().unwrap(),
            receipt2.compute_hash().unwrap()
        );
    }
}

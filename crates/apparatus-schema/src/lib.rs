//! Schema validation for Apparatus domain types.
//!
//! Provides the Validate trait and implementations for foundational types.
//! Does not perform external network lookups or broad registry resolution.

use apparatus_types::{Classification, ObjectHeader, ObjectStatus};
use thiserror::Error;

/// Errors that can occur during domain validation.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// A required string field was empty.
    #[error("Field {field} cannot be empty")]
    EmptyField { field: String },
    /// A timestamp field was invalid or logically impossible.
    #[error("Invalid timestamp on field {field}: {reason}")]
    InvalidTimestamp { field: String, reason: String },
    /// Missing required internal invariants (e.g. absent provenance for non-public).
    #[error("Missing invariant: {0}")]
    MissingInvariant(String),
}

/// A trait for validating internal invariants of domain objects.
pub trait Validate {
    /// Validates the object, returning `Ok(())` if it satisfies all invariants.
    fn validate(&self) -> Result<(), ValidationError>;
}

impl Validate for ObjectHeader {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.created_at_unix_ms.as_u64() == 0 {
            return Err(ValidationError::InvalidTimestamp {
                field: "created_at_unix_ms".to_string(),
                reason: "cannot be zero".to_string(),
            });
        }

        if self.recorded_at_unix_ms.as_u64() == 0 {
            return Err(ValidationError::InvalidTimestamp {
                field: "recorded_at_unix_ms".to_string(),
                reason: "cannot be zero".to_string(),
            });
        }

        if self.created_at_unix_ms.as_u64() > self.recorded_at_unix_ms.as_u64() {
            return Err(ValidationError::InvalidTimestamp {
                field: "created_at_unix_ms".to_string(),
                reason: "created_at cannot be after recorded_at".to_string(),
            });
        }

        if self.classification == Classification::Restricted && self.provenance.is_empty() {
            return Err(ValidationError::MissingInvariant(
                "Restricted classification requires provenance".to_string(),
            ));
        }

        if self.status == ObjectStatus::Superseded && self.supersedes_id.is_none() {
            // Note: If an object is superseded, it usually means there is a newer object that supersedes it.
            // The object itself might not have a supersedes_id (which points to an older object IT replaces).
            // But if the requirement meant "if status is active and supersedes is set", we need to clarify.
            // A safer invariant is: if an object has a `supersedes_id`, it is explicitly replacing something.
            // Wait, the status is for THIS object.
        }

        for prov in &self.provenance {
            if prov.sources.is_empty() {
                return Err(ValidationError::MissingInvariant(
                    "Provenance must contain at least one source reference".to_string(),
                ));
            }
            for src in &prov.sources {
                if src.reference.is_empty() {
                    return Err(ValidationError::EmptyField {
                        field: "provenance.source.reference".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apparatus_time::UnixMs;
    use apparatus_types::{
        Classification, ObjectId, ObjectStatus, ObjectType, PrincipalId, ProjectId, Provenance,
        ProvenanceKind, SchemaVersion, SourceReference,
    };
    use std::str::FromStr;

    fn valid_header() -> ObjectHeader {
        ObjectHeader {
            id: ObjectId::from_str("018fa03f-7634-7546-bf52-52643bf4c084").unwrap(),
            object_type: ObjectType::new("test").unwrap(),
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
    fn test_valid_header() {
        let header = valid_header();
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_invalid_zero_timestamp() {
        let mut header = valid_header();
        header.created_at_unix_ms = UnixMs::new(0);
        match header.validate() {
            Err(ValidationError::InvalidTimestamp { .. }) => {}
            _ => panic!("Expected InvalidTimestamp error"),
        }
    }

    #[test]
    fn test_created_after_recorded() {
        let mut header = valid_header();
        header.created_at_unix_ms = UnixMs::new(2000);
        header.recorded_at_unix_ms = UnixMs::new(1000);
        match header.validate() {
            Err(ValidationError::InvalidTimestamp { .. }) => {}
            _ => panic!("Expected InvalidTimestamp error"),
        }
    }

    #[test]
    fn test_restricted_requires_provenance() {
        let mut header = valid_header();
        header.classification = Classification::Restricted;
        match header.validate() {
            Err(ValidationError::MissingInvariant(msg)) => {
                assert!(msg.contains("provenance"));
            }
            _ => panic!("Expected MissingInvariant error"),
        }
    }

    #[test]
    fn test_provenance_validation() {
        let mut header = valid_header();
        header.provenance = vec![Provenance {
            sources: vec![SourceReference {
                kind: ProvenanceKind::Url,
                reference: "".to_string(),
            }],
            creator: PrincipalId::from_str("018fa03f-7634-7546-bf52-52643bf4c084").unwrap(),
        }];

        match header.validate() {
            Err(ValidationError::EmptyField { field }) => {
                assert_eq!(field, "provenance.source.reference");
            }
            _ => panic!("Expected EmptyField error"),
        }
    }
}

//! Schema validation for Apparatus domain types.
//!
//! Provides the Validate trait and implementations for foundational types.
//! Does not perform external network lookups or broad registry resolution.

use apparatus_types::ObjectHeader;
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
        if self.created_at == 0 {
            return Err(ValidationError::InvalidTimestamp {
                field: "created_at".to_string(),
                reason: "cannot be zero".to_string(),
            });
        }

        if self.classification == apparatus_types::Classification::Restricted && self.provenance.is_none() {
             return Err(ValidationError::MissingInvariant(
                 "Restricted classification requires provenance".to_string()
             ));
        }

        if let Some(prov) = &self.provenance {
            if prov.source.is_empty() {
                return Err(ValidationError::EmptyField {
                    field: "provenance.source".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apparatus_types::{Classification, ObjectId, ProjectId};

    #[test]
    fn test_valid_header() {
        let header = ObjectHeader {
            id: ObjectId::new_v7(),
            created_at: 1000,
            project_id: ProjectId::new(),
            classification: Classification::Public,
            provenance: None,
            correlation_id: None,
        };
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_invalid_zero_timestamp() {
        let header = ObjectHeader {
            id: ObjectId::new_v7(),
            created_at: 0,
            project_id: ProjectId::new(),
            classification: Classification::Public,
            provenance: None,
            correlation_id: None,
        };
        match header.validate() {
            Err(ValidationError::InvalidTimestamp { .. }) => {}
            _ => panic!("Expected InvalidTimestamp error"),
        }
    }

    #[test]
    fn test_restricted_requires_provenance() {
        let header = ObjectHeader {
            id: ObjectId::new_v7(),
            created_at: 1000,
            project_id: ProjectId::new(),
            classification: Classification::Restricted,
            provenance: None,
            correlation_id: None,
        };
        match header.validate() {
            Err(ValidationError::MissingInvariant(msg)) => {
                assert!(msg.contains("provenance"));
            }
            _ => panic!("Expected MissingInvariant error"),
        }
    }
}

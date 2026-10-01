//! Core types for the Apparatus foundation.
//!
//! This crate contains identity and basic object schema primitives.
//! It does not perform persistence or handle network requests.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

pub mod quarantine;
pub mod rws;

/// A generalized Object ID, currently implemented as UUIDv7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectId(Uuid);

impl ObjectId {
    /// Create a new UUIDv7-based ID.
    pub fn new_v7() -> Self {
        Self(Uuid::now_v7())
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Error returned when parsing an id from text fails.
#[derive(Debug, thiserror::Error)]
#[error("invalid object id: {0}")]
pub struct ParseIdError(String);

impl FromStr for ObjectId {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| ParseIdError(s.to_string()))
    }
}

macro_rules! id_type {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(ObjectId);

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            /// Create a new id.
            pub fn new() -> Self {
                Self(ObjectId::new_v7())
            }

            /// Get the underlying ObjectId.
            pub fn inner(&self) -> ObjectId {
                self.0
            }
        }

        impl From<ObjectId> for $name {
            fn from(id: ObjectId) -> Self {
                Self(id)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl FromStr for $name {
            type Err = ParseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                ObjectId::from_str(s).map(Self)
            }
        }
    };
}

id_type!(ProjectId, "Unique identifier for a Project context.");
id_type!(
    ArtifactId,
    "Unique identifier for an Artifact (often used to index a descriptor or object)."
);
id_type!(ReceiptId, "Unique identifier for a Ledger Receipt.");
id_type!(
    PrincipalId,
    "Unique identifier for a Principal (user or service identity)."
);
id_type!(
    CorrelationId,
    "Unique identifier used to correlate causally related events."
);

/// Classification of an object's sensitivity or rights context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    /// Object is unrestricted.
    Public,
    /// Object requires specific authorization to read.
    Restricted,
    /// Internal / operational use.
    Internal,
}

/// Provenance metadata describing the source of an object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// Where the object came from (e.g., URL or system string).
    pub source: String,
    /// Who authorized or produced the object.
    pub creator: PrincipalId,
}

/// Standard envelope header for canonical Apparatus objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectHeader {
    /// Canonical identity of the object.
    pub id: ObjectId,
    /// System time when the object was created (Unix ms).
    pub created_at: u64,
    /// Containing project context.
    pub project_id: ProjectId,
    /// Sensitivity classification.
    pub classification: Classification,
    /// Lineage/source of the object.
    pub provenance: Option<Provenance>,
    /// Optional correlation id linking this to a larger operation.
    pub correlation_id: Option<CorrelationId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_roundtrips() {
        let obj_id = ObjectId::new_v7();
        let serialized = serde_json::to_string(&obj_id).unwrap();
        let deserialized: ObjectId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(obj_id, deserialized);
    }

    #[test]
    fn test_object_header_distinct_ids() {
        let header = ObjectHeader {
            id: ObjectId::new_v7(),
            created_at: 12345,
            project_id: ProjectId::new(),
            classification: Classification::Public,
            provenance: None,
            correlation_id: None,
        };
        assert_ne!(header.id, header.project_id.inner());
    }
}

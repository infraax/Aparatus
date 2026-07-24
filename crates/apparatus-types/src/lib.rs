//! Core types for the Apparatus foundation.
//!
//! This crate contains identity and basic object schema primitives.
//! It does not perform persistence or handle network requests.

use apparatus_time::UnixMs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

/// Error type for identity parsing/formatting.
#[derive(Debug, Error)]
pub enum IdError {
    /// Provided UUID string is invalid.
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(#[from] uuid::Error),
    /// Provided UUID is not version 7.
    #[error("Invalid UUID version, expected v7, got {0:?}")]
    InvalidVersion(Option<uuid::Version>),
}

/// A generalized Object ID, implemented as a strictly validated UUIDv7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectId(Uuid);

impl ObjectId {
    /// Create a new UUIDv7-based ID.
    pub fn new_v7() -> Self {
        Self(Uuid::now_v7())
    }

    /// Retrieve the underlying UUID.
    pub fn inner(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ObjectId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        if uuid.get_version() != Some(uuid::Version::SortRand) {
            // SortMac is the uuid crate's enum variant for version 7
            return Err(IdError::InvalidVersion(uuid.get_version()));
        }
        Ok(Self(uuid))
    }
}

impl TryFrom<Uuid> for ObjectId {
    type Error = IdError;

    fn try_from(uuid: Uuid) -> Result<Self, Self::Error> {
        if uuid.get_version() != Some(uuid::Version::SortRand) {
            return Err(IdError::InvalidVersion(uuid.get_version()));
        }
        Ok(Self(uuid))
    }
}

macro_rules! id_type {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(ObjectId);

        impl $name {
            /// Get the underlying ObjectId.
            pub fn inner(&self) -> ObjectId {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let obj_id = ObjectId::from_str(s)?;
                Ok(Self(obj_id))
            }
        }

        impl From<ObjectId> for $name {
            fn from(id: ObjectId) -> Self {
                Self(id)
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
id_type!(
    CausationId,
    "Unique identifier used to track causality between events."
);
id_type!(
    SupersedesId,
    "Unique identifier pointing to an object this object supersedes."
);

/// Classification of an object's sensitivity or rights context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    /// Object is unrestricted.
    Public,
    /// Object requires specific authorization to read.
    Restricted,
    /// Internal / operational use.
    Internal,
}

/// Lifecycle status for canonical objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectStatus {
    /// The object is currently active and authoritative.
    Active,
    /// The object has been replaced by a newer version (see `supersedes`).
    Superseded,
    /// The object is marked for future retirement.
    Deprecated,
    /// The object's authority has been officially revoked.
    Revoked,
    /// The object has been logically deleted (content may or may not be purged).
    LogicallyDeleted,
}

/// The kind of provenance reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceKind {
    /// Originates from a specific remote URL.
    Url,
    /// Originates from another internal subsystem.
    Subsystem,
    /// Manually authored/uploaded.
    Manual,
    /// An opaque string if the source kind is unknown but recorded.
    Unknown,
}

/// A strongly-typed reference to a source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReference {
    /// The kind of the source.
    pub kind: ProvenanceKind,
    /// The string representation of the source (e.g., a URL or system ID).
    pub reference: String,
}

/// Provenance metadata describing the source of an object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// List of source references. Cannot be empty if provenance is provided.
    pub sources: Vec<SourceReference>,
    /// Who authorized or produced the object.
    pub creator: PrincipalId,
}

/// A short typed content digest (SHA-256) avoiding cyclic dependency with apparatus-crypto.
/// This represents the lower-hex string of a SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentDigest(String);

impl ContentDigest {
    /// Construct from a 64-character lower-hex SHA-256 string.
    pub fn new(hex: String) -> Result<Self, &'static str> {
        if hex.len() != 64 {
            return Err("Invalid length for SHA-256 hex");
        }
        if !hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        {
            return Err("Invalid characters in SHA-256 hex (must be lowercase hex)");
        }
        Ok(Self(hex))
    }

    /// Get the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Defines the object's type in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectType(String);

impl ObjectType {
    /// Construct a new ObjectType.
    pub fn new(ty: impl Into<String>) -> Result<Self, &'static str> {
        let ty = ty.into();
        if ty.is_empty() {
            return Err("ObjectType cannot be empty");
        }
        Ok(Self(ty))
    }
}

/// Defines the schema version of the object's payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion(String);

impl SchemaVersion {
    /// Construct a new SchemaVersion.
    pub fn new(version: impl Into<String>) -> Result<Self, &'static str> {
        let version = version.into();
        if version.is_empty() {
            return Err("SchemaVersion cannot be empty");
        }
        Ok(Self(version))
    }
}

/// Standard envelope header for canonical Apparatus objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectHeader {
    /// Canonical identity of the object.
    pub id: ObjectId,
    /// The type of object.
    pub object_type: ObjectType,
    /// The schema version of the object's payload.
    pub schema_version: SchemaVersion,
    /// System time when the object was logically created (Unix ms).
    pub created_at_unix_ms: UnixMs,
    /// System time when the object was recorded in the system (Unix ms).
    pub recorded_at_unix_ms: UnixMs,
    /// Optional system time when the event originally occurred (Unix ms).
    pub occurred_at_unix_ms: Option<UnixMs>,
    /// Principal who owns/authored this object.
    pub owner_id: PrincipalId,
    /// Containing project context (optional if global/system scoped).
    pub project_id: Option<ProjectId>,
    /// Sensitivity classification.
    pub classification: Classification,
    /// Lineage/source of the object.
    pub provenance: Vec<Provenance>,
    /// Optional content SHA-256 digest.
    pub content_digest: Option<ContentDigest>,
    /// Lifecycle status.
    pub status: ObjectStatus,
    /// The object this one replaces.
    pub supersedes_id: Option<SupersedesId>,
    /// Optional correlation id linking this to a larger operation.
    pub correlation_id: Option<CorrelationId>,
    /// Optional causation id linking this to the direct cause.
    pub causation_id: Option<CausationId>,
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
    fn test_uuid_v7_validation() {
        // v4 UUID
        let v4_uuid = Uuid::nil();
        assert!(ObjectId::try_from(v4_uuid).is_err());
        assert!(ObjectId::from_str(&v4_uuid.to_string()).is_err());

        // v7 UUID
        let v7_uuid = Uuid::now_v7();
        assert!(ObjectId::try_from(v7_uuid).is_ok());
        assert!(ObjectId::from_str(&v7_uuid.to_string()).is_ok());
    }

    #[test]
    fn test_content_digest_validation() {
        let valid_hex =
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".to_string();
        assert!(ContentDigest::new(valid_hex).is_ok());

        let invalid_length = "b94d27b9".to_string();
        assert!(ContentDigest::new(invalid_length).is_err());

        let invalid_chars =
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcdeZ".to_string();
        assert!(ContentDigest::new(invalid_chars).is_err());

        let uppercase =
            "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9".to_string();
        assert!(ContentDigest::new(uppercase).is_err());
    }
}

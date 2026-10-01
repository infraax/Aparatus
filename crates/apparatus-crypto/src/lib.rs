//! Cryptographic primitives for Apparatus.
//!
//! Provides explicit hashing mechanisms, notably SHA-256 for artifacts and the
//! chain, plus the envelope layer's BLAKE3/CID v1 addresses and signing keys.

use sha2::{Digest, Sha256};
use std::fmt;

pub mod canonical;
pub mod cid;
pub mod signing;

pub use canonical::{canonical_json, CanonicalError};
pub use cid::{Blake3Digest, Cid, CidCodec};
pub use signing::{Ed25519Key, MlDsa65Stub, SigningKey};

/// An error that occurs during cryptographic operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// Provided hex digest string has an invalid length.
    #[error("invalid digest length, expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    /// Provided hex digest string contains invalid characters.
    #[error("invalid hex character in digest")]
    InvalidHex,
}

/// A 32-byte SHA-256 digest. Serialises as a 64-character lowercase hex string.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256Digest([u8; 32]);

#[cfg(feature = "serde")]
impl serde::Serialize for Sha256Digest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:x}", self))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Sha256Digest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <String as serde::Deserialize>::deserialize(deserializer)?;
        Sha256Digest::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl Sha256Digest {
    /// Compute the SHA-256 digest of the given bytes.
    pub fn compute(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut array = [0u8; 32];
        array.copy_from_slice(&result);
        Self(array)
    }

    /// Parse a SHA-256 digest from a 64-character lowercase hex string.
    pub fn from_hex(s: &str) -> Result<Self, CryptoError> {
        if s.len() != 64 {
            return Err(CryptoError::InvalidLength {
                expected: 64,
                actual: s.len(),
            });
        }
        // Reject non-ASCII (slicing would panic), signs and uppercase before parsing.
        if !s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            return Err(CryptoError::InvalidHex);
        }
        let mut array = [0u8; 32];
        for i in 0..32 {
            let byte_str = &s[i * 2..i * 2 + 2];
            array[i] = u8::from_str_radix(byte_str, 16).map_err(|_| CryptoError::InvalidHex)?;
        }
        Ok(Self(array))
    }

    /// Return the raw byte array of the digest.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::LowerHex for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sha256Digest({:x})", self)
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_known_answer() {
        // Echo -n "hello world" | sha256sum
        let data = b"hello world";
        let digest = Sha256Digest::compute(data);
        let hex = format!("{:x}", digest);
        assert_eq!(
            hex,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sha256_from_hex() {
        let hex = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let digest = Sha256Digest::from_hex(hex).unwrap();
        let formatted = format!("{:x}", digest);
        assert_eq!(hex, formatted);
    }

    #[test]
    fn test_sha256_invalid_hex() {
        assert!(Sha256Digest::from_hex("invalid_length").is_err());
        assert!(Sha256Digest::from_hex(
            "z94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        )
        .is_err());
    }
}

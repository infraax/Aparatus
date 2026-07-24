//! Cryptographic primitives for Apparatus.
//!
//! Provides explicit hashing mechanisms, notably SHA-256 for artifacts.

use sha2::{Digest, Sha256};
use std::fmt;
use std::str::FromStr;

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

/// A 32-byte SHA-256 digest.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256Digest([u8; 32]);

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

        if !s
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        {
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

impl FromStr for Sha256Digest {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
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
        let digest = Sha256Digest::from_str(hex).unwrap();
        let formatted = format!("{:x}", digest);
        assert_eq!(hex, formatted);
    }

    #[test]
    fn test_sha256_invalid_hex() {
        assert!(Sha256Digest::from_str("invalid_length").is_err());
        assert!(Sha256Digest::from_str(
            "z94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        )
        .is_err());
        // Reject uppercase
        assert!(Sha256Digest::from_str(
            "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9"
        )
        .is_err());
    }
}

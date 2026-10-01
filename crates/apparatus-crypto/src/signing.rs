//! Signing keys for envelopes (NAP-corpus #12, Stage 0).
//!
//! `SigningKey` is the seam later stages plug into. Stage 0 ships:
//! - `Ed25519Key`: the real path (RFC 8032, deterministic).
//! - `MlDsa65Stub`: a placeholder whose verification always succeeds. It signs
//!   under its own scheme name, `ml_dsa_65_stub`, so receipts it produced stay
//!   verifiable as what they are when Stage 2 (#14) adds a real `ml_dsa_65`
//!   scheme next to it. Nothing signed by the stub is authenticated.

use ed25519_dalek::Signer;
use sha2::{Digest, Sha256};

use crate::cid::{base32_lower, Blake3Digest};

/// Wire name of the Ed25519 scheme.
pub const SCHEME_ED25519: &str = "ed25519";
/// Wire name of the ML-DSA-65 placeholder. Never reuse it for a real implementation.
pub const SCHEME_ML_DSA_65_STUB: &str = "ml_dsa_65_stub";

/// A key that signs envelope bytes.
pub trait SigningKey {
    /// Wire name of the scheme, bound into the signed bytes and the key id.
    fn scheme(&self) -> &'static str;
    /// Public key bytes as carried on the receipt.
    fn public_key(&self) -> Vec<u8>;
    /// Sign `msg`.
    fn sign(&self, msg: &[u8]) -> Vec<u8>;
    /// Verify `sig` over `msg` against this key's public half.
    fn verify(&self, msg: &[u8], sig: &[u8]) -> bool;
}

/// Ed25519 (RFC 8032) — the real Stage 0 path.
pub struct Ed25519Key {
    inner: ed25519_dalek::SigningKey,
}

impl Ed25519Key {
    /// Build from a 32-byte secret seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            inner: ed25519_dalek::SigningKey::from_bytes(seed),
        }
    }

    /// Fresh key from the OS random source.
    pub fn generate() -> Result<Self, getrandom::Error> {
        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed)?;
        Ok(Self::from_seed(&seed))
    }

    /// The secret seed, for persistence by the caller.
    pub fn seed(&self) -> [u8; 32] {
        self.inner.to_bytes()
    }
}

impl SigningKey for Ed25519Key {
    fn scheme(&self) -> &'static str {
        SCHEME_ED25519
    }

    fn public_key(&self) -> Vec<u8> {
        self.inner.verifying_key().to_bytes().to_vec()
    }

    fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.inner.sign(msg).to_bytes().to_vec()
    }

    fn verify(&self, msg: &[u8], sig: &[u8]) -> bool {
        ed25519_verify(&self.public_key(), msg, sig)
    }
}

/// Verify an Ed25519 signature. Malformed keys or signatures verify as false.
pub fn ed25519_verify(public_key: &[u8], msg: &[u8], sig: &[u8]) -> bool {
    let Ok(pk) = <[u8; 32]>::try_from(public_key) else {
        return false;
    };
    let Ok(sig) = <[u8; 64]>::try_from(sig) else {
        return false;
    };
    let Ok(vk) = ed25519_dalek::VerifyingKey::from_bytes(&pk) else {
        return false;
    };
    vk.verify_strict(msg, &ed25519_dalek::Signature::from_bytes(&sig))
        .is_ok()
}

/// ML-DSA-65 placeholder (FIPS 204 is Stage 2, NAP-corpus #14).
///
/// `verify` returns `true` for any input, by design. The "signature" it emits is
/// BLAKE3(public_key || msg): it binds nothing and must not be trusted.
pub struct MlDsa65Stub {
    public_key: Vec<u8>,
}

impl MlDsa65Stub {
    /// A stub key identified by arbitrary public bytes (e.g. the node's Ed25519 key).
    pub fn new(public_key: Vec<u8>) -> Self {
        Self { public_key }
    }
}

impl SigningKey for MlDsa65Stub {
    fn scheme(&self) -> &'static str {
        SCHEME_ML_DSA_65_STUB
    }

    fn public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let mut bytes = self.public_key.clone();
        bytes.extend_from_slice(msg);
        Blake3Digest::compute(&bytes).as_bytes().to_vec()
    }

    fn verify(&self, msg: &[u8], sig: &[u8]) -> bool {
        ml_dsa_65_stub_verify(&self.public_key, msg, sig)
    }
}

/// Stub verification: always `true` (Stage 0 acceptance criterion).
pub fn ml_dsa_65_stub_verify(_public_key: &[u8], _msg: &[u8], _sig: &[u8]) -> bool {
    true
}

/// Key id bound to its scheme (NAP T6 D.3 construction, T1 B.5 key commitment):
/// `"ARCHIVE-KID-" || base32_nopad(SHA-256(scheme || 0x00 || public_key)[0..10])`.
pub fn key_id(scheme: &str, public_key: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(scheme.as_bytes());
    h.update([0u8]);
    h.update(public_key);
    let full = h.finalize();
    format!("ARCHIVE-KID-{}", base32_lower(&full[..10]).to_uppercase())
}

/// Domain-separated message: `tag || u32_be(len(body)) || body` (NAP T6 D.3 framing).
pub fn frame_message(tag: &[u8], body: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(tag.len() + 4 + body.len());
    m.extend_from_slice(tag);
    m.extend_from_slice(&(body.len() as u32).to_be_bytes());
    m.extend_from_slice(body);
    m
}

/// Lowercase hex encoding.
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decode lowercase hex; `None` on odd length or a non-hex character.
pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ed25519_rfc8032_test_vector_1() {
        // RFC 8032 §7.1, TEST 1 (empty message).
        let seed: [u8; 32] =
            from_hex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .unwrap()
                .try_into()
                .unwrap();
        let key = Ed25519Key::from_seed(&seed);
        assert_eq!(
            to_hex(&key.public_key()),
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
        );
        let sig = key.sign(b"");
        assert_eq!(
            to_hex(&sig),
            "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155\
             5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
        );
        assert!(key.verify(b"", &sig));
    }

    #[test]
    fn ed25519_rejects_tampering() {
        let key = Ed25519Key::generate().unwrap();
        let sig = key.sign(b"payload");
        assert!(key.verify(b"payload", &sig));
        assert!(!key.verify(b"payload!", &sig));
        let mut bad = sig.clone();
        bad[0] ^= 1;
        assert!(!key.verify(b"payload", &bad));
        assert!(!ed25519_verify(&key.public_key(), b"payload", &sig[..63]));
        let other = Ed25519Key::generate().unwrap();
        assert!(!ed25519_verify(&other.public_key(), b"payload", &sig));
        // Round-trip through the stored seed.
        let again = Ed25519Key::from_seed(&key.seed());
        assert_eq!(again.public_key(), key.public_key());
    }

    #[test]
    fn ml_dsa_65_stub_verify_is_always_true() {
        let stub = MlDsa65Stub::new(vec![7; 32]);
        assert_eq!(stub.scheme(), "ml_dsa_65_stub");
        let sig = stub.sign(b"anything");
        assert!(stub.verify(b"anything", &sig));
        // Deliberately unauthenticated: wrong message and garbage both pass.
        assert!(stub.verify(b"something else", b"garbage"));
        assert!(ml_dsa_65_stub_verify(&[], &[], &[]));
    }

    #[test]
    fn key_id_binds_the_scheme() {
        let pk = [1u8; 32];
        let a = key_id(SCHEME_ED25519, &pk);
        let b = key_id(SCHEME_ML_DSA_65_STUB, &pk);
        assert_ne!(a, b);
        assert!(a.starts_with("ARCHIVE-KID-"));
        assert_eq!(a.len(), "ARCHIVE-KID-".len() + 16);
    }

    #[test]
    fn frame_and_hex() {
        assert_eq!(frame_message(b"T\0", b"ab"), b"T\0\0\0\0\x02ab".to_vec());
        assert_eq!(from_hex(&to_hex(&[0, 255, 16])).unwrap(), vec![0, 255, 16]);
        assert!(from_hex("abc").is_none());
        assert!(from_hex("AB").is_none());
    }
}

//! BLAKE3-256 digests and CID v1 content addresses (NAP T6 B.1).
//!
//! `CID( v1, codec, multihash(0x1e, 0x20, BLAKE3(bytes)) )`, string form
//! `"b" || base32_lower_nopad(binary_cid)`. Stage 0 computes CIDs next to the
//! SHA-256 chain hash; it does not deduplicate on them.

use std::fmt;

/// Multihash code for BLAKE3-256.
pub const MULTIHASH_BLAKE3: u8 = 0x1e;
/// CID version byte.
pub const CID_V1: u8 = 0x01;

/// A 32-byte BLAKE3 digest. Displays as 64 lowercase hex characters.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Blake3Digest([u8; 32]);

impl Blake3Digest {
    /// Compute the BLAKE3-256 digest of the given bytes.
    pub fn compute(data: &[u8]) -> Self {
        Self(*blake3::hash(data).as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::LowerHex for Blake3Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Blake3Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blake3Digest({:x})", self)
    }
}

/// Multicodec content type of the addressed bytes. Both codes fit in one varint byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CidCodec {
    /// 0x55: opaque bytes (used for canonical JSON payloads in Stage 0).
    Raw,
    /// 0x71: deterministic CBOR.
    DagCbor,
}

impl CidCodec {
    pub fn code(&self) -> u8 {
        match self {
            CidCodec::Raw => 0x55,
            CidCodec::DagCbor => 0x71,
        }
    }
}

/// A CID v1 with a BLAKE3-256 multihash.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cid {
    codec: CidCodec,
    digest: Blake3Digest,
}

impl Cid {
    /// Address `data` under `codec`.
    pub fn compute(codec: CidCodec, data: &[u8]) -> Self {
        Self {
            codec,
            digest: Blake3Digest::compute(data),
        }
    }

    pub fn codec(&self) -> CidCodec {
        self.codec
    }

    pub fn digest(&self) -> &Blake3Digest {
        &self.digest
    }

    /// Binary form: version, codec, multihash code, digest length, digest (36 bytes).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(36);
        out.extend_from_slice(&[CID_V1, self.codec.code(), MULTIHASH_BLAKE3, 0x20]);
        out.extend_from_slice(self.digest.as_bytes());
        out
    }
}

impl fmt::Display for Cid {
    /// Multibase base32 lowercase, prefix `b`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b{}", base32_lower(&self.to_bytes()))
    }
}

impl fmt::Debug for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cid({self})")
    }
}

/// RFC 4648 base32, lowercase alphabet, no padding.
pub fn base32_lower(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut out = String::with_capacity(bytes.len().div_ceil(5) * 8);
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for &b in bytes {
        buffer = (buffer << 8) | u32::from(b);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ALPHABET[((buffer >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((buffer << (5 - bits)) & 31) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_known_answer() {
        // Official BLAKE3 test vector for the empty input.
        assert_eq!(
            format!("{:x}", Blake3Digest::compute(b"")),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn base32_matches_rfc4648_vectors() {
        // RFC 4648 §10, lowercased, padding stripped.
        for (input, expected) in [
            ("", ""),
            ("f", "my"),
            ("fo", "mzxq"),
            ("foo", "mzxw6"),
            ("foob", "mzxw6yq"),
            ("fooba", "mzxw6ytb"),
            ("foobar", "mzxw6ytboi"),
        ] {
            assert_eq!(base32_lower(input.as_bytes()), expected);
        }
    }

    #[test]
    fn cid_v1_raw_blake3_layout() {
        let cid = Cid::compute(CidCodec::Raw, b"");
        let bytes = cid.to_bytes();
        assert_eq!(&bytes[..4], &[0x01, 0x55, 0x1e, 0x20]);
        assert_eq!(bytes.len(), 36);
        let s = cid.to_string();
        // 36 bytes → 58 base32 chars, plus the multibase prefix.
        assert_eq!(s.len(), 59);
        // Every CIDv1/raw/blake3 string starts with the same multibase+header prefix.
        assert!(s.starts_with("bafkr4i"), "{s}");
        assert_eq!(
            s,
            // Computed independently: Python base64.b32encode over 01 55 1e 20 || BLAKE3("").
            "bafkr4ifpcne3t5pzugtkaqcn5i3nzskjtpfslsnnyejlpte2spfoihzsmi"
        );
        assert_ne!(
            Cid::compute(CidCodec::DagCbor, b"").to_string(),
            s,
            "codec is part of the address"
        );
    }
}

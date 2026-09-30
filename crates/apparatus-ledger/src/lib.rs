//! Append-only receipt ledger for Apparatus.
//!
//! Provides the receipt domain type, canonical JSON hashing (RWS-2.0 X-04),
//! and `FileLedger`: an append-only JSONL chain with a HEAD pointer.

use apparatus_crypto::Sha256Digest;
use apparatus_store::{ReceiptLedger, StoreError};
use apparatus_types::{ObjectHeader, ReceiptId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Ledger-related errors.
#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// The value cannot be encoded canonically (e.g. it contains a float).
    #[error("Non-canonical value: {0}")]
    NonCanonical(String),
    #[error("Store error: {0}")]
    Store(#[from] StoreError),
    /// The on-disk chain is inconsistent.
    #[error("Integrity failure at line {line}: {reason}")]
    Integrity { line: usize, reason: String },
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
    /// JSON describing the mutation applied (RWS 2.0 payload).
    pub operation_payload: serde_json::Value,
}

impl Receipt {
    /// The receipt's id as a `ReceiptId`.
    pub fn receipt_id(&self) -> ReceiptId {
        ReceiptId::from(self.header.id)
    }

    /// Canonical JSON bytes of the whole receipt: sorted keys, no whitespace,
    /// integers only. This is the exact line written to the ledger file.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, LedgerError> {
        canonical_json(&serde_json::to_value(self)?)
    }

    /// Deterministically hash the receipt over its canonical bytes.
    pub fn compute_hash(&self) -> Result<Sha256Digest, LedgerError> {
        Ok(Sha256Digest::compute(&self.canonical_bytes()?))
    }
}

/// Encode a JSON value canonically: object keys sorted by byte order, no
/// insignificant whitespace, and no floating-point numbers.
pub fn canonical_json(value: &Value) -> Result<Vec<u8>, LedgerError> {
    let mut out = Vec::new();
    write_canonical(value, &mut out)?;
    Ok(out)
}

fn write_canonical(value: &Value, out: &mut Vec<u8>) -> Result<(), LedgerError> {
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => {
            out.extend_from_slice(serde_json::to_string(value)?.as_bytes());
        }
        Value::Number(n) => {
            if n.is_f64() {
                return Err(LedgerError::NonCanonical(format!(
                    "floating-point number {n} not allowed; use integer units"
                )));
            }
            out.extend_from_slice(n.to_string().as_bytes());
        }
        Value::Array(items) => {
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push(b'{');
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                out.extend_from_slice(serde_json::to_string(key)?.as_bytes());
                out.push(b':');
                write_canonical(&map[key], out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

/// A receipt read back from the ledger, with its verified hash.
#[derive(Debug, Clone)]
pub struct ChainEntry {
    pub receipt: Receipt,
    pub hash: Sha256Digest,
}

/// Append-only ledger stored as `ledger.jsonl` plus a `HEAD` file in one directory.
///
/// `HEAD` holds `<receipt-id> <hash-hex>` of the last appended receipt.
#[derive(Debug, Clone)]
pub struct FileLedger {
    dir: PathBuf,
}

impl FileLedger {
    /// Use `dir` (normally `<project>/.apparatus`) as the ledger location.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn ledger_path(&self) -> PathBuf {
        self.dir.join("ledger.jsonl")
    }

    fn head_path(&self) -> PathBuf {
        self.dir.join("HEAD")
    }

    /// The current head, or `None` for an empty chain.
    pub fn head(&self) -> Result<Option<(ReceiptId, Sha256Digest)>, StoreError> {
        let path = self.head_path();
        if !path.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(path)?;
        let mut parts = text.split_whitespace();
        let (Some(id), Some(hash), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(StoreError::Backend("malformed HEAD".into()));
        };
        let id: ReceiptId = id
            .parse()
            .map_err(|_| StoreError::Backend("malformed HEAD id".into()))?;
        let hash = Sha256Digest::from_hex(hash).map_err(|e| StoreError::Backend(e.to_string()))?;
        Ok(Some((id, hash)))
    }

    /// Append a receipt. Fails unless it extends the current head exactly.
    pub fn append_receipt(&self, receipt: &Receipt) -> Result<Sha256Digest, LedgerError> {
        let head = self.head()?;
        let expected = head;
        let actual = match (receipt.previous_receipt_id, receipt.previous_hash) {
            (Some(id), Some(hash)) => Some((id, hash)),
            (None, None) => None,
            _ => {
                return Err(StoreError::ChainMismatch {
                    expected: describe(expected),
                    actual: "partial previous link".into(),
                }
                .into())
            }
        };
        if expected != actual {
            return Err(StoreError::ChainMismatch {
                expected: describe(expected),
                actual: describe(actual),
            }
            .into());
        }
        let bytes = receipt.canonical_bytes()?;
        let hash = Sha256Digest::compute(&bytes);

        fs::create_dir_all(&self.dir).map_err(StoreError::from)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.ledger_path())
            .map_err(StoreError::from)?;
        file.write_all(&bytes).map_err(StoreError::from)?;
        file.write_all(b"\n").map_err(StoreError::from)?;
        file.sync_all().map_err(StoreError::from)?;

        write_atomic(
            &self.head_path(),
            format!("{} {:x}\n", receipt.receipt_id(), hash).as_bytes(),
        )?;
        Ok(hash)
    }

    /// Read and verify the whole chain: every line is canonical, every link
    /// matches the previous hash, and the last entry matches `HEAD`.
    pub fn verify_chain(&self) -> Result<Vec<ChainEntry>, LedgerError> {
        let path = self.ledger_path();
        let text = if path.exists() {
            fs::read_to_string(&path).map_err(StoreError::from)?
        } else {
            String::new()
        };
        let mut entries: Vec<ChainEntry> = Vec::new();
        for (idx, line) in text.lines().enumerate() {
            let line_no = idx + 1;
            let receipt: Receipt =
                serde_json::from_str(line).map_err(|e| LedgerError::Integrity {
                    line: line_no,
                    reason: format!("unparseable receipt: {e}"),
                })?;
            let bytes = receipt.canonical_bytes()?;
            if bytes != line.as_bytes() {
                return Err(LedgerError::Integrity {
                    line: line_no,
                    reason: "line is not in canonical form".into(),
                });
            }
            let prev = entries.last().map(|e| (e.receipt.receipt_id(), e.hash));
            let link = match (receipt.previous_receipt_id, receipt.previous_hash) {
                (Some(id), Some(hash)) => Some((id, hash)),
                (None, None) => None,
                _ => {
                    return Err(LedgerError::Integrity {
                        line: line_no,
                        reason: "partial previous link".into(),
                    })
                }
            };
            if link != prev {
                return Err(LedgerError::Integrity {
                    line: line_no,
                    reason: format!(
                        "previous link {} does not match {}",
                        describe(link),
                        describe(prev)
                    ),
                });
            }
            entries.push(ChainEntry {
                hash: Sha256Digest::compute(&bytes),
                receipt,
            });
        }
        let last = entries.last().map(|e| (e.receipt.receipt_id(), e.hash));
        if last != self.head()? {
            return Err(LedgerError::Integrity {
                line: entries.len(),
                reason: "HEAD does not match last ledger entry".into(),
            });
        }
        Ok(entries)
    }
}

fn describe(link: Option<(ReceiptId, Sha256Digest)>) -> String {
    match link {
        Some((id, hash)) => format!("{id}@{hash:x}"),
        None => "<empty chain>".into(),
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let tmp = path.with_extension("tmp");
    let mut file = fs::File::create(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&tmp, path)?;
    Ok(())
}

impl ReceiptLedger for FileLedger {
    fn append(&self, id: ReceiptId, receipt_bytes: &[u8]) -> Result<(), StoreError> {
        let receipt: Receipt = serde_json::from_slice(receipt_bytes)
            .map_err(|e| StoreError::Backend(format!("unparseable receipt: {e}")))?;
        if receipt.receipt_id() != id {
            return Err(StoreError::Backend(
                "receipt id does not match header".into(),
            ));
        }
        self.append_receipt(&receipt)
            .map(|_| ())
            .map_err(|e| match e {
                LedgerError::Store(s) => s,
                other => StoreError::Backend(other.to_string()),
            })
    }

    fn get(&self, id: ReceiptId) -> Result<Vec<u8>, StoreError> {
        let entries = self
            .verify_chain()
            .map_err(|e| StoreError::Backend(e.to_string()))?;
        entries
            .into_iter()
            .find(|e| e.receipt.receipt_id() == id)
            .map(|e| {
                e.receipt
                    .canonical_bytes()
                    .map_err(|err| StoreError::Backend(err.to_string()))
            })
            .unwrap_or_else(|| Err(StoreError::ReceiptNotFound(id.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apparatus_types::{Classification, ObjectId, ProjectId};

    fn header() -> ObjectHeader {
        ObjectHeader {
            id: ObjectId::new_v7(),
            created_at: 1000,
            project_id: ProjectId::new(),
            classification: Classification::Public,
            provenance: None,
            correlation_id: None,
        }
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("apparatus-ledger-{tag}-{}", ObjectId::new_v7()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_deterministic_receipt_hash() {
        let header = header();

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

    #[test]
    fn canonical_hash_ignores_key_order() {
        let header = header();
        let a: Value =
            serde_json::from_str(r#"{"kind":"event","b":{"y":2,"x":1},"a":[3,{"q":1,"p":0}]}"#)
                .unwrap();
        let b: Value =
            serde_json::from_str(r#"{"a":[3,{"p":0,"q":1}],"b":{"x":1,"y":2},"kind":"event"}"#)
                .unwrap();
        let r1 = Receipt {
            header: header.clone(),
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload: a,
        };
        let r2 = Receipt {
            header,
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload: b,
        };
        assert_eq!(r1.compute_hash().unwrap(), r2.compute_hash().unwrap());
        assert_eq!(
            canonical_json(&r1.operation_payload).unwrap(),
            br#"{"a":[3,{"p":0,"q":1}],"b":{"x":1,"y":2},"kind":"event"}"#.to_vec()
        );
    }

    #[test]
    fn canonical_json_rejects_floats() {
        let v = serde_json::json!({ "amount": 1.5 });
        assert!(matches!(
            canonical_json(&v),
            Err(LedgerError::NonCanonical(_))
        ));
        assert!(canonical_json(&serde_json::json!({ "amount": 15 })).is_ok());
    }

    #[test]
    fn file_ledger_chains_and_refuses_wrong_previous_hash() {
        let dir = temp_dir("chain");
        let ledger = FileLedger::new(&dir);
        assert!(ledger.head().unwrap().is_none());

        let first = Receipt {
            header: header(),
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload: serde_json::json!({ "kind": "event" }),
        };
        let h1 = ledger.append_receipt(&first).unwrap();

        // A second receipt that claims to start a new chain is refused.
        let orphan = Receipt {
            header: header(),
            ..first.clone()
        };
        assert!(matches!(
            ledger.append_receipt(&orphan),
            Err(LedgerError::Store(StoreError::ChainMismatch { .. }))
        ));

        // A receipt pointing at the right id but the wrong hash is refused.
        let wrong = Receipt {
            header: header(),
            previous_receipt_id: Some(first.receipt_id()),
            previous_hash: Some(Sha256Digest::compute(b"not the head")),
            operation_payload: serde_json::json!({ "kind": "event" }),
        };
        assert!(ledger.append_receipt(&wrong).is_err());

        let second = Receipt {
            previous_hash: Some(h1),
            ..wrong
        };
        ledger.append_receipt(&second).unwrap();

        let entries = ledger.verify_chain().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, h1);

        // The trait path refuses a receipt that does not extend HEAD.
        let stale = Receipt {
            header: header(),
            ..second.clone()
        };
        let bytes = serde_json::to_vec(&stale).unwrap();
        assert!(ledger.append(stale.receipt_id(), &bytes).is_err());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn verify_chain_detects_tampering() {
        let dir = temp_dir("tamper");
        let ledger = FileLedger::new(&dir);
        let r = Receipt {
            header: header(),
            previous_receipt_id: None,
            previous_hash: None,
            operation_payload: serde_json::json!({ "kind": "event", "n": 1 }),
        };
        ledger.append_receipt(&r).unwrap();
        let path = dir.join("ledger.jsonl");
        let text = fs::read_to_string(&path)
            .unwrap()
            .replace("\"n\":1", "\"n\":2");
        fs::write(&path, text).unwrap();
        assert!(matches!(
            ledger.verify_chain(),
            Err(LedgerError::Integrity { .. })
        ));
        fs::remove_dir_all(dir).ok();
    }
}

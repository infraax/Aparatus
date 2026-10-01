//! Canonical JSON (RWS-2.0 X-04): object keys sorted by byte order, no
//! insignificant whitespace, integers only. Shared by the ledger hash and the
//! envelope payload hash / signing bytes, so both see the same bytes.

use serde_json::Value;

/// The value cannot be encoded canonically (e.g. it contains a float).
#[derive(Debug, thiserror::Error)]
pub enum CanonicalError {
    #[error("Non-canonical value: {0}")]
    NonCanonical(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Encode a JSON value canonically.
pub fn canonical_json(value: &Value) -> Result<Vec<u8>, CanonicalError> {
    let mut out = Vec::new();
    write_canonical(value, &mut out)?;
    Ok(out)
}

fn write_canonical(value: &Value, out: &mut Vec<u8>) -> Result<(), CanonicalError> {
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => {
            out.extend_from_slice(serde_json::to_string(value)?.as_bytes());
        }
        Value::Number(n) => {
            if n.is_f64() {
                return Err(CanonicalError::NonCanonical(format!(
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

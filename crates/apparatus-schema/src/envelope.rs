//! Envelope Stage 0 (NAP-corpus #12): payload addressing, signing bytes and
//! the stateless envelope rules.
//!
//! Rule ids `ENV-01..05` are Stage 0's own; RWS 2.0 has no envelope rules yet.
//! See `docs/milestones/ENVELOPE-STAGE-0.md`.

use crate::ValidationError;
use apparatus_crypto::signing::{
    ed25519_verify, frame_message, from_hex, key_id, ml_dsa_65_stub_verify, to_hex,
};
use apparatus_crypto::{canonical_json, CanonicalError, Cid, CidCodec, Sha256Digest, SigningKey};
use apparatus_types::rws::{
    EnvelopeSignature, EventEnvelope, Lifecycle, ProvenanceKind, SignatureScheme,
};
use serde_json::Value;

/// Domain tag in front of every envelope signature (NAP T6 D.3 framing).
pub const SIGNING_TAG: &[u8] = b"APPARATUS-ENVELOPE-SIG-v0\x00";

/// Largest canonical inline payload, in bytes (NAP T6 B.4: inline only <= 4 KB).
pub const MAX_INLINE_PAYLOAD: usize = 4096;

/// Keys outside the signature scope: the signature itself and the mutable
/// lifecycle state (NAP T6 D.2 excludes `signature` and `state`).
const UNSIGNED_KEYS: [&str; 2] = ["signature", "lifecycle"];

fn rule(rule: &'static str, reason: impl Into<String>) -> ValidationError {
    ValidationError::Rule {
        rule,
        reason: reason.into(),
    }
}

/// Canonical bytes of a payload: what `payload_hash` and `cid` address.
pub fn payload_bytes(payload: &Value) -> Result<Vec<u8>, CanonicalError> {
    canonical_json(payload)
}

/// `(payload_hash, cid)` of canonical payload bytes.
pub fn address(bytes: &[u8]) -> (String, String) {
    (
        format!("{:x}", Sha256Digest::compute(bytes)),
        Cid::compute(CidCodec::Raw, bytes).to_string(),
    )
}

/// The exact bytes a signature covers: the framed canonical JSON of the
/// envelope with `signature` and `lifecycle` removed. `signature_scheme` stays
/// in scope so a signature cannot be relabelled (NAP T1 B.5).
pub fn signing_bytes(env: &EventEnvelope) -> Result<Vec<u8>, CanonicalError> {
    let mut value = serde_json::to_value(env)?;
    if let Value::Object(map) = &mut value {
        for k in UNSIGNED_KEYS {
            map.remove(k);
        }
    }
    Ok(frame_message(SIGNING_TAG, &canonical_json(&value)?))
}

/// Sign `env` with `key`. The key's scheme must be the envelope's scheme.
pub fn sign(env: &mut EventEnvelope, key: &dyn SigningKey) -> Result<(), String> {
    if key.scheme() != env.signature_scheme.as_str() {
        return Err(format!(
            "key scheme {} does not match envelope scheme {}",
            key.scheme(),
            env.signature_scheme.as_str()
        ));
    }
    let bytes = signing_bytes(env).map_err(|e| e.to_string())?;
    let pk = key.public_key();
    env.signature = Some(EnvelopeSignature {
        signing_key_id: key_id(key.scheme(), &pk),
        public_key: to_hex(&pk),
        value: to_hex(&key.sign(&bytes)),
    });
    Ok(())
}

/// Verify `env.signature` under `env.signature_scheme`. `false` when unsigned.
pub fn verify(env: &EventEnvelope) -> bool {
    let Some(sig) = &env.signature else {
        return false;
    };
    let (Some(pk), Some(value)) = (from_hex(&sig.public_key), from_hex(&sig.value)) else {
        return false;
    };
    if sig.signing_key_id != key_id(env.signature_scheme.as_str(), &pk) {
        return false;
    }
    let Ok(bytes) = signing_bytes(env) else {
        return false;
    };
    match env.signature_scheme {
        SignatureScheme::Ed25519 => ed25519_verify(&pk, &bytes, &value),
        SignatureScheme::MlDsa65Stub => ml_dsa_65_stub_verify(&pk, &bytes, &value),
    }
}

/// Stateless envelope rules. A violation becomes a refused receipt on the chain.
/// Chain rules (ENV-05 transitions, ENV-07 sealed) live in `State::check_envelope`.
///
/// - ENV-01 required fields: `event_id`, `source`, `module` non-empty; `unix_timestamp` > 0.
/// - ENV-06 provenance: `measured` needs `evidence_tag`; `inferred` needs a non-empty `source_ref`.
/// - ENV-02 payload: canonical JSON (no floats), at most `MAX_INLINE_PAYLOAD` bytes.
/// - ENV-03 `payload_hash` and `cid` match the canonical payload bytes.
/// - ENV-04 lifecycle vs signature: `draft` carries none; every later state
///   carries one that verifies under `signature_scheme`.
/// - ENV-08 `anchored` is refused: anchoring is Stage 3 (#15).
pub fn validate(env: &EventEnvelope) -> Result<(), ValidationError> {
    for (field, value) in [
        ("event_id", &env.event_id),
        ("source", &env.source),
        ("module", &env.module),
    ] {
        if value.trim().is_empty() {
            return Err(rule("ENV-01", format!("envelope needs a {field}")));
        }
    }
    if env.unix_timestamp == 0 {
        return Err(rule("ENV-01", "envelope needs a unix_timestamp"));
    }
    let bytes = payload_bytes(&env.payload)
        .map_err(|e| rule("ENV-02", format!("payload is not canonical: {e}")))?;
    if bytes.len() > MAX_INLINE_PAYLOAD {
        return Err(rule(
            "ENV-02",
            format!(
                "payload is {} bytes; inline payloads are at most {MAX_INLINE_PAYLOAD}",
                bytes.len()
            ),
        ));
    }
    let (hash, cid) = address(&bytes);
    if env.payload_hash != hash {
        return Err(rule("ENV-03", "payload_hash does not match the payload"));
    }
    if env.cid != cid {
        return Err(rule("ENV-03", "cid does not match the payload"));
    }
    match (env.lifecycle, &env.signature) {
        (Lifecycle::Draft, None) => {}
        (Lifecycle::Draft, Some(_)) => {
            return Err(rule("ENV-04", "a draft envelope carries no signature"));
        }
        (_, None) => {
            return Err(rule(
                "ENV-04",
                format!("a {:?} envelope needs a signature", env.lifecycle).to_lowercase(),
            ));
        }
        (_, Some(_)) => {
            if !verify(env) {
                return Err(rule(
                    "ENV-04",
                    format!(
                        "signature does not verify under {}",
                        env.signature_scheme.as_str()
                    ),
                ));
            }
        }
    }
    match env.provenance {
        ProvenanceKind::Measured if env.evidence_tag.is_none() => {
            return Err(rule(
                "ENV-06",
                "measured provenance needs an evidence_tag (A, B, C, E or NF)",
            ));
        }
        ProvenanceKind::Inferred if env.source_ref.as_deref().unwrap_or("").trim().is_empty() => {
            return Err(rule("ENV-06", "inferred provenance needs a source_ref"));
        }
        _ => {}
    }
    if env.lifecycle == Lifecycle::Anchored {
        return Err(rule(
            "ENV-08",
            "anchoring is Stage 3; an envelope cannot be anchored yet",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use apparatus_crypto::{Ed25519Key, MlDsa65Stub};
    use apparatus_types::rws::EvidenceTag;

    fn draft(payload: Value) -> EventEnvelope {
        let bytes = payload_bytes(&payload).unwrap();
        let (payload_hash, cid) = address(&bytes);
        EventEnvelope {
            event_id: "evt-1".into(),
            unix_timestamp: 1_790_401_238,
            source: "sensor".into(),
            module: "dexos.test".into(),
            provenance: ProvenanceKind::Measured,
            evidence_tag: Some(EvidenceTag::C),
            source_ref: None,
            payload,
            payload_hash,
            cid,
            lifecycle: Lifecycle::Draft,
            signature_scheme: SignatureScheme::Ed25519,
            signature: None,
        }
    }

    fn code(r: Result<(), ValidationError>) -> &'static str {
        match r {
            Err(ValidationError::Rule { rule, .. }) => rule,
            other => panic!("expected a rule violation, got {other:?}"),
        }
    }

    #[test]
    fn draft_and_ed25519_signed_envelopes_validate() {
        let env = draft(serde_json::json!({ "temp_c": 21, "room": "lab" }));
        validate(&env).unwrap();
        assert!(env.cid.starts_with("bafkr4i"));

        let key = Ed25519Key::generate().unwrap();
        let mut signed = env.clone();
        signed.lifecycle = Lifecycle::Signed;
        sign(&mut signed, &key).unwrap();
        validate(&signed).unwrap();
        // Lifecycle is outside the signature scope: declaring `sealed` keeps it valid.
        signed.lifecycle = Lifecycle::Sealed;
        validate(&signed).unwrap();
    }

    #[test]
    fn tampering_breaks_the_signature() {
        let key = Ed25519Key::generate().unwrap();
        let mut env = draft(serde_json::json!({ "n": 1 }));
        env.lifecycle = Lifecycle::Signed;
        sign(&mut env, &key).unwrap();

        let mut moved = env.clone();
        moved.module = "dexos.other".into();
        assert_eq!(code(validate(&moved)), "ENV-04");

        let mut relabelled = env.clone();
        relabelled.provenance = ProvenanceKind::Inferred;
        assert_eq!(code(validate(&relabelled)), "ENV-04");

        // Swapping the scheme label is caught (key id and scope bind it).
        let mut rescheme = env.clone();
        rescheme.signature_scheme = SignatureScheme::MlDsa65Stub;
        assert_eq!(code(validate(&rescheme)), "ENV-04");

        let mut forged = env;
        forged.payload = serde_json::json!({ "n": 2 });
        assert_eq!(code(validate(&forged)), "ENV-03");
    }

    #[test]
    fn ml_dsa_stub_verifies_by_design() {
        let mut env = draft(serde_json::json!({ "n": 1 }));
        env.signature_scheme = SignatureScheme::MlDsa65Stub;
        env.lifecycle = Lifecycle::Signed;
        let stub = MlDsa65Stub::new(vec![9; 32]);
        sign(&mut env, &stub).unwrap();
        assert!(verify(&env));
        // The stub authenticates nothing: a garbage value still verifies.
        env.signature.as_mut().unwrap().value = "00".into();
        assert!(verify(&env));
        validate(&env).unwrap();
        // A key of another scheme cannot sign under the stub's name.
        let key = Ed25519Key::generate().unwrap();
        assert!(sign(&mut env, &key).is_err());
    }

    #[test]
    fn bad_envelopes_name_their_rule() {
        let mut e = draft(serde_json::json!({}));
        e.module = " ".into();
        assert_eq!(code(validate(&e)), "ENV-01");

        let mut e = draft(serde_json::json!({}));
        e.unix_timestamp = 0;
        assert_eq!(code(validate(&e)), "ENV-01");

        let mut e = draft(serde_json::json!({}));
        e.payload = serde_json::json!({ "x": 1.5 });
        assert_eq!(code(validate(&e)), "ENV-02");

        let big = "x".repeat(MAX_INLINE_PAYLOAD);
        assert_eq!(
            code(validate(&draft(serde_json::json!({ "blob": big })))),
            "ENV-02"
        );

        let mut e = draft(serde_json::json!({ "a": 1 }));
        e.cid = Cid::compute(CidCodec::DagCbor, b"{\"a\":1}").to_string();
        assert_eq!(code(validate(&e)), "ENV-03");

        let mut e = draft(serde_json::json!({}));
        e.lifecycle = Lifecycle::Anchored;
        assert_eq!(code(validate(&e)), "ENV-04");

        let key = Ed25519Key::generate().unwrap();
        let mut e = draft(serde_json::json!({}));
        e.lifecycle = Lifecycle::Signed;
        sign(&mut e, &key).unwrap();
        e.lifecycle = Lifecycle::Draft;
        assert_eq!(code(validate(&e)), "ENV-04");
    }

    #[test]
    fn payload_address_ignores_key_order() {
        let a: Value = serde_json::from_str(r#"{"b":1,"a":[2,{"d":0,"c":1}]}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"a":[2,{"c":1,"d":0}],"b":1}"#).unwrap();
        assert_eq!(
            address(&payload_bytes(&a).unwrap()),
            address(&payload_bytes(&b).unwrap())
        );
    }
}

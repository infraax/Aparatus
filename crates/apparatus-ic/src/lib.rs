//! Read-only client for a local IC replica, built on the official `ic-agent`
//! (dfinity/agent-rs). It reads the public status endpoint and nothing else.
//! It never starts, stops or configures a replica.

use ic_agent::agent::status::Value;
use ic_agent::export::{reqwest, Principal};
use ic_agent::{Agent, AgentError};
use sha2::{Digest, Sha256};
use std::time::Duration;

/// Version of the `ic-agent` crate this client is built against (pinned in Cargo.toml).
pub const IC_AGENT_VERSION: &str = "0.49.2";

/// Default replica API address used by `scripts/ic-up.sh`.
pub const DEFAULT_URL: &str = "http://127.0.0.1:8080";

/// Path of the public status endpoint, relative to the replica URL.
pub const STATUS_PATH: &str = "/api/v2/status";

const TIMEOUT: Duration = Duration::from_secs(5);

/// What the replica said about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicaStatus {
    /// `replica_health_status`, e.g. `healthy` or `waiting_for_certified_state`.
    pub health: Option<String>,
    /// `certified_height`: the latest height whose state the replica certified.
    pub certified_height: Option<u64>,
    /// `impl_version`: the replica commit.
    pub impl_version: Option<String>,
    /// Lowercase hex SHA-256 of the DER root key.
    pub root_key_sha256: Option<String>,
    /// Self-authenticating principal of the DER root key. Not a field of the agent's
    /// `Status`; derived here. On a root subnet it equals the subnet id in the replica log.
    pub root_key_principal: Option<String>,
}

/// Why a status read failed. A failed read never starts anything.
#[derive(Debug, thiserror::Error)]
pub enum IcError {
    /// The URL could not be used to build an agent.
    #[error("invalid replica url {url}: {reason}")]
    InvalidUrl { url: String, reason: String },
    /// Nothing answered: connection refused, timeout, DNS, TLS.
    #[error("replica unreachable at {url}: {reason}")]
    Unreachable { url: String, reason: String },
    /// Something answered, but not with a valid replica status.
    #[error("bad status response from {url}: {reason}")]
    BadResponse { url: String, reason: String },
}

impl IcError {
    /// True when the replica is down or not reachable.
    pub fn is_unreachable(&self) -> bool {
        matches!(self, IcError::Unreachable { .. })
    }
}

/// Full status URL for a replica base URL.
pub fn status_url(base: &str) -> String {
    format!("{}{STATUS_PATH}", base.trim_end_matches('/'))
}

/// Read `GET <base>/api/v2/status` through `ic-agent`. Blocking; at most ~5 s.
pub fn status(base: &str) -> Result<ReplicaStatus, IcError> {
    let invalid = |reason: String| IcError::InvalidUrl {
        url: base.to_string(),
        reason,
    };
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .connect_timeout(TIMEOUT)
        .build()
        .map_err(|e| invalid(e.to_string()))?;
    let agent = Agent::builder()
        .with_url(base)
        .with_http_client(client)
        .with_max_tcp_error_retries(0)
        .build()
        .map_err(|e| invalid(e.to_string()))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| invalid(format!("tokio runtime: {e}")))?;
    let status = runtime
        .block_on(agent.status())
        .map_err(|e| classify(base, e))?;
    let certified_height = match status.values.get("certified_height").map(|v| v.as_ref()) {
        Some(Value::Integer(h)) if *h >= 0 => Some(*h as u64),
        _ => None,
    };
    let root_key = status.root_key.as_deref();
    Ok(ReplicaStatus {
        health: status.replica_health_status,
        certified_height,
        impl_version: status.impl_version,
        root_key_sha256: root_key.map(|k| hex(&Sha256::digest(k))),
        root_key_principal: root_key.map(|k| Principal::self_authenticating(k).to_text()),
    })
}

fn classify(base: &str, e: AgentError) -> IcError {
    let url = base.to_string();
    match e {
        AgentError::TransportError(t) => IcError::Unreachable {
            url,
            reason: t.to_string(),
        },
        other => IcError::BadResponse {
            url,
            reason: other.to_string(),
        },
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn down_replica_is_a_typed_unreachable_error() {
        // Bind and drop: the port is free and nothing listens on it.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let err = status(&format!("http://127.0.0.1:{port}")).unwrap_err();
        assert!(err.is_unreachable(), "{err:?}");
    }

    /// Minimal CBOR for a status map: text keys; text, unsigned or byte-string values.
    enum C<'a> {
        T(&'a str),
        U(u64),
        B(&'a [u8]),
    }

    fn head(major: u8, n: u64, out: &mut Vec<u8>) {
        let m = major << 5;
        match n {
            0..=23 => out.push(m | n as u8),
            24..=0xff => out.extend([m | 24, n as u8]),
            0x100..=0xffff => {
                out.push(m | 25);
                out.extend((n as u16).to_be_bytes());
            }
            _ => {
                out.push(m | 27);
                out.extend(n.to_be_bytes());
            }
        }
    }

    fn status_cbor(fields: &[(&str, C)]) -> Vec<u8> {
        let mut out = vec![0xd9, 0xd9, 0xf7]; // self-describe tag 55799
        head(5, fields.len() as u64, &mut out);
        for (k, v) in fields {
            head(3, k.len() as u64, &mut out);
            out.extend(k.as_bytes());
            match v {
                C::T(t) => {
                    head(3, t.len() as u64, &mut out);
                    out.extend(t.as_bytes());
                }
                C::U(u) => head(0, *u, &mut out),
                C::B(b) => {
                    head(2, b.len() as u64, &mut out);
                    out.extend(*b);
                }
            }
        }
        out
    }

    /// Serve `body` as application/cbor to every request on a fresh local port.
    fn serve(body: Vec<u8>) -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut s = stream.unwrap();
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                let header = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/cbor\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = s.write_all(header.as_bytes());
                let _ = s.write_all(&body);
            }
        });
        url
    }

    #[test]
    fn reads_health_height_and_root_key_through_ic_agent() {
        let key = [7u8; 133];
        let url = serve(status_cbor(&[
            (
                "impl_version",
                C::T("d26cd031176beec51b39fbb9e39e80a3a46a748e"),
            ),
            ("replica_health_status", C::T("healthy")),
            ("certified_height", C::U(4242)),
            ("root_key", C::B(&key)),
        ]));
        let s = status(&url).unwrap();
        assert_eq!(s.health.as_deref(), Some("healthy"));
        assert_eq!(s.certified_height, Some(4242));
        assert_eq!(
            s.impl_version.as_deref(),
            Some("d26cd031176beec51b39fbb9e39e80a3a46a748e")
        );
        assert_eq!(s.root_key_sha256.unwrap().len(), 64);
        assert_eq!(
            s.root_key_principal.as_deref(),
            Some(Principal::self_authenticating(key).to_text().as_str())
        );
    }

    #[test]
    fn non_status_body_is_a_bad_response_not_unreachable() {
        let url = serve(b"not cbor".to_vec());
        let err = status(&url).unwrap_err();
        assert!(matches!(err, IcError::BadResponse { .. }), "{err:?}");
    }

    #[test]
    fn status_url_joins_the_path() {
        assert_eq!(
            status_url("http://127.0.0.1:8080/"),
            "http://127.0.0.1:8080/api/v2/status"
        );
    }
}

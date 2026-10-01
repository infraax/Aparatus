//! JSON-lines RPC over the project's Unix socket.
//!
//! Clients find the daemon at `<project>/.apparatus/apparatusd.sock`. A socket
//! file that refuses connections is stale (daemon gone): it is removed and the
//! caller falls back to local mode, which takes `.apparatus/LOCK` itself.

use crate::api::{Request, Response};
use crate::kernel::apparatus_dir;
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Socket path for a project.
pub fn socket_path(root: &Path) -> PathBuf {
    apparatus_dir(root).join("apparatusd.sock")
}

/// Connect to a running daemon. `Ok(None)`: no daemon (no socket, or a stale
/// one that was removed).
pub fn connect(root: &Path) -> Result<Option<UnixStream>> {
    let path = socket_path(root);
    if !path.exists() {
        return Ok(None);
    }
    match UnixStream::connect(&path) {
        Ok(stream) => Ok(Some(stream)),
        Err(e) if matches!(e.kind(), ErrorKind::ConnectionRefused | ErrorKind::NotFound) => {
            std::fs::remove_file(&path).ok();
            eprintln!(
                "note: removed stale socket {} (no daemon listening); running locally",
                path.display()
            );
            Ok(None)
        }
        Err(e) => Err(e).with_context(|| format!("connecting to {}", path.display())),
    }
}

/// A connected client. Requests on one connection are answered in order.
pub struct Client {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl Client {
    pub fn new(stream: UnixStream) -> Result<Self> {
        // Generous: the daemon answers `busy` itself well before this.
        stream.set_read_timeout(Some(Duration::from_secs(120)))?;
        Ok(Self {
            writer: stream.try_clone()?,
            reader: BufReader::new(stream),
        })
    }

    /// Send one request and wait for its response.
    pub fn call(&mut self, req: &Request) -> Result<Response> {
        let mut line = serde_json::to_string(req)?;
        line.push('\n');
        self.call_raw(&line)
    }

    /// Send one raw line (tests use this for malformed input).
    pub fn call_raw(&mut self, line: &str) -> Result<Response> {
        self.writer.write_all(line.as_bytes())?;
        self.writer.flush()?;
        let mut reply = String::new();
        let n = self.reader.read_line(&mut reply)?;
        if n == 0 {
            anyhow::bail!("daemon closed the connection");
        }
        serde_json::from_str(&reply).context("daemon sent an unreadable response")
    }
}

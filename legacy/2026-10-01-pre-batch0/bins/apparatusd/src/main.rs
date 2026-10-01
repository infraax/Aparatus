//! `apparatusd`: the always-on process around the single-writer kernel.
//!
//! many clients → one Unix socket → one writer actor → one FileLedger + CAS.
//!
//! Connections are accepted concurrently (one thread each). Every request —
//! read or write — is handed to the writer actor over a bounded channel, so
//! the chain keeps exactly one writer. A full channel answers `busy`; it never
//! spawns a second writer. A panicking handler kills that request only: the
//! actor reloads the chain from disk, records a ticket, answers `error`, and
//! carries on. The daemon stops on SIGTERM/SIGINT, not when a client leaves.

#[cfg(unix)]
mod daemon;

#[cfg(unix)]
fn main() -> std::process::ExitCode {
    daemon::main()
}

#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("apparatusd needs Unix domain sockets; use the `apparatus` CLI in local mode");
    std::process::ExitCode::from(1)
}

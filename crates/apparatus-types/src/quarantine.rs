//! Package quarantine (Aparatus issue #21): what a client observed about one
//! lockfile pin, and advisories from a feed. The writer applies the rules.

use serde::{Deserialize, Serialize};

/// Default wait before a pinned version may be adopted, in days. Not zero.
pub const DEFAULT_WAIT_DAYS: u64 = 5;

/// Package registry family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ecosystem {
    /// crates.io (`Cargo.lock`).
    Cargo,
    /// npm registry (`package-lock.json`, `pnpm-lock.yaml`).
    Npm,
}

impl Ecosystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ecosystem::Cargo => "cargo",
            Ecosystem::Npm => "npm",
        }
    }
}

/// One pin from a lockfile, with what the registry says about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinObservation {
    pub ecosystem: Ecosystem,
    pub name: String,
    pub version: String,
    /// Hash as written in the lockfile (`checksum` sha256 hex, or npm `integrity`).
    pub lock_integrity: String,
    /// Hash the registry publishes for this version; `None` if it could not be read.
    #[serde(default)]
    pub registry_integrity: Option<String>,
    /// Registry publish time, seconds since the epoch; `None` if unknown.
    #[serde(default)]
    pub published_at: Option<u64>,
    /// Where the registry facts came from (index URL, registry URL, or `fixture:<path>`).
    pub registry: String,
    /// Why the registry facts are missing, if they are.
    #[serde(default)]
    pub lookup_error: Option<String>,
    /// Lockfile the pin came from.
    pub lockfile: String,
}

/// An advisory from a feed: these versions of this package are known bad.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvisoryRecord {
    /// Feed name, e.g. `fixture`, `osv`, `rustsec`.
    pub feed: String,
    /// Advisory id in that feed.
    pub id: String,
    pub ecosystem: Ecosystem,
    pub package: String,
    pub bad_versions: Vec<String>,
    #[serde(default)]
    pub recommended: Option<String>,
    pub url: String,
}

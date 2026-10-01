//! Client half of `rws quarantine` (Aparatus issue #21): read lockfiles, look each
//! registry pin up, load an advisory feed. It never writes a lockfile. The writer
//! (kernel) decides waiting / adopted / refused and records it on the chain.

use apparatus_types::quarantine::{AdvisoryRecord, Ecosystem, PinObservation};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// A pin read from a lockfile, before any registry lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockPin {
    pub ecosystem: Ecosystem,
    pub name: String,
    pub version: String,
    pub integrity: String,
    pub lockfile: String,
}

#[derive(Debug, thiserror::Error)]
pub enum QuarantineError {
    #[error("reading {path}: {reason}")]
    Read { path: String, reason: String },
    #[error("{path}: {reason}")]
    Parse { path: String, reason: String },
}

fn read(path: &Path) -> Result<String, QuarantineError> {
    std::fs::read_to_string(path).map_err(|e| QuarantineError::Read {
        path: path.display().to_string(),
        reason: e.to_string(),
    })
}

/// Lockfiles to scan in `root`: `Cargo.lock`, and `package-lock.json` / `pnpm-lock.yaml` if present.
pub fn default_lockfiles(root: &Path) -> Vec<std::path::PathBuf> {
    ["Cargo.lock", "package-lock.json", "pnpm-lock.yaml"]
        .iter()
        .map(|f| root.join(f))
        .filter(|p| p.exists())
        .collect()
}

/// Parse any supported lockfile by its file name.
pub fn parse_lockfile(path: &Path) -> Result<Vec<LockPin>, QuarantineError> {
    let text = read(path)?;
    let label = path.display().to_string();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.ends_with(".lock") {
        Ok(parse_cargo_lock(&text, &label))
    } else if name.ends_with("package-lock.json") {
        parse_package_lock(&text, &label)
    } else if name.ends_with("pnpm-lock.yaml") {
        Ok(parse_pnpm_lock(&text, &label))
    } else {
        Err(QuarantineError::Parse {
            path: label,
            reason: "unknown lockfile type (Cargo.lock, package-lock.json, pnpm-lock.yaml)".into(),
        })
    }
}

/// `Cargo.lock`: registry packages only (they have `source = "registry+…"` and a `checksum`).
/// Path and git dependencies have no registry hash and are skipped.
pub fn parse_cargo_lock(text: &str, label: &str) -> Vec<LockPin> {
    let mut out = Vec::new();
    let mut cur: BTreeMap<&str, String> = BTreeMap::new();
    let mut flush = |cur: &mut BTreeMap<&str, String>| {
        if let (Some(n), Some(v), Some(s), Some(c)) = (
            cur.get("name"),
            cur.get("version"),
            cur.get("source"),
            cur.get("checksum"),
        ) {
            if s.starts_with("registry+") {
                out.push(LockPin {
                    ecosystem: Ecosystem::Cargo,
                    name: n.clone(),
                    version: v.clone(),
                    integrity: c.clone(),
                    lockfile: label.to_string(),
                });
            }
        }
        cur.clear();
    };
    for line in text.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            flush(&mut cur);
            continue;
        }
        if let Some((k, v)) = line.split_once(" = ") {
            if matches!(k, "name" | "version" | "source" | "checksum") && v.starts_with('"') {
                cur.insert(k, v.trim_matches('"').to_string());
            }
        }
    }
    flush(&mut cur);
    out
}

/// `package-lock.json` lockfileVersion 2 or 3 (`packages` map). Version 1 is refused.
pub fn parse_package_lock(text: &str, label: &str) -> Result<Vec<LockPin>, QuarantineError> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| QuarantineError::Parse {
        path: label.into(),
        reason: e.to_string(),
    })?;
    let Some(packages) = v.get("packages").and_then(|p| p.as_object()) else {
        return Err(QuarantineError::Parse {
            path: label.into(),
            reason: "no `packages` map (lockfileVersion 1 is not supported)".into(),
        });
    };
    let mut out = Vec::new();
    for (key, p) in packages {
        let Some(idx) = key.rfind("node_modules/") else {
            continue; // "" is the root project
        };
        let name = &key[idx + "node_modules/".len()..];
        let (Some(version), Some(integrity)) = (
            p.get("version").and_then(|x| x.as_str()),
            p.get("integrity").and_then(|x| x.as_str()),
        ) else {
            continue; // linked / bundled entries carry no registry integrity
        };
        out.push(LockPin {
            ecosystem: Ecosystem::Npm,
            name: name.to_string(),
            version: version.to_string(),
            integrity: integrity.to_string(),
            lockfile: label.to_string(),
        });
    }
    Ok(out)
}

/// `pnpm-lock.yaml` (v6 and v9 key forms) without a YAML library: reads the
/// `packages:` section, entries `name@version:` followed by `resolution: {integrity: …}`.
pub fn parse_pnpm_lock(text: &str, label: &str) -> Vec<LockPin> {
    let mut out = Vec::new();
    let mut in_packages = false;
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        if !line.starts_with(' ') && !line.trim().is_empty() {
            in_packages = line.trim_end() == "packages:";
            current = None;
            continue;
        }
        if !in_packages {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let t = line.trim();
        if indent == 2 && t.ends_with(':') {
            let key = t
                .trim_end_matches(':')
                .trim_matches(|c| c == '\'' || c == '"');
            let key = key.trim_start_matches('/');
            let key = key.split('(').next().unwrap_or(key);
            current = key
                .rfind('@')
                .filter(|&i| i > 0)
                .map(|i| (key[..i].to_string(), key[i + 1..].to_string()));
        } else if let (Some((name, version)), Some(rest)) =
            (&current, t.strip_prefix("resolution: {integrity: "))
        {
            let integrity = rest
                .trim_end_matches('}')
                .split(',')
                .next()
                .unwrap_or("")
                .trim();
            out.push(LockPin {
                ecosystem: Ecosystem::Npm,
                name: name.clone(),
                version: version.clone(),
                integrity: integrity.to_string(),
                lockfile: label.to_string(),
            });
        }
    }
    out
}

/// crates.io sparse-index path for a crate name.
pub fn crates_index_path(name: &str) -> String {
    let n = name.to_lowercase();
    match n.len() {
        1 => format!("1/{n}"),
        2 => format!("2/{n}"),
        3 => format!("3/{}/{n}", &n[..1]),
        _ => format!("{}/{}/{n}", &n[..2], &n[2..4]),
    }
}

fn parse_time(s: &str) -> Option<u64> {
    OffsetDateTime::parse(s, &Rfc3339)
        .ok()
        .and_then(|t| u64::try_from(t.unix_timestamp()).ok())
}

/// Where registry facts come from.
pub enum Registry {
    /// crates.io sparse index (`cksum`, `pubtime`) and registry.npmjs.org (`dist.integrity`, `time`).
    Online { client: reqwest::blocking::Client },
    /// A local JSON file: `[{ecosystem, name, version, integrity, published_at (RFC 3339)}]`.
    Fixture {
        label: String,
        entries: BTreeMap<(Ecosystem, String, String), (String, Option<u64>)>,
    },
}

#[derive(Deserialize)]
struct FixtureEntry {
    ecosystem: Ecosystem,
    name: String,
    version: String,
    integrity: String,
    #[serde(default)]
    published_at: Option<String>,
}

impl Registry {
    pub fn online() -> Result<Self, QuarantineError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("apparatus-quarantine/0.1 (+https://github.com/infraax/Aparatus)")
            .build()
            .map_err(|e| QuarantineError::Read {
                path: "http client".into(),
                reason: e.to_string(),
            })?;
        Ok(Registry::Online { client })
    }

    pub fn fixture(path: &Path) -> Result<Self, QuarantineError> {
        let label = format!("fixture:{}", path.display());
        let list: Vec<FixtureEntry> =
            serde_json::from_str(&read(path)?).map_err(|e| QuarantineError::Parse {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;
        let entries = list
            .into_iter()
            .map(|e| {
                let at = e.published_at.as_deref().and_then(parse_time);
                ((e.ecosystem, e.name, e.version), (e.integrity, at))
            })
            .collect();
        Ok(Registry::Fixture { label, entries })
    }

    /// Look every pin up. Online lookups run on a few threads, one request per package name.
    pub fn observe(&self, pins: &[LockPin]) -> Vec<PinObservation> {
        match self {
            Registry::Fixture { label, entries } => pins
                .iter()
                .map(|p| {
                    let hit = entries.get(&(p.ecosystem, p.name.clone(), p.version.clone()));
                    observation(
                        p,
                        label.clone(),
                        hit.map(|(i, at)| (i.clone(), *at)),
                        hit.is_none().then(|| "not in fixture registry".to_string()),
                    )
                })
                .collect(),
            Registry::Online { client } => {
                let mut names: Vec<(Ecosystem, String)> =
                    pins.iter().map(|p| (p.ecosystem, p.name.clone())).collect();
                names.sort();
                names.dedup();
                let docs: BTreeMap<(Ecosystem, String), (String, Result<String, String>)> =
                    std::thread::scope(|s| {
                        let chunks: Vec<_> = names.chunks(names.len().div_ceil(8).max(1)).collect();
                        let handles: Vec<_> = chunks
                            .into_iter()
                            .map(|chunk| {
                                s.spawn(move || {
                                    chunk
                                        .iter()
                                        .map(|(eco, name)| {
                                            ((*eco, name.clone()), fetch(client, *eco, name))
                                        })
                                        .collect::<Vec<_>>()
                                })
                            })
                            .collect();
                        handles
                            .into_iter()
                            .flat_map(|h| h.join().unwrap())
                            .collect()
                    });
                pins.iter()
                    .map(|p| {
                        let (url, doc) = &docs[&(p.ecosystem, p.name.clone())];
                        match doc {
                            Ok(body) => {
                                let found = match p.ecosystem {
                                    Ecosystem::Cargo => cargo_version(body, &p.version),
                                    Ecosystem::Npm => npm_version(body, &p.version),
                                };
                                let err = found
                                    .is_none()
                                    .then(|| format!("version {} not in registry", p.version));
                                observation(p, url.clone(), found, err)
                            }
                            Err(e) => observation(p, url.clone(), None, Some(e.clone())),
                        }
                    })
                    .collect()
            }
        }
    }
}

fn observation(
    p: &LockPin,
    registry: String,
    found: Option<(String, Option<u64>)>,
    lookup_error: Option<String>,
) -> PinObservation {
    PinObservation {
        ecosystem: p.ecosystem,
        name: p.name.clone(),
        version: p.version.clone(),
        lock_integrity: p.integrity.clone(),
        registry_integrity: found.as_ref().map(|f| f.0.clone()),
        published_at: found.and_then(|f| f.1),
        registry,
        lookup_error,
        lockfile: p.lockfile.clone(),
    }
}

fn fetch(
    client: &reqwest::blocking::Client,
    eco: Ecosystem,
    name: &str,
) -> (String, Result<String, String>) {
    let url = match eco {
        Ecosystem::Cargo => format!("https://index.crates.io/{}", crates_index_path(name)),
        Ecosystem::Npm => format!("https://registry.npmjs.org/{}", name.replace('/', "%2f")),
    };
    let body = client
        .get(&url)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.text())
        .map_err(|e| e.to_string());
    (url, body)
}

/// From a sparse-index file (one JSON object per line): `(cksum, pubtime)` for `version`.
pub fn cargo_version(index: &str, version: &str) -> Option<(String, Option<u64>)> {
    index.lines().find_map(|l| {
        let v: serde_json::Value = serde_json::from_str(l).ok()?;
        (v.get("vers")?.as_str()? == version).then(|| {
            (
                v.get("cksum")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string(),
                v.get("pubtime")
                    .and_then(|t| t.as_str())
                    .and_then(parse_time),
            )
        })
    })
}

/// From an npm packument: `(dist.integrity, time[version])`.
pub fn npm_version(doc: &str, version: &str) -> Option<(String, Option<u64>)> {
    let v: serde_json::Value = serde_json::from_str(doc).ok()?;
    let integrity = v
        .get("versions")?
        .get(version)?
        .get("dist")?
        .get("integrity")?
        .as_str()?
        .to_string();
    let at = v
        .get("time")
        .and_then(|t| t.get(version))
        .and_then(|t| t.as_str())
        .and_then(parse_time);
    Some((integrity, at))
}

#[derive(Deserialize)]
struct Feed {
    feed: String,
    advisories: Vec<FeedEntry>,
}

#[derive(Deserialize)]
struct FeedEntry {
    id: String,
    ecosystem: Ecosystem,
    package: String,
    bad_versions: Vec<String>,
    #[serde(default)]
    recommended: Option<String>,
    url: String,
}

/// A local advisory feed file: `{"feed": "...", "advisories": [{id, ecosystem, package, bad_versions, recommended, url}]}`.
pub fn load_feed(path: &Path) -> Result<Vec<AdvisoryRecord>, QuarantineError> {
    let feed: Feed = serde_json::from_str(&read(path)?).map_err(|e| QuarantineError::Parse {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    Ok(feed
        .advisories
        .into_iter()
        .map(|a| AdvisoryRecord {
            feed: feed.feed.clone(),
            id: a.id,
            ecosystem: a.ecosystem,
            package: a.package,
            bad_versions: a.bad_versions,
            recommended: a.recommended,
            url: a.url,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_lock_keeps_registry_pins_only() {
        let lock = r#"
version = 4

[[package]]
name = "local-crate"
version = "0.1.0"

[[package]]
name = "serde"
version = "1.0.200"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaa"

[[package]]
name = "gitdep"
version = "0.2.0"
source = "git+https://example.org/x#abc"
"#;
        let pins = parse_cargo_lock(lock, "Cargo.lock");
        assert_eq!(pins.len(), 1);
        assert_eq!(pins[0].name, "serde");
        assert_eq!(pins[0].integrity, "aaaa");
    }

    #[test]
    fn package_lock_v3_and_pnpm_v9() {
        let npm = r#"{"lockfileVersion":3,"packages":{"":{"name":"app"},
            "node_modules/left-pad":{"version":"1.3.0","integrity":"sha512-AAA"},
            "node_modules/@scope/x/node_modules/y":{"version":"2.0.0","integrity":"sha512-BBB"}}}"#;
        let pins = parse_package_lock(npm, "package-lock.json").unwrap();
        assert_eq!(pins.len(), 2);
        assert!(pins.iter().any(|p| p.name == "y" && p.version == "2.0.0"));
        assert!(parse_package_lock(r#"{"lockfileVersion":1,"dependencies":{}}"#, "p").is_err());

        let pnpm = "lockfileVersion: '9.0'\n\nimporters:\n  .:\n    dependencies: {}\n\npackages:\n\n  '@scope/pkg@1.2.3':\n    resolution: {integrity: sha512-CCC}\n\n  left-pad@1.3.0:\n    resolution: {integrity: sha512-DDD}\n\nsnapshots:\n\n  left-pad@1.3.0: {}\n";
        let pins = parse_pnpm_lock(pnpm, "pnpm-lock.yaml");
        assert_eq!(pins.len(), 2);
        assert_eq!(pins[0].name, "@scope/pkg");
        assert_eq!(pins[0].integrity, "sha512-CCC");
        assert_eq!(pins[1].name, "left-pad");
    }

    #[test]
    fn crates_index_paths_and_versions() {
        assert_eq!(crates_index_path("a"), "1/a");
        assert_eq!(crates_index_path("ab"), "2/ab");
        assert_eq!(crates_index_path("abc"), "3/a/abc");
        assert_eq!(crates_index_path("Serde"), "se/rd/serde");
        let idx = "{\"vers\":\"1.0.0\",\"cksum\":\"c1\",\"pubtime\":\"2026-07-18T23:00:00Z\"}\n{\"vers\":\"1.0.1\",\"cksum\":\"c2\"}";
        assert_eq!(cargo_version(idx, "1.0.0").unwrap().0, "c1");
        assert!(cargo_version(idx, "1.0.0").unwrap().1.is_some());
        assert_eq!(cargo_version(idx, "1.0.1").unwrap(), ("c2".into(), None));
        assert!(cargo_version(idx, "9.9.9").is_none());
        let doc = r#"{"versions":{"1.3.0":{"dist":{"integrity":"sha512-X"}}},"time":{"1.3.0":"2018-04-09T01:10:45.796Z"}}"#;
        assert_eq!(npm_version(doc, "1.3.0").unwrap().0, "sha512-X");
    }
}

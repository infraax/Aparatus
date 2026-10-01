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

/// OSV (`api.osv.dev`) is the live feed. It also serves the RustSec advisory database
/// (`RUSTSEC-*` ids), so one query covers both. Advisories become Advice; nothing is applied.
pub const OSV_URL: &str = "https://api.osv.dev";

fn osv_ecosystem(e: Ecosystem) -> &'static str {
    match e {
        Ecosystem::Cargo => "crates.io",
        Ecosystem::Npm => "npm",
    }
}

/// Body of `POST /v1/querybatch`: one query per pin.
pub fn osv_batch_body(pins: &[LockPin]) -> serde_json::Value {
    serde_json::json!({"queries": pins.iter().map(|p| serde_json::json!({
        "package": {"name": p.name, "ecosystem": osv_ecosystem(p.ecosystem)},
        "version": p.version,
    })).collect::<Vec<_>>()})
}

/// `(pin index, vulnerability id)` for every hit in a querybatch response.
pub fn osv_batch_hits(resp: &serde_json::Value) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, r) in resp
        .get("results")
        .and_then(|r| r.as_array())
        .into_iter()
        .flatten()
        .enumerate()
    {
        for v in r
            .get("vulns")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                out.push((i, id.to_string()));
            }
        }
    }
    out
}

fn version_key(v: &str) -> Vec<u64> {
    v.split(['.', '-', '+'])
        .map_while(|x| x.parse::<u64>().ok())
        .collect()
}

/// One OSV vulnerability record → an `AdvisoryRecord` for `pin`, or `None` if withdrawn.
/// `bad_versions` is the pinned version (OSV matched it). `recommended` is the lowest
/// `fixed` version above the pin for that package, if OSV lists one.
pub fn osv_record(pin: &LockPin, vuln: &serde_json::Value) -> Option<AdvisoryRecord> {
    if vuln.get("withdrawn").is_some() {
        return None;
    }
    let id = vuln.get("id")?.as_str()?.to_string();
    let eco = osv_ecosystem(pin.ecosystem);
    let current = version_key(&pin.version);
    let mut fixed: Vec<String> = Vec::new();
    for a in vuln
        .get("affected")
        .and_then(|a| a.as_array())
        .into_iter()
        .flatten()
    {
        let pkg = a.get("package");
        let same = pkg.and_then(|p| p.get("name")).and_then(|n| n.as_str())
            == Some(pin.name.as_str())
            && pkg
                .and_then(|p| p.get("ecosystem"))
                .and_then(|e| e.as_str())
                == Some(eco);
        if !same {
            continue;
        }
        for r in a
            .get("ranges")
            .and_then(|r| r.as_array())
            .into_iter()
            .flatten()
        {
            for ev in r
                .get("events")
                .and_then(|e| e.as_array())
                .into_iter()
                .flatten()
            {
                if let Some(f) = ev.get("fixed").and_then(|f| f.as_str()) {
                    if version_key(f) > current {
                        fixed.push(f.to_string());
                    }
                }
            }
        }
    }
    fixed.sort_by_key(|f| version_key(f));
    let rustsec = id.starts_with("RUSTSEC-");
    Some(AdvisoryRecord {
        feed: if rustsec { "rustsec (via osv)" } else { "osv" }.to_string(),
        url: if rustsec {
            format!("https://rustsec.org/advisories/{id}")
        } else {
            format!("https://osv.dev/vulnerability/{id}")
        },
        id,
        ecosystem: pin.ecosystem,
        package: pin.name.clone(),
        bad_versions: vec![pin.version.clone()],
        recommended: fixed.into_iter().next(),
    })
}

/// Drop a non-RustSec id when a RustSec advisory for the same pin lists it as an alias
/// (OSV returns both the GHSA and the RUSTSEC record for one Rust vulnerability).
pub fn osv_dedupe(records: Vec<(AdvisoryRecord, Vec<String>)>) -> Vec<AdvisoryRecord> {
    let keep: Vec<bool> = records
        .iter()
        .map(|(r, _)| {
            r.id.starts_with("RUSTSEC-")
                || !records.iter().any(|(o, aliases)| {
                    o.id.starts_with("RUSTSEC-")
                        && o.package == r.package
                        && o.bad_versions == r.bad_versions
                        && aliases.contains(&r.id)
                })
        })
        .collect();
    records
        .into_iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|((r, _), _)| r)
        .collect()
}

/// Live OSV advisories for `pins`: one querybatch (in chunks of 500), then one record fetch
/// per distinct id. Network only; never writes a lockfile.
pub fn osv_advisories(pins: &[LockPin]) -> Result<Vec<AdvisoryRecord>, QuarantineError> {
    let net = |reason: String| QuarantineError::Read {
        path: OSV_URL.into(),
        reason,
    };
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("apparatus-quarantine/0.1 (+https://github.com/infraax/Aparatus)")
        .build()
        .map_err(|e| net(e.to_string()))?;
    let mut hits = Vec::new();
    for (c, chunk) in pins.chunks(500).enumerate() {
        let text = client
            .post(format!("{OSV_URL}/v1/querybatch"))
            .header("content-type", "application/json")
            .body(osv_batch_body(chunk).to_string())
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.text())
            .map_err(|e| net(e.to_string()))?;
        let resp: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| net(e.to_string()))?;
        hits.extend(
            osv_batch_hits(&resp)
                .into_iter()
                .map(|(i, id)| (c * 500 + i, id)),
        );
    }
    let mut vulns: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for (_, id) in &hits {
        if !vulns.contains_key(id) {
            let text = client
                .get(format!("{OSV_URL}/v1/vulns/{id}"))
                .send()
                .and_then(|r| r.error_for_status())
                .and_then(|r| r.text())
                .map_err(|e| net(e.to_string()))?;
            let v: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| net(e.to_string()))?;
            vulns.insert(id.clone(), v);
        }
    }
    let records = hits
        .iter()
        .filter_map(|(i, id)| {
            let v = &vulns[id];
            let aliases = v
                .get("aliases")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            osv_record(&pins[*i], v).map(|r| (r, aliases))
        })
        .collect();
    Ok(osv_dedupe(records))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(name: &str, version: &str) -> LockPin {
        LockPin {
            ecosystem: Ecosystem::Cargo,
            name: name.into(),
            version: version.into(),
            integrity: "00".into(),
            lockfile: "Cargo.lock".into(),
        }
    }

    #[test]
    fn osv_batch_body_and_hits_round_trip() {
        let pins = [pin("smallvec", "0.6.9"), pin("serde", "1.0.200")];
        let body = osv_batch_body(&pins);
        assert_eq!(body["queries"][0]["package"]["ecosystem"], "crates.io");
        assert_eq!(body["queries"][1]["version"], "1.0.200");
        let resp = serde_json::json!({"results": [
            {"vulns": [{"id": "GHSA-mm7v-vpv8-xfc3"}, {"id": "RUSTSEC-2019-0009"}]},
            {}
        ]});
        assert_eq!(
            osv_batch_hits(&resp),
            vec![
                (0, "GHSA-mm7v-vpv8-xfc3".to_string()),
                (0, "RUSTSEC-2019-0009".to_string())
            ]
        );
    }

    #[test]
    fn osv_record_names_feed_id_and_lowest_fix_and_dedupes_the_ghsa_alias() {
        let p = pin("smallvec", "0.6.9");
        let rustsec = serde_json::json!({
            "id": "RUSTSEC-2019-0009", "aliases": ["CVE-2019-15551", "GHSA-mm7v-vpv8-xfc3"],
            "affected": [{"package": {"name": "smallvec", "ecosystem": "crates.io"},
                          "ranges": [{"events": [{"introduced": "0.6.5"}, {"fixed": "0.6.10"}]},
                                     {"events": [{"introduced": "0"}, {"fixed": "0.6.3"}]}]}]
        });
        let ghsa = serde_json::json!({"id": "GHSA-mm7v-vpv8-xfc3", "aliases": ["RUSTSEC-2019-0009"],
            "affected": [{"package": {"name": "smallvec", "ecosystem": "crates.io"},
                          "ranges": [{"events": [{"fixed": "0.6.10"}]}]}]});
        let r = osv_record(&p, &rustsec).unwrap();
        assert_eq!(r.feed, "rustsec (via osv)");
        assert_eq!(r.id, "RUSTSEC-2019-0009");
        assert_eq!(r.bad_versions, vec!["0.6.9"]);
        assert_eq!(
            r.recommended.as_deref(),
            Some("0.6.10"),
            "0.6.3 is below the pin"
        );
        assert_eq!(r.url, "https://rustsec.org/advisories/RUSTSEC-2019-0009");
        let g = osv_record(&p, &ghsa).unwrap();
        assert_eq!(g.feed, "osv");
        let kept = osv_dedupe(vec![
            (g, vec!["RUSTSEC-2019-0009".into()]),
            (
                r,
                vec!["CVE-2019-15551".into(), "GHSA-mm7v-vpv8-xfc3".into()],
            ),
        ]);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].id, "RUSTSEC-2019-0009");
        assert!(osv_record(
            &p,
            &serde_json::json!({"id": "X", "withdrawn": "2024-01-01T00:00:00Z"})
        )
        .is_none());
    }

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

//! Ingest drop-pipeline: `.apparatus/in/{new,work,ok,refuse,quarantine}/`.
//!
//! Corpus ingest is keep work: a boring pipe. Agents drop files into
//! `in/new/` (write elsewhere, then rename in); only the writer moves a file
//! `new → work`, one at a time, and every file leaves `work/` for exactly one
//! of `ok/`, `refuse/` or `quarantine/` with a `<name>.json` result beside it.
//!
//! - Empty file → C-03 `reject` receipt, `refuse/`.
//! - Digest already admitted → idempotent no-op, `ok/`, no second artefact.
//! - Chain refuses (any rule) → `refuse/` with the rule.
//! - Unreadable file, bad sidecar, CAS/disk failure → `quarantine/` + ticket.
//! - Missing tag → admitted as `C` + ticket "untagged ingest".
//!
//! A file found in `work/` is a leftover from a crash. It is offered once more
//! (guarded by a `.<name>.retry` marker); if that attempt dies too, the next
//! step quarantines it. Bytes are admitted only by a receipt, and orphan CAS
//! bytes are set aside before a retry (see `api::ingest_bytes`).

use crate::api::{classification, execute, ingest_bytes, ingest_reject, ExecOptions, Response};
use crate::api::{Op, Request};
use crate::kernel::{apparatus_dir, Kernel, Submit};
use crate::signal::ensure_ticket;
use anyhow::{anyhow, bail, Context, Result};
use apparatus_crypto::Sha256Digest;
use apparatus_types::rws::EvidenceTag;
use apparatus_types::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

/// Files that may wait in `in/new/` before `ingest_enqueue` answers `busy`.
pub const DEFAULT_CAPACITY: usize = 256;
/// Sidecar suffix: `<file>.rws.json` travels with `<file>`.
pub const SIDECAR: &str = ".rws.json";
/// Ticket title (dampened: one open at a time) while `in/new/` is full.
pub const QUEUE_FULL: &str = "ingest queue full";

const DIRS: [&str; 5] = ["new", "work", "ok", "refuse", "quarantine"];

/// Optional metadata for a dropped file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sidecar {
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub tag: Option<EvidenceTag>,
    #[serde(default)]
    pub classification: Option<String>,
    #[serde(default)]
    pub replica: bool,
}

/// Counts per pipe folder (payload files only; sidecars and dotfiles skipped).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Depth {
    pub new: usize,
    pub work: usize,
    pub quarantine: usize,
}

pub fn in_dir(root: &Path) -> PathBuf {
    apparatus_dir(root).join("in")
}

fn sub(root: &Path, which: &str) -> PathBuf {
    in_dir(root).join(which)
}

pub fn ensure_dirs(root: &Path) -> Result<()> {
    for d in DIRS {
        fs::create_dir_all(sub(root, d)).with_context(|| format!("creating in/{d}"))?;
    }
    Ok(())
}

fn is_payload(name: &str) -> bool {
    !name.starts_with('.') && !name.ends_with(SIDECAR)
}

/// Payload files in `dir`, oldest first (mtime, then name).
fn payloads(dir: &Path) -> Vec<String> {
    let Ok(rd) = fs::read_dir(dir) else {
        return vec![];
    };
    let mut files: Vec<(std::time::SystemTime, String)> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let mtime = e.metadata().and_then(|m| m.modified()).ok()?;
            is_payload(&name).then_some((mtime, name))
        })
        .collect();
    files.sort();
    files.into_iter().map(|(_, n)| n).collect()
}

/// Settled files in `dir`: those with a `<name>.json` result beside them.
fn count_json_results(dir: &Path) -> usize {
    payloads(dir)
        .iter()
        .filter(|n| dir.join(format!("{n}.json")).exists())
        .count()
}

pub fn depth(root: &Path) -> Depth {
    Depth {
        new: payloads(&sub(root, "new")).len(),
        work: payloads(&sub(root, "work")).len(),
        quarantine: count_json_results(&sub(root, "quarantine")),
    }
}

/// A free name in `dir`: `name`, else `name.1`, `name.2`, …
fn free_name(dir: &Path, name: &str) -> String {
    let taken = |n: &str| dir.join(n).exists() || dir.join(format!("{n}.json")).exists();
    if !taken(name) {
        return name.to_string();
    }
    (1..)
        .map(|i| format!("{name}.{i}"))
        .find(|n| !taken(n))
        .expect("some suffix is free")
}

/// Move `from_dir/name` (and its sidecar) into `to_dir`, writing `result` as
/// `<final>.json`. Returns the final name.
fn settle(from_dir: &Path, to_dir: &Path, name: &str, result: serde_json::Value) -> Result<String> {
    let fin = free_name(to_dir, name);
    write_json(&to_dir.join(format!("{fin}.json")), &result)?;
    let side = from_dir.join(format!("{name}{SIDECAR}"));
    if side.exists() {
        fs::rename(&side, to_dir.join(format!("{fin}{SIDECAR}")))?;
    }
    fs::rename(from_dir.join(name), to_dir.join(&fin))
        .with_context(|| format!("moving {name} out of {}", from_dir.display()))?;
    Ok(fin)
}

fn write_json(path: &Path, v: &serde_json::Value) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(v)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn system_request(k: &Kernel, op: Op) -> Request {
    Request {
        principal: Some(k.meta.default_principal.clone()),
        role: None,
        producer_kind: None,
        op,
    }
}

/// File a ticket under the project's default principal; returns a log line.
pub(crate) fn file_ticket(
    k: &mut Kernel,
    title: String,
    subject: Option<ObjectId>,
    body: Option<String>,
) -> String {
    let req = system_request(
        k,
        Op::Ticket {
            title: title.clone(),
            subject,
            body,
        },
    );
    let resp = execute(k, req, ExecOptions::default());
    match (resp.ok, resp.id) {
        (true, Some(id)) => format!("ticket {id} {title}"),
        _ => format!(
            "could not file ticket '{title}': {}",
            resp.error.unwrap_or_default()
        ),
    }
}

/// Process at most one file. `Ok(None)`: nothing to do.
pub fn step(k: &mut Kernel) -> Result<Option<Vec<String>>> {
    let root = k.root().to_path_buf();
    ensure_dirs(&root)?;
    let work = sub(&root, "work");
    if let Some(name) = payloads(&work).into_iter().next() {
        // Leftover from a crash: one more try, then quarantine.
        let marker = work.join(format!(".{name}.retry"));
        if marker.exists() {
            let lines = quarantine(k, &name, "processing died twice in work/".into());
            fs::remove_file(&marker).ok();
            return Ok(Some(lines));
        }
        fs::write(&marker, b"")?;
        let mut lines = vec![format!("re-offering {name} left in work/")];
        lines.extend(process(k, &name));
        fs::remove_file(&marker).ok();
        return Ok(Some(lines));
    }
    let new = sub(&root, "new");
    let Some(name) = payloads(&new).into_iter().next() else {
        return Ok(None);
    };
    // Sidecar first, so a crash between the two moves keeps them together in work/.
    let side = new.join(format!("{name}{SIDECAR}"));
    if side.exists() {
        fs::rename(&side, work.join(format!("{name}{SIDECAR}")))?;
    }
    fs::rename(new.join(&name), work.join(&name))
        .with_context(|| format!("moving {name} into work/"))?;
    Ok(Some(process(k, &name)))
}

/// Recover leftovers and empty `in/new/`. Used by `rws ingest-dir` and `ingest_drain`.
pub fn drain(k: &mut Kernel) -> Result<Vec<String>> {
    let mut out = vec![];
    while let Some(lines) = step(k)? {
        out.extend(lines);
    }
    let d = depth(k.root());
    out.push(format!(
        "pipe idle: new={} work={} quarantine={}",
        d.new, d.work, d.quarantine
    ));
    Ok(out)
}

/// Decide one file sitting in `work/`. Never returns an error: every failure
/// ends in `quarantine/` (or, if even that fails, stays in `work/` for the retry).
fn process(k: &mut Kernel, name: &str) -> Vec<String> {
    let root = k.root().to_path_buf();
    let work = sub(&root, "work");
    let bytes = match fs::read(work.join(name)) {
        Ok(b) => b,
        Err(e) => return quarantine(k, name, format!("unreadable: {e}")),
    };
    let side_path = work.join(format!("{name}{SIDECAR}"));
    let sidecar: Sidecar = if side_path.exists() {
        match fs::read(&side_path)
            .map_err(anyhow::Error::from)
            .and_then(|b| serde_json::from_slice(&b).map_err(anyhow::Error::from))
        {
            Ok(s) => s,
            Err(e) => return quarantine(k, name, format!("bad sidecar {name}{SIDECAR}: {e}")),
        }
    } else {
        Sidecar::default()
    };
    match decide(k, name, &bytes, &sidecar) {
        Ok(lines) => lines,
        Err(e) => quarantine(k, name, format!("{e:#}")),
    }
}

fn decide(k: &mut Kernel, name: &str, bytes: &[u8], sc: &Sidecar) -> Result<Vec<String>> {
    let root = k.root().to_path_buf();
    let work = sub(&root, "work");
    let locator = format!("in/{name}");
    let purpose = sc.purpose.clone().unwrap_or_else(|| "corpus".into());
    let tag = sc.tag.unwrap_or(EvidenceTag::C);
    let (signer, _) = k.principal(Some(&k.meta.default_principal.clone()));

    if bytes.is_empty() {
        let reason = "empty file".to_string();
        let receipt = match ingest_reject(k, signer, bytes, locator, purpose, tag, reason.clone())?
        {
            Submit::Accepted { id, .. } => Some(id),
            Submit::Refused(_) => None,
        };
        k.hook().after_refuse("C-03", name);
        let fin = settle(
            &work,
            &sub(&root, "refuse"),
            name,
            json!({ "status": "refused", "rule": "C-03", "reason": reason, "receipt": receipt }),
        )?;
        return Ok(vec![format!("refuse {fin} C-03 {reason}")]);
    }

    let digest = Sha256Digest::compute(bytes);
    let hex = format!("{digest:x}");
    if k.state.has_digest(&hex) {
        let artifact = k
            .state
            .artifacts()
            .find(|(_, a)| a.digest == hex)
            .map(|(id, _)| *id);
        let fin = settle(
            &work,
            &sub(&root, "ok"),
            name,
            json!({ "status": "already_admitted", "artifact_id": artifact, "digest": hex }),
        )?;
        return Ok(vec![format!("ok {fin} already admitted sha256 {hex}")]);
    }

    let class = classification(sc.classification.as_deref())?;
    let (artifact_id, _, result) = ingest_bytes(
        k,
        signer,
        None,
        bytes.to_vec(),
        locator,
        purpose,
        tag,
        class,
        sc.replica,
    )?;
    match result {
        Submit::Refused(r) => {
            let fin = settle(
                &work,
                &sub(&root, "refuse"),
                name,
                json!({
                    "status": "refused",
                    "rule": r.rejection.rule,
                    "reason": r.rejection.to_string(),
                    "refusal_event": r.recorded.map(|(id, _)| id),
                }),
            )?;
            Ok(vec![format!("refuse {fin} {}", r.rejection)])
        }
        Submit::Accepted { id, hash, .. } => {
            let fin = settle(
                &work,
                &sub(&root, "ok"),
                name,
                json!({
                    "status": "admitted",
                    "receipt": id,
                    "hash": format!("{hash:x}"),
                    "artifact_id": artifact_id,
                    "digest": hex,
                    "tag": tag,
                }),
            )?;
            let mut lines = vec![format!("ok {fin} {id} artifact {artifact_id} sha256 {hex}")];
            if sc.tag.is_none() {
                lines.push(file_ticket(
                    k,
                    format!("untagged ingest: {fin}"),
                    Some(id),
                    Some("admitted as C; correct the tag if it is stronger evidence".into()),
                ));
            }
            Ok(lines)
        }
    }
}

/// Park a file from `work/` in `quarantine/` and file a ticket.
fn quarantine(k: &mut Kernel, name: &str, reason: String) -> Vec<String> {
    let root = k.root().to_path_buf();
    let mut lines = vec![];
    match settle(
        &sub(&root, "work"),
        &sub(&root, "quarantine"),
        name,
        json!({ "status": "quarantined", "reason": reason }),
    ) {
        Ok(fin) => lines.push(format!("quarantine {fin} {reason}")),
        Err(e) => lines.push(format!("could not quarantine {name}: {e:#}; left in work/")),
    }
    k.hook().after_refuse("quarantine", name);
    lines.push(file_ticket(
        k,
        format!("ingest quarantined: {name}"),
        None,
        Some(reason),
    ));
    lines
}

/// Validate a client-chosen file name: one plain component.
fn clean_name(name: &str) -> Result<String> {
    let n = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("name must be a file name"))?;
    if n.is_empty() || !is_payload(n) || n.ends_with(".json.tmp") {
        bail!("name '{n}' is reserved (dotfile or {SIDECAR})");
    }
    Ok(n.to_string())
}

/// `ingest_enqueue`: place bytes in `in/new/` (atomically) for the pipe.
/// A full queue answers `busy` and keeps one open "ingest queue full" ticket.
pub fn enqueue(
    k: &mut Kernel,
    name: &str,
    bytes: &[u8],
    sidecar: Option<Sidecar>,
    capacity: usize,
) -> Result<Response> {
    let root = k.root().to_path_buf();
    ensure_dirs(&root)?;
    let name = clean_name(name)?;
    let d = depth(&root);
    if d.new >= capacity {
        let line = ensure_ticket(k, QUEUE_FULL, Some(format!("{} files waiting", d.new)));
        let mut r = Response::busy(format!(
            "ingest queue full ({} waiting, capacity {capacity}); retry later — see `rws signals`",
            d.new
        ));
        r.lines = line.into_iter().collect();
        return Ok(r);
    }
    let new = sub(&root, "new");
    let fin = free_name(&new, &name);
    if let Some(sc) = sidecar {
        let tmp = new.join(format!(".{fin}{SIDECAR}.tmp"));
        fs::write(&tmp, serde_json::to_vec_pretty(&sc)?)?;
        fs::rename(&tmp, new.join(format!("{fin}{SIDECAR}")))?;
    }
    let tmp = new.join(format!(".{fin}.tmp"));
    fs::write(&tmp, bytes).with_context(|| format!("writing in/new/{fin}"))?;
    fs::rename(&tmp, new.join(&fin))?;
    let mut r = Response::ok(vec![format!("queued in/new/{fin}")]);
    r.data = Some(json!({ "queued": fin, "depth": d.new + 1 }));
    Ok(r)
}

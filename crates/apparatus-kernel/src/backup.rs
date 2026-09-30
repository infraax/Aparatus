//! Replica backup and restore.
//!
//! A backup is a replica (A-08): it confers no rights and is never the source
//! of truth. It is a consistent copy taken by the one writer (the daemon's
//! writer actor, or a local process holding `.apparatus/LOCK`), so no receipt
//! can land half-way. `MANIFEST.json` is written last: a directory without it
//! is an interrupted backup and `restore` refuses it.
//!
//! Restore is ingest-like: copy into a staging project, verify the chain,
//! replay every rule and every CAS digest there, and only then move the files
//! into an empty `.apparatus/`. It never runs over a live HEAD.

use crate::api::check;
use crate::kernel::{apparatus_dir, Kernel};
use anyhow::{anyhow, bail, Context, Result};
use apparatus_ledger::{FileLedger, LOCK_TIMEOUT};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const MANIFEST: &str = "MANIFEST.json";
/// What a backup holds, relative to `.apparatus/`. HEAD is moved in last on restore.
const FILES: [&str; 3] = ["project.json", "ledger.jsonl", "HEAD"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Always true: a backup is a replica, never authority (A-08).
    pub replica: bool,
    pub created_ms: u64,
    pub project_id: String,
    pub head_id: Option<String>,
    pub head_hash: Option<String>,
    pub receipts: usize,
    /// Files copied (ledger, HEAD, project.json, CAS blobs and pointers).
    pub files: usize,
    pub source: String,
    pub note: String,
}

fn is_empty_dir(p: &Path) -> Result<bool> {
    Ok(fs::read_dir(p)?.next().is_none())
}

/// Copy `src` into `dst` recursively, skipping temp files and `orphans/`.
fn copy_tree(src: &Path, dst: &Path) -> Result<usize> {
    fs::create_dir_all(dst)?;
    let mut n = 0;
    for e in fs::read_dir(src)? {
        let e = e?;
        let name = e.file_name();
        let name_s = name.to_string_lossy();
        if name_s == "orphans" || name_s.contains(".tmp") {
            continue;
        }
        let ty = e.file_type()?;
        if ty.is_dir() {
            n += copy_tree(&e.path(), &dst.join(&name))?;
        } else if ty.is_file() {
            fs::copy(e.path(), dst.join(&name))
                .with_context(|| format!("copying {}", e.path().display()))?;
            n += 1;
        }
    }
    Ok(n)
}

/// Copy the chain, CAS and project.json of `from` (an `.apparatus/`) into `to`.
fn copy_project(from: &Path, to: &Path) -> Result<usize> {
    fs::create_dir_all(to)?;
    let mut n = 0;
    for f in FILES {
        let src = from.join(f);
        if src.exists() {
            fs::copy(&src, to.join(f)).with_context(|| format!("copying {f}"))?;
            n += 1;
        }
    }
    let cas = from.join("cas");
    if cas.exists() {
        n += copy_tree(&cas, &to.join("cas"))?;
    }
    Ok(n)
}

/// Back up the project `k` holds into `to` (absent or empty).
pub fn backup(k: &Kernel, to: &Path) -> Result<Manifest> {
    k.verify_disk()?;
    if to.exists() && !is_empty_dir(to)? {
        bail!(
            "{} is not empty; a backup goes into a new directory",
            to.display()
        );
    }
    let files = copy_project(&apparatus_dir(k.root()), to)?;
    // The copy must be the chain we hold, byte for byte up to HEAD.
    let copied = FileLedger::new(to)
        .verify_chain()
        .map_err(|e| anyhow!("backup copy does not verify: {e}"))?;
    let copied_head = copied.last().map(|e| (e.receipt.receipt_id(), e.hash));
    if copied_head != k.head() {
        bail!("backup copy head differs from the live head");
    }
    let head = k.head();
    let manifest = Manifest {
        replica: true,
        created_ms: k.now(),
        project_id: k.meta.project_id.to_string(),
        head_id: head.map(|(id, _)| id.to_string()),
        head_hash: head.map(|(_, h)| format!("{h:x}")),
        receipts: k.entries.len(),
        files,
        source: k.root().display().to_string(),
        note: "Replica (A-08): no rights can be derived from this copy; source of truth is the live chain.".into(),
    };
    let tmp = to.join(format!("{MANIFEST}.tmp"));
    fs::write(&tmp, serde_json::to_vec_pretty(&manifest)?)?;
    fs::rename(&tmp, to.join(MANIFEST))?;
    k.hook().after_backup(to);
    Ok(manifest)
}

pub fn manifest_lines(m: &Manifest, to: &Path) -> Vec<String> {
    vec![
        format!(
            "backup {} ({} files, {} receipts)",
            to.display(),
            m.files,
            m.receipts
        ),
        format!(
            "head {} {}",
            m.head_id.as_deref().unwrap_or("-"),
            m.head_hash.as_deref().unwrap_or("-")
        ),
        "replica: true".into(),
    ]
}

/// Restore a backup into `root`, whose `.apparatus/` must be absent or empty.
pub fn restore(from: &Path, root: &Path) -> Result<Vec<String>> {
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(from.join(MANIFEST)).with_context(|| {
            format!(
                "{} has no {MANIFEST}: not a finished backup",
                from.display()
            )
        })?)
        .context("reading MANIFEST.json")?;
    if !manifest.replica {
        bail!("{MANIFEST} is not marked replica: true");
    }
    let dest = apparatus_dir(root);
    if dest.exists() && !is_empty_dir(&dest)? {
        bail!(
            "{} is not empty; restore only into an empty project, never over a live HEAD",
            dest.display()
        );
    }
    let stage = root.join(".apparatus-restore");
    if stage.exists() {
        bail!(
            "{} exists (an earlier restore was interrupted); remove it first",
            stage.display()
        );
    }
    let staged = apparatus_dir(&stage);
    let result = (|| -> Result<Vec<String>> {
        copy_project(from, &staged)?;
        // Full verification in the staging project: chain, rules, CAS.
        let k = Kernel::open(&stage).context("restored copy does not open")?;
        let head = k.head();
        let head_id = head.map(|(id, _)| id.to_string());
        let head_hash = head.map(|(_, h)| format!("{h:x}"));
        if head_id != manifest.head_id || head_hash != manifest.head_hash {
            bail!("restored head does not match the manifest");
        }
        let report = check(&k);
        if !report.ok {
            bail!("restored copy fails check: {}", report.lines.join("; "));
        }
        let receipts = k.entries.len();
        drop(k);
        fs::remove_file(staged.join("LOCK")).ok();

        // Move into place under the destination's lock; HEAD last.
        let _lock = FileLedger::new(&dest)
            .lock(LOCK_TIMEOUT)
            .map_err(|e| anyhow!("{e}"))?;
        for f in ["cas", "project.json", "ledger.jsonl", "HEAD"] {
            let src = staged.join(f);
            if src.exists() {
                fs::rename(&src, dest.join(f)).with_context(|| format!("moving {f} into place"))?;
            }
        }
        Ok(vec![
            format!("restored {} receipts into {}", receipts, dest.display()),
            format!(
                "head {} {}",
                head_id.as_deref().unwrap_or("-"),
                head_hash.as_deref().unwrap_or("-")
            ),
        ])
    })();
    fs::remove_dir_all(&stage).ok();
    result
}

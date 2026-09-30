//! Local RWS kernel: one project directory, one `.apparatus/` chain.
//!
//! Every write goes: build payload → `State::check` → side effect (CAS) →
//! `FileLedger::append_receipt` → `State::apply`. A refused write is recorded
//! as an `illegal_transition_rejected` event on the same chain.
//!
//! A `Kernel` holds the exclusive `.apparatus/LOCK` from before it reads the
//! chain until it is dropped, so two CLI processes never append against the
//! same HEAD.

use anyhow::{anyhow, bail, Context, Result};
use apparatus_artifacts::FsArtifactStore;
use apparatus_crypto::Sha256Digest;
use apparatus_ledger::{ChainEntry, FileLedger, LedgerLock, Receipt, LOCK_TIMEOUT};
use apparatus_schema::rws::{decode_payload, Rejection, State};
use apparatus_time::{Clock, SystemClock};
use apparatus_types::rws::*;
use apparatus_types::{
    ArtifactId, Classification, ObjectHeader, ObjectId, PrincipalId, ProjectId, Provenance,
    ReceiptId,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DAY_MS: u64 = 86_400_000;

/// Contents of `.apparatus/project.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub project_id: ProjectId,
    pub name: String,
    pub solo: bool,
    pub default_principal: String,
}

/// Result of a refused write.
#[derive(Debug)]
pub struct Refused {
    pub rejection: Rejection,
    /// The refusal event, if it could be recorded.
    pub recorded: Option<(ObjectId, Sha256Digest)>,
}

pub enum Submit {
    Accepted {
        id: ObjectId,
        kind: &'static str,
        hash: Sha256Digest,
    },
    Refused(Refused),
}

/// One receipt of a `submit_batch`.
pub struct BatchPart {
    pub signer: PrincipalId,
    pub role: Role,
    pub subject: Option<ObjectId>,
    pub evidence: Vec<ArtifactId>,
    pub body: Body,
}

pub struct Kernel {
    pub meta: ProjectMeta,
    pub state: State,
    pub entries: Vec<ChainEntry>,
    ledger: FileLedger,
    pub cas: FsArtifactStore,
    clock: SystemClock,
    root: PathBuf,
    _lock: LedgerLock,
}

pub fn apparatus_dir(root: &Path) -> PathBuf {
    root.join(".apparatus")
}

pub fn buffer_path(root: &Path) -> PathBuf {
    root.join("rws").join("receipts.jsonl")
}

impl Kernel {
    /// Take the writer lock for `root`.
    fn lock(root: &Path) -> Result<LedgerLock> {
        FileLedger::new(apparatus_dir(root))
            .lock(LOCK_TIMEOUT)
            .map_err(|e| anyhow!("another apparatus process is writing this project: {e}"))
    }

    /// Open an initialised project, verifying the chain and replaying every receipt.
    pub fn open(root: &Path) -> Result<Self> {
        let lock = Self::lock(root)?;
        Self::open_locked(root, lock)
    }

    fn open_locked(root: &Path, lock: LedgerLock) -> Result<Self> {
        let dir = apparatus_dir(root);
        let (meta, entries, state) = Self::load(root)?;
        Ok(Self {
            meta,
            state,
            entries,
            ledger: FileLedger::new(&dir),
            cas: FsArtifactStore::new(dir.join("cas")),
            clock: SystemClock,
            root: root.to_path_buf(),
            _lock: lock,
        })
    }

    /// Read project.json, verify the chain on disk and replay it. No locking.
    fn load(root: &Path) -> Result<(ProjectMeta, Vec<ChainEntry>, State)> {
        let dir = apparatus_dir(root);
        let meta_path = dir.join("project.json");
        if !meta_path.exists() {
            bail!(
                "no .apparatus/project.json in {} — run `apparatus rws init`",
                root.display()
            );
        }
        let meta: ProjectMeta = serde_json::from_slice(&fs::read(&meta_path)?)
            .context("reading .apparatus/project.json")?;
        let entries = FileLedger::new(&dir)
            .verify_chain()
            .map_err(|e| anyhow!("integrity: {e}"))?;
        let mut state = State::new();
        for (i, entry) in entries.iter().enumerate() {
            let payload = decode_payload(&entry.receipt.operation_payload)
                .map_err(|r| anyhow!("integrity: line {}: {r}", i + 1))?;
            if entry.receipt.header.project_id != meta.project_id {
                bail!(
                    "integrity: line {}: receipt belongs to another project",
                    i + 1
                );
            }
            state
                .admit(&entry.receipt.header, &payload)
                .map_err(|r| anyhow!("integrity: line {} replays as illegal: {r}", i + 1))?;
        }
        Ok((meta, entries, state))
    }

    /// Re-read the chain from disk, keeping the lock (used after a handler panic).
    pub fn reload(&mut self) -> Result<()> {
        let (meta, entries, state) = Self::load(&self.root)?;
        self.meta = meta;
        self.entries = entries;
        self.state = state;
        Ok(())
    }

    /// Verify the on-disk chain and that it is exactly what this kernel holds.
    pub fn verify_disk(&self) -> Result<()> {
        let disk = self
            .ledger
            .verify_chain()
            .map_err(|e| anyhow!("integrity: {e}"))?;
        let disk_head = disk.last().map(|e| (e.receipt.receipt_id(), e.hash));
        if disk.len() != self.entries.len() || disk_head != self.head() {
            bail!("integrity: on-disk chain differs from the loaded chain");
        }
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create `.apparatus/` with an empty chain.
    fn create(root: &Path, meta: ProjectMeta) -> Result<Self> {
        let dir = apparatus_dir(root);
        let lock = Self::lock(root)?;
        if dir.join("project.json").exists() {
            bail!("{} is already initialised", root.display());
        }
        fs::create_dir_all(dir.join("cas"))?;
        fs::write(dir.join("project.json"), serde_json::to_vec_pretty(&meta)?)?;
        Self::open_locked(root, lock)
    }

    /// `rws init`: bootstrap, or import an existing `rws/receipts.jsonl` buffer once.
    pub fn init(
        root: &Path,
        name: Option<String>,
        principal: String,
        solo: bool,
    ) -> Result<Vec<String>> {
        let name = name.unwrap_or_else(|| {
            root.canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "project".into())
        });
        let buffer = buffer_path(root);
        if buffer.exists() {
            let mut lines = vec![format!("found import buffer {}", buffer.display())];
            lines.extend(Self::import_jsonl(root, &buffer, Some(name), solo)?);
            return Ok(lines);
        }
        let meta = ProjectMeta {
            project_id: ProjectId::new(),
            name,
            solo,
            default_principal: principal.clone(),
        };
        let mut k = Self::create(root, meta)?;
        let owner = PrincipalId::new();
        let mut roles = vec![Role::Continuity];
        if solo {
            // Solo also holds the handover mandate so G4 needs no extra bind (M2).
            roles.extend([Role::Policy, Role::Execution, Role::MandateHolder]);
        }
        let mut out = Vec::new();
        for role in roles {
            let body = Body::RoleBind(RoleBind {
                bound_role: role,
                principal: Some(owner),
                principal_name: Some(principal.clone()),
                principal_kind: Some(PrincipalKind::Human),
            });
            // The bootstrap bind is always self-certified (R-07).
            match k.submit_as(
                owner,
                Role::Continuity,
                None,
                vec![],
                true,
                Classification::Internal,
                body,
                |_| Ok(()),
            )? {
                Submit::Accepted { id, kind, hash } => out.push(line(id, kind, &hash)),
                Submit::Refused(r) => bail!("bootstrap refused: {}", r.rejection),
            }
        }
        Ok(out)
    }

    /// Import a receipt buffer into an empty chain, then freeze the buffer.
    pub fn import_jsonl(
        root: &Path,
        path: &Path,
        name: Option<String>,
        solo: bool,
    ) -> Result<Vec<String>> {
        let text =
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let mut receipts = Vec::new();
        for (i, l) in text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
        {
            let r: Receipt = serde_json::from_str(l)
                .with_context(|| format!("line {}: not a receipt", i + 1))?;
            receipts.push((i + 1, r));
        }
        let Some((_, first)) = receipts.first() else {
            bail!("{} is empty", path.display());
        };
        let dir = apparatus_dir(root);
        let mut k = if dir.join("project.json").exists() {
            Self::open(root)?
        } else {
            let first_name = decode_payload(&first.operation_payload)
                .ok()
                .and_then(|p| match p.body {
                    Body::RoleBind(b) => b.principal_name,
                    _ => None,
                })
                .unwrap_or_else(|| "owner".into());
            Self::create(
                root,
                ProjectMeta {
                    project_id: first.header.project_id,
                    name: name.unwrap_or_else(|| "project".into()),
                    solo,
                    default_principal: first_name,
                },
            )?
        };
        if k.head().is_some() {
            bail!("refused: the ledger already has a HEAD; a buffer is imported only into an empty chain (X-11)");
        }

        // Dry run on a copy of the state: all lines or nothing.
        let mut dry = k.state.clone();
        let mut prev: Option<(ReceiptId, Sha256Digest)> = None;
        let mut chained = Vec::new();
        for (line_no, mut r) in receipts {
            match (r.previous_receipt_id, r.previous_hash) {
                (None, None) => {
                    r.previous_receipt_id = prev.map(|p| p.0);
                    r.previous_hash = prev.map(|p| p.1);
                }
                (id, hash) => {
                    if id.zip(hash) != prev {
                        bail!("refused: line {line_no}: previous link does not follow the buffer order");
                    }
                }
            }
            if r.header.project_id != k.meta.project_id {
                bail!("refused: line {line_no}: receipt belongs to another project");
            }
            let payload = decode_payload(&r.operation_payload)
                .map_err(|e| anyhow!("refused: line {line_no}: {e}"))?;
            if let Body::Ingest(i) = &payload.body {
                if matches!(
                    i.outcome,
                    IngestOutcome::Admit | IngestOutcome::AdmitRestricted
                ) {
                    let digest = Sha256Digest::from_hex(&i.digest)
                        .map_err(|e| anyhow!("line {line_no}: {e}"))?;
                    if !k.cas.verify(&digest)? {
                        bail!("refused: line {line_no}: C-03: admitted bytes are not in .apparatus/cas; import them with `rws ingest` after the buffer");
                    }
                }
            }
            dry.admit(&r.header, &payload)
                .map_err(|e| anyhow!("refused: line {line_no}: {e}"))?;
            let hash = r.compute_hash()?;
            prev = Some((r.receipt_id(), hash));
            chained.push((r, payload));
        }

        let mut out = Vec::new();
        for (r, payload) in chained {
            let hash = k.ledger.append_receipt(&r)?;
            k.state.apply(&r.header, &payload);
            out.push(line(r.header.id, payload.body.kind(), &hash));
            k.entries.push(ChainEntry { receipt: r, hash });
        }
        let frozen = path.with_extension("jsonl.imported");
        fs::rename(path, &frozen)?;
        out.push(format!("buffer frozen as {}", frozen.display()));
        Ok(out)
    }

    pub fn head(&self) -> Option<(ReceiptId, Sha256Digest)> {
        self.entries
            .last()
            .map(|e| (e.receipt.receipt_id(), e.hash))
    }

    pub fn now(&self) -> u64 {
        self.clock.now_ms()
    }

    /// Resolve a principal by name; unknown names get a fresh (unbound) id.
    pub fn principal(&self, name: Option<&str>) -> (PrincipalId, String) {
        let name = name.unwrap_or(&self.meta.default_principal).to_string();
        match self.state.principal_by_name(&name) {
            Some((id, _)) => (id, name),
            None => (PrincipalId::new(), name),
        }
    }

    /// The role to sign under: explicit override, else the first preferred role held.
    pub fn pick_role(
        &self,
        principal: PrincipalId,
        explicit: Option<Role>,
        preferred: &[Role],
    ) -> Role {
        if let Some(r) = explicit {
            return r;
        }
        preferred
            .iter()
            .copied()
            .find(|r| self.state.holds(principal, *r))
            .unwrap_or(preferred[0])
    }

    fn header(
        &self,
        id: ObjectId,
        signer: PrincipalId,
        classification: Classification,
    ) -> ObjectHeader {
        ObjectHeader {
            id,
            created_at: self.now(),
            project_id: self.meta.project_id,
            classification,
            provenance: Some(Provenance {
                source: "apparatus-cli".into(),
                creator: signer,
            }),
            correlation_id: None,
        }
    }

    /// Submit with the project's default self-certification flag.
    pub fn submit(
        &mut self,
        signer: PrincipalId,
        role: Role,
        subject: Option<ObjectId>,
        evidence: Vec<ArtifactId>,
        body: Body,
    ) -> Result<Submit> {
        let solo = self.meta.solo;
        self.submit_as(
            signer,
            role,
            subject,
            evidence,
            solo,
            Classification::Internal,
            body,
            |_| Ok(()),
        )
    }

    /// Check, run `effect` (e.g. write CAS bytes), append, fold.
    #[allow(clippy::too_many_arguments)]
    pub fn submit_as(
        &mut self,
        signer: PrincipalId,
        role: Role,
        subject: Option<ObjectId>,
        evidence: Vec<ArtifactId>,
        self_certified: bool,
        classification: Classification,
        body: Body,
        effect: impl FnOnce(&Self) -> Result<()>,
    ) -> Result<Submit> {
        let id = ObjectId::new_v7();
        let header = self.header(id, signer, classification);
        let payload = Payload {
            rws: RWS_VERSION.into(),
            subject_id: subject,
            signer,
            role,
            authority_ref: None,
            evidence,
            self_certified,
            body,
        };
        self.submit_payload(header, payload, effect)
    }

    /// Submit several receipts as one unit: each is checked against the state
    /// as it would be after the previous ones. All are appended, or none is and
    /// the first refusal is recorded. The writer lock is held throughout.
    pub fn submit_batch(&mut self, parts: Vec<BatchPart>) -> Result<Vec<Submit>> {
        let self_certified = self.meta.solo;
        let prepared: Vec<(ObjectHeader, Payload)> = parts
            .into_iter()
            .map(|part| {
                let header = self.header(ObjectId::new_v7(), part.signer, Classification::Internal);
                let payload = Payload {
                    rws: RWS_VERSION.into(),
                    subject_id: part.subject,
                    signer: part.signer,
                    role: part.role,
                    authority_ref: None,
                    evidence: part.evidence,
                    self_certified,
                    body: part.body,
                };
                (header, payload)
            })
            .collect();
        let mut dry = self.state.clone();
        for (header, payload) in &prepared {
            if let Err(rejection) = dry.check(header, payload) {
                let recorded = self.record_refusal(payload, &rejection)?;
                return Ok(vec![Submit::Refused(Refused {
                    rejection,
                    recorded,
                })]);
            }
            dry.apply(header, payload);
        }
        let mut out = Vec::new();
        for (header, payload) in prepared {
            let id = header.id;
            let hash = self.append(header, &payload)?;
            out.push(Submit::Accepted {
                id,
                kind: payload.body.kind(),
                hash,
            });
        }
        Ok(out)
    }

    pub fn submit_payload(
        &mut self,
        header: ObjectHeader,
        payload: Payload,
        effect: impl FnOnce(&Self) -> Result<()>,
    ) -> Result<Submit> {
        if let Err(rejection) = self.state.check(&header, &payload) {
            let recorded = self.record_refusal(&payload, &rejection)?;
            return Ok(Submit::Refused(Refused {
                rejection,
                recorded,
            }));
        }
        effect(self)?;
        let hash = self.append(header, &payload)?;
        Ok(Submit::Accepted {
            id: self
                .entries
                .last()
                .expect("just appended")
                .receipt
                .header
                .id,
            kind: payload.body.kind(),
            hash,
        })
    }

    fn append(&mut self, header: ObjectHeader, payload: &Payload) -> Result<Sha256Digest> {
        let head = self.head();
        let receipt = Receipt {
            header,
            previous_receipt_id: head.map(|h| h.0),
            previous_hash: head.map(|h| h.1),
            operation_payload: serde_json::to_value(payload)?,
        };
        let hash = self.ledger.append_receipt(&receipt)?;
        self.state.apply(&receipt.header, payload);
        self.entries.push(ChainEntry { receipt, hash });
        Ok(hash)
    }

    /// Record a refusal as `illegal_transition_rejected` (K-12, B-14, M-11, X-05).
    fn record_refusal(
        &mut self,
        attempted: &Payload,
        rejection: &Rejection,
    ) -> Result<Option<(ObjectId, Sha256Digest)>> {
        if self.state.is_empty() {
            return Ok(None);
        }
        let subject = attempted.subject_id.filter(|s| self.state.exists(*s));
        let attempted_json = serde_json::to_string(attempted)?;
        let body = Body::Event(Event {
            event: EventKind::IllegalTransitionRejected,
            impact: Some(format!("{} refused: {}", attempted.body.kind(), rejection)),
            next_decision: Some(NextDecision {
                role: EventKind::IllegalTransitionRejected.default_next_role(),
                by: self.now() + 7 * DAY_MS,
                options: vec!["fix and resubmit".into(), "accept refusal".into()],
            }),
            refs: vec![],
            detail: Some(format!(
                "rule {}; attempted {attempted_json}",
                rejection.rule
            )),
            ticket: None,
        });
        let id = ObjectId::new_v7();
        let header = self.header(id, attempted.signer, Classification::Internal);
        let payload = Payload {
            rws: RWS_VERSION.into(),
            subject_id: subject,
            signer: attempted.signer,
            role: attempted.role,
            authority_ref: None,
            evidence: vec![],
            self_certified: attempted.self_certified,
            body,
        };
        if self.state.check(&header, &payload).is_err() {
            return Ok(None);
        }
        let hash = self.append(header, &payload)?;
        Ok(Some((id, hash)))
    }

    /// Find a receipt by id.
    pub fn receipt(&self, id: ObjectId) -> Option<&ChainEntry> {
        self.entries.iter().find(|e| e.receipt.header.id == id)
    }
}

pub fn line(id: ObjectId, kind: &str, hash: &Sha256Digest) -> String {
    format!("{id} {kind} {hash:x}")
}

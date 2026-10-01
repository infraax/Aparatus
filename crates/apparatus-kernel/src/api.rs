//! Request/response protocol shared by the CLI (local) and `apparatusd` (RPC).
//!
//! One `Request` per JSON line, one `Response` per JSON line. `execute` runs a
//! request against a `Kernel`; the CLI calls it in-process, the daemon calls it
//! from its single writer actor. Same code, same rules, one chain.

use crate::kernel::{buffer_path, line, BatchPart, Kernel, Submit, DAY_MS};
use anyhow::{anyhow, bail, Context, Result};
use apparatus_crypto::{MlDsa65Stub, Sha256Digest, SigningKey};
use apparatus_schema::envelope;
use apparatus_schema::rws::{role_rank, ItemState};
use apparatus_types::rws::*;
use apparatus_types::{ArtifactId, Classification, ObjectId, PrincipalId};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A client request: who (principal, optional role and producer kind) plus one operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Principal name to sign as. Default: the project's default principal.
    #[serde(default)]
    pub principal: Option<String>,
    /// Role override. Default depends on the operation.
    #[serde(default)]
    pub role: Option<Role>,
    /// Declared kind of the calling principal; must match its binding if given.
    #[serde(default)]
    pub producer_kind: Option<PrincipalKind>,
    #[serde(flatten)]
    pub op: Op,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListWhat {
    Items,
    Events,
    Tickets,
}

/// Operations. Reads: `head`, `check`, `show`, `list`, `tickets`. Everything else writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    Head,
    Check,
    Show {
        id: ObjectId,
    },
    List {
        what: ListWhat,
    },
    Tickets {
        #[serde(default)]
        open: bool,
    },
    RoleBind {
        bound_role: Role,
        target: String,
        #[serde(default = "human")]
        kind: PrincipalKind,
    },
    Ingest {
        /// Absolute path readable by the process that executes the request.
        #[serde(default)]
        path: Option<String>,
        /// Inline UTF-8 content (for clients without shared files).
        #[serde(default)]
        content: Option<String>,
        /// Locator recorded for inline content.
        #[serde(default)]
        name: Option<String>,
        purpose: String,
        tag: EvidenceTag,
        #[serde(default)]
        classification: Option<String>,
        #[serde(default)]
        replica: bool,
    },
    QueueKeep {
        #[serde(default)]
        asset: Option<String>,
        #[serde(default)]
        item: Option<ObjectId>,
        #[serde(default)]
        to: Option<KeepState>,
        #[serde(default)]
        lead: Option<String>,
        #[serde(default)]
        adds_function: bool,
        #[serde(default)]
        urgent_safety: bool,
        #[serde(default)]
        evidence: Vec<ArtifactId>,
        #[serde(default)]
        by: Option<ObjectId>,
        #[serde(default)]
        reason: Option<String>,
    },
    QueueBuild {
        asset: String,
        #[serde(default)]
        lead: Option<String>,
        #[serde(default)]
        evidence: Vec<ArtifactId>,
    },
    Gate {
        item: ObjectId,
        gate: GateKind,
        outcome: GateOutcome,
        #[serde(default)]
        evidence: Vec<ArtifactId>,
        #[serde(default)]
        envelope: Option<ObjectId>,
        #[serde(default)]
        required: Option<u64>,
        #[serde(default)]
        available: Option<u64>,
        #[serde(default)]
        discharge: Option<String>,
        #[serde(default)]
        agreement: Option<ObjectId>,
        #[serde(default)]
        keep_envelope: Option<ObjectId>,
    },
    Means {
        envelope: ObjectId,
        from: MeansState,
        to: MeansTarget,
        amount: u64,
        #[serde(default)]
        unit: Option<String>,
        #[serde(default)]
        item: Option<ObjectId>,
        #[serde(default)]
        gate: Option<ObjectId>,
        #[serde(default)]
        agreement: Option<ObjectId>,
    },
    Event {
        kind: EventKind,
        #[serde(default)]
        subject: Option<ObjectId>,
        #[serde(default)]
        impact: Option<String>,
        #[serde(default)]
        next_role: Option<Role>,
        #[serde(default = "seven")]
        by_days: u64,
        #[serde(default)]
        refs: Vec<ObjectId>,
    },
    Advice {
        summary: String,
        #[serde(default)]
        input: Vec<ObjectId>,
    },
    PolicyEnvelope {
        queue: QueueKind,
        ceiling: u64,
        #[serde(default = "money")]
        means_kind: String,
        #[serde(default = "eur")]
        unit: String,
        #[serde(default = "year")]
        period_days: u64,
        #[serde(default = "full")]
        carry_over: CarryOver,
        #[serde(default)]
        overprogramming: bool,
    },
    PolicyAgreement {
        assets: Vec<String>,
        #[serde(default)]
        body: Option<ArtifactId>,
        /// `name:role` entries; default the signer in all three roles.
        #[serde(default)]
        signers: Vec<String>,
    },
    PolicyRule {
        #[serde(default)]
        body: Option<ArtifactId>,
        #[serde(default)]
        start_funding_share_bp: Option<u32>,
        /// `<advice-id>:followed|partial|rejected:<reason>` entries.
        #[serde(default)]
        adopts: Vec<String>,
    },
    Correction {
        corrects: ObjectId,
        reason: String,
        #[serde(default)]
        replacement: Option<ObjectId>,
        #[serde(default)]
        removal: bool,
    },
    Review {
        review: ReviewKind,
        subject: ObjectId,
        #[serde(default)]
        out_of_cycle: bool,
        #[serde(default)]
        trigger: Option<ObjectId>,
        #[serde(default = "event_str")]
        trigger_type: String,
        #[serde(default)]
        step: Option<u8>,
        #[serde(default)]
        artefact: Option<ArtifactId>,
    },
    /// File a ticket: a `report_back` event carrying `ticket`.
    Ticket {
        title: String,
        #[serde(default)]
        subject: Option<ObjectId>,
        #[serde(default)]
        body: Option<String>,
    },
    /// Close a ticket: a `report_back_resolved` event on the ticket id.
    TicketClose {
        ticket: ObjectId,
        #[serde(default)]
        reason: Option<String>,
    },
    /// Record an event envelope (NAP-corpus #12, Stage 0). The writer computes
    /// `payload_hash`, `cid` and the signature unless the client states them;
    /// stated values are checked, and a bad envelope is a refusal on the chain.
    EventEnvelope {
        payload: serde_json::Value,
        source: String,
        module: String,
        provenance: ProvenanceKind,
        /// Required for `measured` (ENV-06).
        #[serde(default)]
        evidence_tag: Option<EvidenceTag>,
        /// Required for `inferred` (ENV-06).
        #[serde(default)]
        source_ref: Option<String>,
        /// Default: a fresh UUIDv7.
        #[serde(default)]
        event_id: Option<String>,
        /// Seconds since the epoch. Default: now.
        #[serde(default)]
        unix_timestamp: Option<u64>,
        /// Default: `signed`. `draft` is stored unsigned; `sealed`/`anchored` are
        /// stored as declared (not enforced until Stage 1).
        #[serde(default)]
        lifecycle: Option<Lifecycle>,
        #[serde(default = "ed25519")]
        signature_scheme: SignatureScheme,
        #[serde(default)]
        payload_hash: Option<String>,
        #[serde(default)]
        cid: Option<String>,
    },
    /// Move an existing event_envelope to `to` (Stage 1). The writer copies the
    /// latest stored version, sets `lifecycle`, and signs when it leaves `draft`.
    /// The schema rules (ENV-04..08) decide; a refusal is recorded on the chain.
    EventEnvelopeAdvance {
        event_id: String,
        to: Lifecycle,
    },
    /// Test hook: panics inside the handler when the executor allows it.
    DebugPanic,
}

fn human() -> PrincipalKind {
    PrincipalKind::Human
}
fn seven() -> u64 {
    7
}
fn money() -> String {
    "money".into()
}
fn eur() -> String {
    "EUR".into()
}
fn year() -> u64 {
    365
}
fn full() -> CarryOver {
    CarryOver::Full
}
fn event_str() -> String {
    "event".into()
}
fn ed25519() -> SignatureScheme {
    SignatureScheme::Ed25519
}

impl Op {
    /// Snake_case operation name.
    pub fn name(&self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.get("op").and_then(|o| o.as_str()).map(String::from))
            .unwrap_or_else(|| "unknown".into())
    }

    pub fn is_read(&self) -> bool {
        matches!(
            self,
            Op::Head | Op::Check | Op::Show { .. } | Op::List { .. } | Op::Tickets { .. }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Error,
    Refused,
    Busy,
}

/// One response line. `lines` is what a CLI prints on stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: Status,
    pub ok: bool,
    #[serde(default)]
    pub id: Option<ObjectId>,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub rule: Option<String>,
    #[serde(default)]
    pub lines: Vec<String>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

impl Response {
    fn new(status: Status) -> Self {
        Self {
            status,
            ok: status == Status::Ok,
            id: None,
            hash: None,
            kind: None,
            error: None,
            rule: None,
            lines: vec![],
            data: None,
        }
    }

    pub fn ok(lines: Vec<String>) -> Self {
        Self {
            lines,
            ..Self::new(Status::Ok)
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            error: Some(msg.into()),
            ..Self::new(Status::Error)
        }
    }

    pub fn busy(msg: impl Into<String>) -> Self {
        Self {
            error: Some(msg.into()),
            ..Self::new(Status::Busy)
        }
    }

    pub fn refused(rule: &str, msg: impl Into<String>) -> Self {
        Self {
            rule: Some(rule.into()),
            error: Some(msg.into()),
            ..Self::new(Status::Refused)
        }
    }

    /// CLI exit code: 0 ok, 1 error/busy, 2 refused.
    pub fn exit_code(&self) -> u8 {
        match self.status {
            Status::Ok => 0,
            Status::Error | Status::Busy => 1,
            Status::Refused => 2,
        }
    }
}

/// How strictly `execute` treats the caller.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExecOptions {
    /// Refuse requests whose principal is not already bound (daemon mode).
    pub strict_principal: bool,
    /// Let `DebugPanic` actually panic (tests only).
    pub allow_debug_panic: bool,
}

/// Run one request. Never panics except for `DebugPanic` when allowed.
pub fn execute(k: &mut Kernel, req: Request, opts: ExecOptions) -> Response {
    match execute_inner(k, req, opts) {
        Ok(r) => r,
        Err(e) => Response::error(format!("{e:#}")),
    }
}

fn execute_inner(k: &mut Kernel, req: Request, opts: ExecOptions) -> Result<Response> {
    if let Op::DebugPanic = req.op {
        if opts.allow_debug_panic {
            panic!("debug_panic requested");
        }
        return Ok(Response::error("debug_panic is disabled"));
    }
    if req.op.is_read() {
        return read(k, &req.op);
    }
    // Resolve the signer.
    let name = req
        .principal
        .clone()
        .unwrap_or_else(|| k.meta.default_principal.clone());
    let known = k.state.principal_by_name(&name).map(|(id, p)| (id, p.kind));
    if opts.strict_principal && known.is_none() {
        return Ok(Response::refused(
            "X-10",
            format!("unknown principal {name}; bind it with role-bind before calling the daemon"),
        ));
    }
    if let (Some(declared), Some((_, kind))) = (req.producer_kind, known) {
        if declared != kind {
            return Ok(Response::refused(
                "R-08",
                format!("principal {name} is bound as {kind:?}, not {declared:?}"),
            ));
        }
    }
    let signer = k.principal(Some(&name)).0;
    let (mut extra, results) = write(k, signer, &name, req.role, req.producer_kind, req.op)?;
    Ok(finish(&mut extra, results))
}

fn finish(extra: &mut Vec<String>, results: Vec<Submit>) -> Response {
    let mut resp = Response::ok(vec![]);
    let mut lines = std::mem::take(extra);
    for r in results {
        match r {
            Submit::Accepted { id, kind, hash } => {
                lines.push(line(id, kind, &hash));
                resp.id = Some(id);
                resp.kind = Some(kind.into());
                resp.hash = Some(format!("{hash:x}"));
            }
            Submit::Refused(r) => {
                // Extra info lines (artifact/envelope ids) belong to accepted writes only.
                lines.clear();
                if let Some((id, hash)) = r.recorded {
                    lines.push(line(id, "event", &hash));
                }
                let mut refused =
                    Response::refused(r.rejection.rule, format!("REFUSED {}", r.rejection));
                refused.lines = lines;
                return refused;
            }
        }
    }
    resp.lines = lines;
    resp
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(s.replace('-', "_")))
        .or_else(|_| serde_json::from_value(serde_json::Value::String(s.to_string())))
        .map_err(|_| anyhow!("invalid value '{s}'"))
}

fn classification(s: Option<&str>) -> Result<Classification> {
    match s.unwrap_or("internal") {
        "public" => Ok(Classification::Public),
        "internal" => Ok(Classification::Internal),
        "restricted" => Ok(Classification::Restricted),
        other => bail!("invalid classification '{other}'"),
    }
}

fn media_type(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
    {
        "md" => "text/markdown",
        "txt" => "text/plain",
        "json" | "jsonl" => "application/json",
        "toml" => "application/toml",
        "rs" => "text/x-rust",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

const ANY_ROLE: [Role; 6] = [
    Role::Execution,
    Role::Policy,
    Role::Continuity,
    Role::Adviser,
    Role::MandateHolder,
    Role::Coordinator,
];

/// Execute a write. Returns extra info lines plus the submit results.
fn write(
    k: &mut Kernel,
    signer: PrincipalId,
    signer_name: &str,
    role: Option<Role>,
    producer_kind: Option<PrincipalKind>,
    op: Op,
) -> Result<(Vec<String>, Vec<Submit>)> {
    use Role::*;
    let one = |s: Submit| Ok((vec![], vec![s]));
    match op {
        Op::RoleBind {
            bound_role,
            target,
            kind,
        } => {
            let (target_id, target_name) = k.principal(Some(&target));
            one(k.submit(
                signer,
                role.unwrap_or(Continuity),
                None,
                vec![],
                Body::RoleBind(RoleBind {
                    bound_role,
                    principal: Some(target_id),
                    principal_name: Some(target_name),
                    principal_kind: Some(kind),
                }),
            )?)
        }
        Op::Ingest {
            path,
            content,
            name,
            purpose,
            tag,
            classification: class,
            replica,
        } => {
            let (bytes, locator) = match (path, content) {
                (Some(p), None) => (
                    std::fs::read(&p).with_context(|| format!("reading {p}"))?,
                    p,
                ),
                (None, Some(c)) => (
                    c.into_bytes(),
                    name.unwrap_or_else(|| format!("inline:{signer_name}")),
                ),
                _ => bail!("ingest needs exactly one of path or content"),
            };
            let classification = classification(class.as_deref())?;
            let digest = Sha256Digest::compute(&bytes);
            let role = k.pick_role(signer, role, &ANY_ROLE);
            let artifact_id = ArtifactId::new();
            let body = Body::Ingest(Ingest {
                outcome: if classification == Classification::Restricted {
                    IngestOutcome::AdmitRestricted
                } else {
                    IngestOutcome::Admit
                },
                purpose,
                artifact_id: Some(artifact_id),
                digest: format!("{digest:x}"),
                size_bytes: bytes.len() as u64,
                media_type: media_type(&locator).into(),
                source_locator: locator,
                evidence_tag: tag,
                reason: None,
                replica,
            });
            let solo = k.meta.solo;
            let result = k.submit_as(
                signer,
                role,
                None,
                vec![],
                solo,
                classification,
                body,
                move |k| {
                    k.cas
                        .put(artifact_id, &bytes)
                        .map(|_| ())
                        .map_err(|e| anyhow!("CAS: {e}"))
                },
            )?;
            Ok((
                vec![format!("artifact {artifact_id} sha256 {digest:x}")],
                vec![result],
            ))
        }
        Op::QueueKeep {
            asset,
            item,
            to,
            lead,
            adds_function,
            urgent_safety,
            evidence,
            by,
            reason,
        } => {
            let lead = lead.map(|l| k.principal(Some(&l)).0);
            let role = role.unwrap_or(Execution);
            let Some(id) = item else {
                return one(k.submit(
                    signer,
                    role,
                    None,
                    evidence,
                    Body::QueueKeep(QueueKeep {
                        from: None,
                        to: KeepState::Signalled,
                        asset_ref: asset
                            .ok_or_else(|| anyhow!("asset is required to create a keep item"))?,
                        lead,
                        adds_function,
                        urgent_safety,
                    }),
                )?);
            };
            let (current, item_asset) = match k.state.item(id) {
                Some(i) => (
                    match i.state {
                        ItemState::Keep(s) => Some(s),
                        _ => None,
                    },
                    i.asset_ref.clone(),
                ),
                None => (None, String::new()),
            };
            let to = to.ok_or_else(|| anyhow!("to is required to move a keep item"))?;
            let asset_ref = asset.unwrap_or(item_asset);
            if to == KeepState::Deferred {
                // K-09: displacement event, then the transition, as one unit.
                let displacement = Body::Event(Event {
                    event: EventKind::Displacement,
                    impact: Some(reason.unwrap_or_else(|| "displaced; reason not stated".into())),
                    next_decision: Some(NextDecision {
                        role: EventKind::Displacement.default_next_role(),
                        by: k.now() + 7 * DAY_MS,
                        options: vec!["re-plan at next programme refresh".into()],
                    }),
                    refs: by.into_iter().collect(),
                    detail: None,
                    ticket: None,
                });
                let defer = Body::QueueKeep(QueueKeep {
                    from: current,
                    to,
                    asset_ref,
                    lead,
                    adds_function: false,
                    urgent_safety: false,
                });
                let results = k.submit_batch(vec![
                    BatchPart {
                        signer,
                        role,
                        subject: Some(id),
                        evidence: vec![],
                        body: displacement,
                    },
                    BatchPart {
                        signer,
                        role,
                        subject: Some(id),
                        evidence,
                        body: defer,
                    },
                ])?;
                return Ok((vec![], results));
            }
            one(k.submit(
                signer,
                role,
                Some(id),
                evidence,
                Body::QueueKeep(QueueKeep {
                    from: current,
                    to,
                    asset_ref,
                    lead,
                    adds_function,
                    urgent_safety,
                }),
            )?)
        }
        Op::QueueBuild {
            asset,
            lead,
            evidence,
        } => {
            let lead = lead.map(|l| k.principal(Some(&l)).0);
            one(k.submit(
                signer,
                role.unwrap_or(Policy),
                None,
                evidence,
                Body::QueueBuild(QueueBuild {
                    asset_ref: asset,
                    lead,
                    adds_function: true,
                    programme_ref: None,
                }),
            )?)
        }
        Op::Gate {
            item,
            gate,
            outcome,
            evidence,
            envelope,
            required,
            available,
            discharge,
            agreement,
            keep_envelope,
        } => {
            let default_role = if gate == GateKind::Handover {
                MandateHolder
            } else {
                Policy
            };
            let role = role.unwrap_or(default_role);
            let means_check = match envelope {
                Some(env) => Some(MeansCheck {
                    envelope_ref: env,
                    required: required
                        .ok_or_else(|| anyhow!("required is needed with envelope"))?,
                    available: available
                        .unwrap_or_else(|| k.state.available_for(env, item).unwrap_or(0)),
                }),
                None => None,
            };
            let discharge = discharge.map(|name| Discharge {
                role: Execution,
                principal: k.principal(Some(&name)).0,
            });
            let keep = agreement.map(|a| KeepLink {
                agreement_ref: a,
                envelope_ref: keep_envelope,
            });
            one(k.submit(
                signer,
                role,
                Some(item),
                evidence,
                Body::Gate(Gate {
                    gate,
                    outcome,
                    signers: vec![Signer {
                        principal: signer,
                        role,
                    }],
                    means_check,
                    discharge,
                    keep,
                    valid_until: None,
                }),
            )?)
        }
        Op::Means {
            envelope,
            from,
            to,
            amount,
            unit,
            item,
            gate,
            agreement,
        } => {
            let default_role = if to == MeansTarget::Spent {
                Execution
            } else {
                Policy
            };
            let unit = unit
                .or_else(|| k.state.envelope(envelope).map(|e| e.envelope.unit.clone()))
                .unwrap_or_default();
            one(k.submit(
                signer,
                role.unwrap_or(default_role),
                None,
                vec![],
                Body::Means(Means {
                    envelope_ref: envelope,
                    from_state: from,
                    to_state: to,
                    amount,
                    unit,
                    item_ref: item,
                    gate_ref: gate,
                    agreement_ref: agreement,
                }),
            )?)
        }
        Op::Event {
            kind,
            subject,
            impact,
            next_role,
            by_days,
            refs,
        } => {
            let role = k.pick_role(signer, role, &ANY_ROLE);
            let (impact, next_decision) = if kind.is_failure() {
                (
                    // Recorded at detection; an unassessed impact stays visible in `check`.
                    Some(impact.unwrap_or_else(|| "unassessed".into())),
                    Some(NextDecision {
                        role: next_role.unwrap_or_else(|| kind.default_next_role()),
                        by: k.now() + by_days * DAY_MS,
                        options: vec![],
                    }),
                )
            } else {
                (impact, None)
            };
            one(k.submit(
                signer,
                role,
                subject,
                vec![],
                Body::Event(Event {
                    event: kind,
                    impact,
                    next_decision,
                    refs,
                    detail: None,
                    ticket: None,
                }),
            )?)
        }
        Op::Advice { summary, input } => {
            let role = k.pick_role(signer, role, &[Adviser, Execution, Policy, Continuity]);
            let producer_kind = producer_kind
                .or_else(|| k.state.principal(signer).map(|p| p.kind))
                .unwrap_or(PrincipalKind::Human);
            one(k.submit(
                signer,
                role,
                None,
                vec![],
                Body::Advice(Advice {
                    producer: signer,
                    producer_kind,
                    summary,
                    inputs: input,
                    addressed_to: None,
                    respond_by: None,
                }),
            )?)
        }
        Op::PolicyEnvelope {
            queue,
            ceiling,
            means_kind,
            unit,
            period_days,
            carry_over,
            overprogramming,
        } => {
            let now = k.now();
            let id = ObjectId::new_v7();
            let mut p = empty_policy(PolicySubKind::Envelope);
            p.signers = vec![Signer {
                principal: signer,
                role: Continuity,
            }];
            p.envelope = Some(Envelope {
                id,
                means_kind,
                unit,
                period: Period {
                    from_ms: now,
                    until_ms: now + period_days * DAY_MS,
                },
                ceiling,
                buckets: Buckets {
                    bound: 0,
                    committed: 0,
                    reserved: ceiling,
                },
                queue,
                carry_over,
                overprogramming,
            });
            let result = k.submit(signer, Continuity, None, vec![], Body::Policy(p))?;
            Ok((vec![format!("envelope {id}")], vec![result]))
        }
        Op::PolicyAgreement {
            assets,
            body,
            signers,
        } => {
            let signers = if signers.is_empty() {
                [Continuity, Policy, Execution]
                    .into_iter()
                    .map(|role| Signer {
                        principal: signer,
                        role,
                    })
                    .collect()
            } else {
                signers
                    .iter()
                    .map(|s| {
                        let (name, role) = s
                            .split_once(':')
                            .ok_or_else(|| anyhow!("signer is name:role"))?;
                        Ok(Signer {
                            principal: k.principal(Some(name)).0,
                            role: parse_enum(role)?,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
            };
            let own = signers
                .iter()
                .find(|s| s.principal == signer)
                .map(|s| s.role)
                .unwrap_or(Continuity);
            let mut p = empty_policy(PolicySubKind::Agreement);
            p.signers = signers;
            p.asset_scope = assets;
            p.body_ref = body;
            one(k.submit(signer, role.unwrap_or(own), None, vec![], Body::Policy(p))?)
        }
        Op::PolicyRule {
            body,
            start_funding_share_bp,
            adopts,
        } => {
            let mut p = empty_policy(PolicySubKind::Rule);
            p.signers = vec![Signer {
                principal: signer,
                role: Policy,
            }];
            p.body_ref = body;
            p.start_funding_share_bp = start_funding_share_bp;
            for a in adopts {
                let mut parts = a.splitn(3, ':');
                let (Some(id), Some(resp), Some(reason)) =
                    (parts.next(), parts.next(), parts.next())
                else {
                    bail!("adopt is <advice-id>:followed|partial|rejected:<reason>");
                };
                let id: ObjectId = id.parse().map_err(|e| anyhow!("{e}"))?;
                p.adopts.push(id);
                p.advice_response.push(AdviceResponse {
                    advice_id: id,
                    response: parse_enum(resp)?,
                    reason: reason.into(),
                });
            }
            one(k.submit(signer, Policy, None, vec![], Body::Policy(p))?)
        }
        Op::Correction {
            corrects,
            reason,
            replacement,
            removal,
        } => {
            // Default: the lowest held role that still ranks at least the original signer.
            let floor = k.state.origin_role(corrects).map(role_rank).unwrap_or(0);
            let mut candidates = [
                Adviser,
                Coordinator,
                Execution,
                Policy,
                MandateHolder,
                Continuity,
            ];
            candidates.sort_by_key(|r| role_rank(*r));
            let role = role.unwrap_or_else(|| {
                candidates
                    .into_iter()
                    .find(|r| role_rank(*r) >= floor && k.state.holds(signer, *r))
                    .unwrap_or(Continuity)
            });
            one(k.submit(
                signer,
                role,
                Some(corrects),
                vec![],
                Body::Correction(Correction {
                    corrects,
                    reason,
                    replacement,
                    removal,
                }),
            )?)
        }
        Op::Review {
            review,
            subject,
            out_of_cycle,
            trigger,
            trigger_type,
            step,
            artefact,
        } => {
            let preferred: &[Role] = match review {
                ReviewKind::Condition | ReviewKind::Replica => &[Execution, Policy, Continuity],
                ReviewKind::Account => &[Continuity, Policy, Execution],
                _ => &[Policy, Continuity, Execution],
            };
            let role = k.pick_role(signer, role, preferred);
            one(k.submit(
                signer,
                role,
                Some(subject),
                vec![],
                Body::Review(Review {
                    review,
                    rhythm: if out_of_cycle {
                        Rhythm::OutOfCycle
                    } else {
                        Rhythm::Scheduled
                    },
                    step,
                    artefact_ref: artefact,
                    trigger: trigger.map(|t| Trigger {
                        trigger_type,
                        trigger_ref: t,
                    }),
                }),
            )?)
        }
        Op::Ticket {
            title,
            subject,
            body,
        } => {
            let role = k.pick_role(signer, role, &ANY_ROLE);
            one(k.submit(
                signer,
                role,
                subject,
                vec![],
                Body::Event(Event {
                    event: EventKind::ReportBack,
                    impact: None,
                    next_decision: None,
                    refs: vec![],
                    detail: None,
                    ticket: Some(Ticket {
                        title,
                        body,
                        reporter: signer_name.to_string(),
                    }),
                }),
            )?)
        }
        Op::TicketClose { ticket, reason } => {
            let role = k.pick_role(signer, role, &ANY_ROLE);
            one(k.submit(
                signer,
                role,
                Some(ticket),
                vec![],
                Body::Event(Event {
                    event: EventKind::ReportBackResolved,
                    impact: reason,
                    next_decision: None,
                    refs: vec![],
                    detail: None,
                    ticket: None,
                }),
            )?)
        }
        Op::EventEnvelope {
            payload,
            source,
            module,
            provenance,
            evidence_tag,
            source_ref,
            event_id,
            unix_timestamp,
            lifecycle,
            signature_scheme,
            payload_hash,
            cid,
        } => {
            let role = k.pick_role(signer, role, &ANY_ROLE);
            // Unencodable payloads get empty addresses; ENV-02 refuses them on the chain.
            let (hash, address) = envelope::payload_bytes(&payload)
                .map(|b| envelope::address(&b))
                .unwrap_or_default();
            let lifecycle = lifecycle.unwrap_or(Lifecycle::Signed);
            let mut env = EventEnvelope {
                event_id: event_id.unwrap_or_else(|| ObjectId::new_v7().to_string()),
                unix_timestamp: unix_timestamp.unwrap_or(k.now() / 1000),
                source,
                module,
                provenance,
                evidence_tag,
                source_ref,
                payload,
                payload_hash: payload_hash.unwrap_or(hash),
                cid: cid.unwrap_or(address),
                duplicate_of: None,
                lifecycle,
                signature_scheme,
                signature: None,
            };
            // ENV-09: same bytes under another event_id are kept as a reference to the first.
            env.duplicate_of = k
                .state
                .cid_owner(&env.cid)
                .filter(|owner| *owner != env.event_id)
                .map(String::from);
            if lifecycle != Lifecycle::Draft && envelope::payload_bytes(&env.payload).is_ok() {
                let node = k.node_key()?;
                match signature_scheme {
                    SignatureScheme::Ed25519 => envelope::sign(&mut env, &node),
                    SignatureScheme::MlDsa65Stub => {
                        envelope::sign(&mut env, &MlDsa65Stub::new(node.public_key()))
                    }
                }
                .map_err(|e| anyhow!(e))?;
            }
            let mut cid_line = format!("event_envelope {} cid {}", env.event_id, env.cid);
            if let Some(owner) = &env.duplicate_of {
                cid_line.push_str(&format!(" duplicate_of {owner}"));
            }
            let result = k.submit(signer, role, None, vec![], Body::EventEnvelope(env))?;
            Ok((vec![cid_line], vec![result]))
        }
        Op::EventEnvelopeAdvance { event_id, to } => {
            let role = k.pick_role(signer, role, &ANY_ROLE);
            let Some(mut env) = k.state.event_envelope(&event_id).cloned() else {
                bail!("no event_envelope with event_id {event_id} on this chain");
            };
            let from = env.lifecycle;
            env.lifecycle = to;
            if to == Lifecycle::Draft {
                env.signature = None;
            } else if from == Lifecycle::Draft {
                let node = k.node_key()?;
                match env.signature_scheme {
                    SignatureScheme::Ed25519 => envelope::sign(&mut env, &node),
                    SignatureScheme::MlDsa65Stub => {
                        envelope::sign(&mut env, &MlDsa65Stub::new(node.public_key()))
                    }
                }
                .map_err(|e| anyhow!(e))?;
            }
            let name = |l: Lifecycle| format!("{l:?}").to_lowercase();
            let line = format!("event_envelope {event_id} {} -> {}", name(from), name(to));
            let result = k.submit(signer, role, None, vec![], Body::EventEnvelope(env))?;
            Ok((vec![line], vec![result]))
        }
        Op::Head
        | Op::Check
        | Op::Show { .. }
        | Op::List { .. }
        | Op::Tickets { .. }
        | Op::DebugPanic => bail!("not a write operation"),
    }
}

fn empty_policy(sub_kind: PolicySubKind) -> Policy {
    Policy {
        sub_kind,
        body_ref: None,
        signers: vec![],
        adopts: vec![],
        advice_response: vec![],
        supersedes: None,
        reaffirms: None,
        envelope: None,
        asset_scope: vec![],
        start_funding_share_bp: None,
    }
}

fn read(k: &Kernel, op: &Op) -> Result<Response> {
    match op {
        Op::Head => {
            let (lines, data) = match k.head() {
                Some((id, hash)) => (
                    vec![format!("{id} {hash:x} {}", k.entries.len())],
                    serde_json::json!({ "id": id, "hash": format!("{hash:x}"), "receipts": k.entries.len() }),
                ),
                None => (vec!["empty".into()], serde_json::json!({ "receipts": 0 })),
            };
            let mut r = Response::ok(lines);
            r.data = Some(data);
            Ok(r)
        }
        Op::Check => Ok(check(k)),
        Op::Show { id } => show(k, *id),
        Op::List { what } => Ok(list(k, *what)),
        Op::Tickets { open } => Ok(tickets(k, *open)),
        _ => bail!("not a read operation"),
    }
}

fn item_line(item: &apparatus_schema::rws::WorkItem) -> String {
    let (queue, state) = match item.state {
        ItemState::Keep(s) => ("keep", format!("{s:?}")),
        ItemState::Build(s) => ("build", format!("{s:?}")),
    };
    format!("item {} {queue} {state} asset={}", item.id, item.asset_ref)
}

fn ticket_lines(k: &Kernel, open_only: bool) -> (Vec<String>, serde_json::Value) {
    let open: Vec<ObjectId> = k.state.open_tickets().iter().map(|t| t.id).collect();
    let mut lines = vec![];
    let mut data = vec![];
    for t in k.state.tickets() {
        let is_open = open.contains(&t.id);
        if open_only && !is_open {
            continue;
        }
        let tk = t.ticket.as_ref().expect("ticket");
        lines.push(format!(
            "ticket {} {} subject={} reporter={} title={}",
            t.id,
            if is_open { "open" } else { "resolved" },
            t.subject
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            tk.reporter,
            tk.title
        ));
        data.push(serde_json::json!({
            "id": t.id, "open": is_open, "subject": t.subject,
            "title": tk.title, "body": tk.body, "reporter": tk.reporter,
        }));
    }
    (lines, serde_json::Value::Array(data))
}

fn tickets(k: &Kernel, open_only: bool) -> Response {
    let (lines, data) = ticket_lines(k, open_only);
    let mut r = Response::ok(lines);
    r.data = Some(data);
    r
}

fn list(k: &Kernel, what: ListWhat) -> Response {
    match what {
        ListWhat::Items => Response::ok(k.state.items().map(item_line).collect()),
        ListWhat::Events => Response::ok(
            k.state
                .events()
                .iter()
                .map(|e| {
                    format!(
                        "event {} {:?} subject={}",
                        e.id,
                        e.event,
                        e.subject
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "-".into())
                    )
                })
                .collect(),
        ),
        ListWhat::Tickets => tickets(k, false),
    }
}

fn show(k: &Kernel, id: ObjectId) -> Result<Response> {
    let mut lines = vec![];
    if let Some(item) = k.state.item(id) {
        let (queue, state) = match item.state {
            ItemState::Keep(s) => ("keep", format!("{s:?}")),
            ItemState::Build(s) => ("build", format!("{s:?}")),
        };
        lines.push(format!(
            "work_item {id} queue={queue} state={state} asset={} adds_function={}",
            item.asset_ref, item.adds_function
        ));
    }
    if let Some(env) = k.state.envelope(id) {
        let e = &env.envelope;
        lines.push(format!(
            "envelope {id} queue={:?} ceiling={} {} reserved={} committed={} bound={} spent={}",
            e.queue,
            e.ceiling,
            e.unit,
            e.buckets.reserved,
            e.buckets.committed,
            e.buckets.bound,
            env.spent
        ));
    }
    if let Some(a) = k.state.artifact(id) {
        lines.push(format!(
            "artifact {id} sha256={} tag={:?} replica={}",
            a.digest, a.tag, a.replica
        ));
    }
    if let Some(entry) = k.receipt(id) {
        lines.push(format!("receipt {id} hash={:x}", entry.hash));
        lines.push(serde_json::to_string_pretty(&entry.receipt)?);
    }
    if lines.is_empty() {
        return Ok(Response::error(format!("{id} not found")));
    }
    Ok(Response::ok(lines))
}

/// Full consistency report: disk chain, CAS, buffer, roles, items, refusals,
/// open failures and open tickets.
pub fn check(k: &Kernel) -> Response {
    let mut out = vec![];
    let mut problems: Vec<String> = vec![];
    out.push(format!("project {} ({})", k.meta.name, k.meta.project_id));
    if let Err(e) = k.verify_disk() {
        out.push(format!("FAIL chain: {e:#}"));
        let mut r = Response::error("check failed");
        r.lines = out;
        return r;
    }
    match k.head() {
        Some((id, hash)) => out.push(format!(
            "chain ok: {} receipts, head {id} {hash:x}",
            k.entries.len()
        )),
        None => out.push("chain ok: empty".into()),
    }
    if buffer_path(k.root()).exists() && k.head().is_some() {
        problems.push("rws/receipts.jsonl exists next to a live chain: two chains (X-11)".into());
    }
    let mut verified = 0;
    for (id, a) in k.state.artifacts() {
        match Sha256Digest::from_hex(&a.digest).map(|d| k.cas.verify(&d)) {
            Ok(Ok(true)) => verified += 1,
            _ => problems.push(format!("artefact {id}: bytes missing or corrupt in CAS")),
        }
    }
    out.push(format!("cas ok: {verified} artefacts verified"));
    for role in [
        Role::Continuity,
        Role::Policy,
        Role::Execution,
        Role::MandateHolder,
        Role::Adviser,
        Role::Coordinator,
    ] {
        let names: Vec<String> = k
            .state
            .holders(role)
            .iter()
            .map(|p| {
                k.state
                    .principal(*p)
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| p.to_string())
            })
            .collect();
        if !names.is_empty() {
            out.push(format!("role {role}: {}", names.join(", ")));
        }
    }
    if k.state.holders(Role::Execution).is_empty() {
        out.push("note: no execution holder bound (R-02); keep work cannot be queued".into());
    }
    out.extend(k.state.items().map(item_line));
    let refusals: Vec<_> = k
        .state
        .events()
        .iter()
        .filter(|e| e.event == EventKind::IllegalTransitionRejected)
        .collect();
    out.push(format!("refusals recorded: {}", refusals.len()));
    for r in &refusals {
        out.push(format!(
            "  refused {} {}",
            r.id,
            r.impact.as_deref().unwrap_or("")
        ));
    }
    let open: Vec<_> = k
        .state
        .open_failures()
        .into_iter()
        .filter(|e| e.event != EventKind::IllegalTransitionRejected)
        .collect();
    out.push(format!("open failures: {}", open.len()));
    for f in open {
        let nd = f
            .next_decision
            .as_ref()
            .map(|n| format!("next: {} by {}", n.role, n.by))
            .unwrap_or_default();
        let flag = if f.impact.as_deref() == Some("unassessed") {
            " [impact unassessed]"
        } else {
            ""
        };
        out.push(format!(
            "  {:?} {} subject={} {nd}{flag}",
            f.event,
            f.id,
            f.subject
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into())
        ));
    }
    let (ticket_out, _) = ticket_lines(k, true);
    out.push(format!("open tickets: {}", ticket_out.len()));
    out.extend(ticket_out.into_iter().map(|l| format!("  {l}")));
    if problems.is_empty() {
        out.push("check ok".into());
        Response::ok(out)
    } else {
        out.extend(problems.iter().map(|p| format!("FAIL {p}")));
        let mut r = Response::error("check failed");
        r.lines = out;
        r
    }
}

//! RWS 2.0 validation: stateless payload checks (`Validate for Payload`) and a
//! chain `State` that folds receipts and refuses illegal transitions.
//!
//! Rule ids in rejections refer to RWS-2.0.md in `infraax/delta`.

use crate::{Validate, ValidationError};
use apparatus_types::rws::*;
use apparatus_types::{Classification, ObjectHeader, ObjectId, PrincipalId, ProjectId};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A refused receipt: the rule that refused it and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub rule: &'static str,
    pub reason: String,
}

impl Rejection {
    fn new(rule: &'static str, reason: impl Into<String>) -> Self {
        Self {
            rule,
            reason: reason.into(),
        }
    }
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.rule, self.reason)
    }
}

impl std::error::Error for Rejection {}

impl From<ValidationError> for Rejection {
    fn from(e: ValidationError) -> Self {
        match e {
            ValidationError::Rule { rule, reason } => Rejection::new(rule, reason),
            ValidationError::InvalidTimestamp { .. } => Rejection::new("C-07", e.to_string()),
            ValidationError::MissingInvariant(_) => Rejection::new("C-04", e.to_string()),
            ValidationError::EmptyField { .. } => Rejection::new("C-05", e.to_string()),
        }
    }
}

fn rule(rule: &'static str, reason: impl Into<String>) -> ValidationError {
    ValidationError::Rule {
        rule,
        reason: reason.into(),
    }
}

fn is_hex_digest(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Decode an `operation_payload` JSON value. Unknown kinds or event kinds fail (X-05.1, F-02).
pub fn decode_payload(value: &serde_json::Value) -> Result<Payload, Rejection> {
    serde_json::from_value(value.clone())
        .map_err(|e| Rejection::new("X-05.1", format!("payload not in the v1 vocabulary: {e}")))
}

/// Structural checks that need no chain state.
impl Validate for Payload {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.rws != RWS_VERSION {
            return Err(rule(
                "X-03",
                format!("rws version {} is not {RWS_VERSION}", self.rws),
            ));
        }
        match &self.body {
            Body::Ingest(i) => {
                if i.purpose.trim().is_empty() {
                    return Err(rule("C-03", "ingest needs a purpose"));
                }
                if !is_hex_digest(&i.digest) {
                    return Err(rule("C-03", "digest must be 64 lowercase hex characters"));
                }
                match i.outcome {
                    IngestOutcome::Admit | IngestOutcome::AdmitRestricted => {
                        if i.artifact_id.is_none() {
                            return Err(rule("C-03", "admitted artefact needs an artifact_id"));
                        }
                    }
                    IngestOutcome::Reject | IngestOutcome::Quarantine => {
                        if i.reason.as_deref().unwrap_or("").trim().is_empty() {
                            return Err(rule("C-03", "reject/quarantine needs a reason"));
                        }
                    }
                    IngestOutcome::ReferenceOnly => {}
                }
            }
            Body::RoleBind(b) => {
                if let Some(kind) = b.principal_kind {
                    if matches!(kind, PrincipalKind::Agent | PrincipalKind::ExternalModel)
                        && !matches!(b.bound_role, Role::Execution | Role::Adviser)
                    {
                        return Err(rule(
                            "R-08",
                            format!(
                                "{kind:?} principals may hold only execution or adviser, not {}",
                                b.bound_role
                            ),
                        ));
                    }
                }
            }
            Body::QueueKeep(q) => {
                if q.asset_ref.trim().is_empty() {
                    return Err(rule("K-01", "keep item needs an asset_ref"));
                }
                if q.from.is_none() && q.to != KeepState::Signalled {
                    return Err(rule("K-06", "a keep item starts as signalled"));
                }
            }
            Body::QueueBuild(q) => {
                if q.asset_ref.trim().is_empty() {
                    return Err(rule("B-03", "build item needs an asset_ref"));
                }
                if !q.adds_function {
                    return Err(rule(
                        "K-07",
                        "build is for new or changed function; adds_function must be true",
                    ));
                }
                if self.subject_id.is_some() {
                    return Err(rule(
                        "B-03",
                        "queue_build only creates; build items move through gates",
                    ));
                }
            }
            Body::Gate(g) => {
                if self.subject_id.is_none() {
                    return Err(rule("B-02", "gate needs a subject work item"));
                }
                if g.signers.is_empty() {
                    return Err(rule("B-02", "gate without signers"));
                }
                if self.evidence.is_empty() {
                    return Err(rule("B-02", "gate without evidence artefact"));
                }
                if g.gate == GateKind::Handover && g.outcome == GateOutcome::Go {
                    if g.discharge.is_none() {
                        return Err(rule("B-10", "handover go needs a discharge of execution"));
                    }
                    if g.keep.is_none() {
                        return Err(rule("B-10", "handover go needs keep.agreement_ref"));
                    }
                }
            }
            Body::Means(m) => {
                if m.amount == 0 {
                    return Err(rule("M-01", "amount must be positive"));
                }
                let same = matches!(
                    (m.from_state, m.to_state),
                    (MeansState::Bound, MeansTarget::Bound)
                        | (MeansState::Committed, MeansTarget::Committed)
                        | (MeansState::Reserved, MeansTarget::Reserved)
                );
                if same {
                    return Err(rule("M-01", "from and to are the same state"));
                }
                if m.from_state == MeansState::Reserved && m.to_state == MeansTarget::Spent {
                    return Err(rule("M-02", "spending from reserved is not allowed"));
                }
                if m.from_state == MeansState::Bound
                    && matches!(m.to_state, MeansTarget::Reserved | MeansTarget::Committed)
                {
                    return Err(rule("M-10", "bound means cannot move back"));
                }
            }
            Body::Advice(a) => {
                if a.summary.trim().is_empty() {
                    return Err(rule("A-05", "advice needs a summary"));
                }
            }
            Body::Policy(p) => {
                if p.adopts.len() != p.advice_response.len()
                    || !p
                        .adopts
                        .iter()
                        .all(|id| p.advice_response.iter().any(|r| r.advice_id == *id))
                {
                    return Err(rule(
                        "A-03",
                        "every adopted advice needs an advice_response",
                    ));
                }
                match p.sub_kind {
                    PolicySubKind::Envelope => {
                        let Some(e) = &p.envelope else {
                            return Err(rule("O-09", "envelope policy without envelope"));
                        };
                        if e.queue == QueueKind::Keep && e.overprogramming {
                            return Err(rule(
                                "K-04",
                                "overprogramming is not allowed on a keep envelope",
                            ));
                        }
                        if e.ceiling == 0 || e.period.until_ms <= e.period.from_ms {
                            return Err(rule(
                                "O-09",
                                "envelope needs a positive ceiling and a period",
                            ));
                        }
                        if e.buckets.bound != 0 || e.buckets.committed != 0 {
                            return Err(rule(
                                "O-09",
                                "a new envelope starts with everything reserved",
                            ));
                        }
                        if e.buckets.reserved > e.ceiling
                            && !(e.queue == QueueKind::Build && e.overprogramming)
                        {
                            return Err(rule("O-09", "reserved exceeds ceiling"));
                        }
                    }
                    PolicySubKind::Agreement => {
                        if p.asset_scope.is_empty()
                            || p.asset_scope.iter().any(|a| a.trim().is_empty())
                        {
                            return Err(rule(
                                "K-01",
                                "agreement needs a non-empty asset_scope (names or \"*\")",
                            ));
                        }
                    }
                    PolicySubKind::Rule => {
                        if p.start_funding_share_bp.is_some_and(|bp| bp > 10_000) {
                            return Err(rule("M-06", "start_funding_share_bp above 10000"));
                        }
                    }
                    PolicySubKind::Framework => {}
                }
                if p.sub_kind != PolicySubKind::Envelope && p.envelope.is_some() {
                    return Err(rule("O-04", "envelope field only on envelope policies"));
                }
            }
            Body::Event(e) => {
                if e.event.is_failure() {
                    if e.impact.as_deref().unwrap_or("").trim().is_empty() {
                        return Err(rule("F-01", "failure event needs an impact"));
                    }
                    if e.next_decision.is_none() {
                        return Err(rule("F-01", "failure event needs a next_decision"));
                    }
                }
                if let Some(t) = &e.ticket {
                    if e.event != EventKind::ReportBack {
                        return Err(rule(
                            "A-07",
                            "a ticket is carried only by a report_back event",
                        ));
                    }
                    if t.title.trim().is_empty() || t.reporter.trim().is_empty() {
                        return Err(rule("A-07", "a ticket needs a title and a reporter"));
                    }
                }
            }
            Body::EventEnvelope(e) => crate::envelope::validate(e)?,
            Body::Correction(c) => {
                if c.reason.trim().is_empty() {
                    return Err(rule("C-06", "correction needs a reason"));
                }
            }
            Body::Review(r) => {
                if r.rhythm == Rhythm::OutOfCycle && r.trigger.is_none() {
                    return Err(rule("V-04", "out-of-cycle review needs a named trigger"));
                }
                if r.review == ReviewKind::Strategic
                    && !r.step.is_some_and(|s| (1..=6).contains(&s))
                {
                    return Err(rule("V-02", "strategic review needs a step 1..6"));
                }
            }
        }
        Ok(())
    }
}

/// Where a work item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    Keep(KeepState),
    Build(BuildState),
}

impl ItemState {
    pub fn queue(&self) -> QueueKind {
        match self {
            ItemState::Keep(_) => QueueKind::Keep,
            ItemState::Build(_) => QueueKind::Build,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: ObjectId,
    pub state: ItemState,
    pub asset_ref: String,
    pub lead: Option<PrincipalId>,
    pub adds_function: bool,
    /// Chain position of the item's latest transition.
    pub last_seq: usize,
}

#[derive(Debug, Clone)]
pub struct EnvelopeState {
    pub envelope: Envelope,
    pub spent: u64,
    /// Per work item: (committed, bound).
    pub per_item: BTreeMap<ObjectId, (u64, u64)>,
}

#[derive(Debug, Clone)]
pub struct ArtifactInfo {
    pub digest: String,
    pub replica: bool,
    pub tag: EvidenceTag,
}

#[derive(Debug, Clone, Copy)]
pub struct GateRecord {
    pub item: ObjectId,
    pub gate: GateKind,
    pub outcome: GateOutcome,
}

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub id: ObjectId,
    pub event: EventKind,
    pub subject: Option<ObjectId>,
    pub impact: Option<String>,
    pub next_decision: Option<NextDecision>,
    pub detail: Option<String>,
    pub ticket: Option<Ticket>,
}

#[derive(Debug, Clone)]
pub struct PrincipalInfo {
    pub name: String,
    pub kind: PrincipalKind,
}

/// Folded view of one project's receipt chain.
#[derive(Debug, Clone, Default)]
pub struct State {
    project_id: Option<ProjectId>,
    receipts: BTreeMap<ObjectId, &'static str>,
    principals: BTreeMap<PrincipalId, PrincipalInfo>,
    roles: BTreeMap<Role, Vec<PrincipalId>>,
    items: BTreeMap<ObjectId, WorkItem>,
    envelopes: BTreeMap<ObjectId, EnvelopeState>,
    artifacts: BTreeMap<ObjectId, ArtifactInfo>,
    digests: BTreeSet<String>,
    advice: BTreeSet<ObjectId>,
    policies: BTreeMap<ObjectId, PolicySubKind>,
    gates: BTreeMap<ObjectId, GateRecord>,
    events: Vec<EventRecord>,
    start_funding_share_bp: Option<u32>,
    /// Chain position of the next receipt to be folded.
    seq: usize,
    /// Active (not superseded) keep agreements and their asset scope (K-01).
    agreements: BTreeMap<ObjectId, Vec<String>>,
    /// Role under which each object was created or last signed (C-06).
    origin_role: BTreeMap<ObjectId, Role>,
    /// Chain position of the latest displacement event per subject (K-09).
    last_displacement: BTreeMap<ObjectId, usize>,
    /// Latest accepted version of each event_envelope, by `event_id` (ENV-05, ENV-07).
    envelope_events: BTreeMap<String, EventEnvelope>,
    /// First `event_id` that carried each CID (ENV-09). One content address, one owner.
    envelope_cids: BTreeMap<String, String>,
}

/// Rank used for corrections: a corrector must rank at least as high as the
/// role that signed the corrected object (RWS-2.0-MAPPING §2, `correction`).
pub fn role_rank(role: Role) -> u8 {
    match role {
        Role::Continuity => 4,
        Role::Policy | Role::MandateHolder => 3,
        Role::Execution => 2,
        Role::Adviser | Role::Coordinator => 1,
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of receipts folded so far.
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    pub fn project_id(&self) -> Option<ProjectId> {
        self.project_id
    }

    pub fn principal(&self, id: PrincipalId) -> Option<&PrincipalInfo> {
        self.principals.get(&id)
    }

    pub fn principal_by_name(&self, name: &str) -> Option<(PrincipalId, &PrincipalInfo)> {
        self.principals
            .iter()
            .find(|(_, p)| p.name == name)
            .map(|(id, p)| (*id, p))
    }

    pub fn holders(&self, role: Role) -> &[PrincipalId] {
        self.roles.get(&role).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn holds(&self, principal: PrincipalId, role: Role) -> bool {
        self.holders(role).contains(&principal)
    }

    pub fn roles_of(&self, principal: PrincipalId) -> Vec<Role> {
        self.roles
            .iter()
            .filter(|(_, ps)| ps.contains(&principal))
            .map(|(r, _)| *r)
            .collect()
    }

    pub fn item(&self, id: ObjectId) -> Option<&WorkItem> {
        self.items.get(&id)
    }

    pub fn items(&self) -> impl Iterator<Item = &WorkItem> {
        self.items.values()
    }

    pub fn envelope(&self, id: ObjectId) -> Option<&EnvelopeState> {
        self.envelopes.get(&id)
    }

    pub fn artifact(&self, id: ObjectId) -> Option<&ArtifactInfo> {
        self.artifacts.get(&id)
    }

    pub fn artifacts(&self) -> impl Iterator<Item = (&ObjectId, &ArtifactInfo)> {
        self.artifacts.iter()
    }

    /// True if an envelope with this `event_id` is on the chain.
    pub fn has_envelope_event(&self, event_id: &str) -> bool {
        self.envelope_events.contains_key(event_id)
    }

    /// The `event_id` that first carried `cid`, if any.
    pub fn cid_owner(&self, cid: &str) -> Option<&str> {
        self.envelope_cids.get(cid).map(String::as_str)
    }

    /// The latest accepted version of an event_envelope.
    pub fn event_envelope(&self, event_id: &str) -> Option<&EventEnvelope> {
        self.envelope_events.get(event_id)
    }

    pub fn has_digest(&self, digest: &str) -> bool {
        self.digests.contains(digest)
    }

    pub fn receipt_kind(&self, id: ObjectId) -> Option<&'static str> {
        self.receipts.get(&id).copied()
    }

    pub fn is_advice(&self, id: ObjectId) -> bool {
        self.advice.contains(&id)
    }

    pub fn policy_kind(&self, id: ObjectId) -> Option<PolicySubKind> {
        self.policies.get(&id).copied()
    }

    pub fn gate(&self, id: ObjectId) -> Option<&GateRecord> {
        self.gates.get(&id)
    }

    pub fn events(&self) -> &[EventRecord] {
        &self.events
    }

    /// All tickets (report_back events carrying a ticket), oldest first.
    pub fn tickets(&self) -> Vec<&EventRecord> {
        self.events.iter().filter(|e| e.ticket.is_some()).collect()
    }

    /// Tickets without a later `report_back_resolved` event on them.
    pub fn open_tickets(&self) -> Vec<&EventRecord> {
        let resolved: BTreeSet<ObjectId> = self
            .events
            .iter()
            .filter(|e| e.event == EventKind::ReportBackResolved)
            .flat_map(|e| e.subject)
            .collect();
        self.tickets()
            .into_iter()
            .filter(|t| !resolved.contains(&t.id))
            .collect()
    }

    /// Active keep agreements: id → asset scope.
    pub fn agreements(&self) -> &BTreeMap<ObjectId, Vec<String>> {
        &self.agreements
    }

    /// True when an active agreement covers `asset` by name or by `*` (K-01).
    pub fn agreement_covers(&self, asset: &str) -> bool {
        self.agreements
            .values()
            .any(|scope| scope.iter().any(|a| a == "*" || a == asset))
    }

    /// Role under which the object was created or signed.
    pub fn origin_role(&self, id: ObjectId) -> Option<Role> {
        self.origin_role.get(&id).copied()
    }

    /// Failure events not referenced by any later event (open for decision).
    pub fn open_failures(&self) -> Vec<&EventRecord> {
        let closed: BTreeSet<ObjectId> = self
            .events
            .iter()
            .filter(|e| !e.event.is_failure())
            .flat_map(|e| e.subject)
            .collect();
        self.events
            .iter()
            .filter(|e| e.event.is_failure() && !closed.contains(&e.id))
            .collect()
    }

    /// Means available to `item` in `envelope`: unallocated reserve plus what
    /// is already committed to that item.
    pub fn available_for(&self, envelope: ObjectId, item: ObjectId) -> Option<u64> {
        let e = self.envelopes.get(&envelope)?;
        let committed = e.per_item.get(&item).map(|(c, _)| *c).unwrap_or(0);
        Some(e.envelope.buckets.reserved.saturating_add(committed))
    }

    /// True when any object with this id is known.
    pub fn exists(&self, id: ObjectId) -> bool {
        self.receipts.contains_key(&id)
            || self.items.contains_key(&id)
            || self.envelopes.contains_key(&id)
            || self.artifacts.contains_key(&id)
    }

    /// Check a candidate receipt against the chain as folded so far.
    pub fn check(&self, header: &ObjectHeader, payload: &Payload) -> Result<(), Rejection> {
        header.validate()?;
        payload.validate()?;

        if let Some(pid) = self.project_id {
            if header.project_id != pid {
                return Err(Rejection::new("C-02", "receipt belongs to another project"));
            }
        }
        if self.receipts.contains_key(&header.id) {
            return Err(Rejection::new("C-01", "receipt id already used"));
        }
        if let Some(prov) = &header.provenance {
            if prov.creator != payload.signer {
                return Err(Rejection::new(
                    "C-05",
                    "provenance creator must be the signer",
                ));
            }
        }

        // Authority: never advice, never a replica (A-01, A-08).
        if let Some(auth) = payload.authority_ref {
            self.check_authority(auth)?;
        }

        // Signer must hold the role it signs under (R-09), except the bootstrap
        // bind and refusal records (which document an attempt, not an act).
        let is_refusal = matches!(&payload.body, Body::Event(e) if e.event == EventKind::IllegalTransitionRejected);
        if self.is_empty() {
            let bootstrap = matches!(&payload.body, Body::RoleBind(b)
                if b.bound_role == Role::Continuity && b.principal == Some(payload.signer))
                && payload.role == Role::Continuity;
            if !bootstrap {
                return Err(Rejection::new(
                    "R-09",
                    "the chain must start with a continuity bootstrap bind (rws init)",
                ));
            }
        } else if !is_refusal && !self.holds(payload.signer, payload.role) {
            return Err(Rejection::new(
                "R-09",
                format!("signer does not hold role {}", payload.role),
            ));
        }
        if let Some(p) = self.principals.get(&payload.signer) {
            if matches!(p.kind, PrincipalKind::Agent | PrincipalKind::ExternalModel)
                && matches!(
                    payload.role,
                    Role::Policy | Role::Continuity | Role::MandateHolder
                )
            {
                return Err(Rejection::new(
                    "R-08",
                    "agents never sign as policy, continuity or mandate holder",
                ));
            }
        }

        for ev in &payload.evidence {
            if !self.artifacts.contains_key(&ev.inner()) {
                return Err(Rejection::new(
                    "B-02",
                    format!("evidence {ev} is not an admitted artefact"),
                ));
            }
        }

        match &payload.body {
            Body::Ingest(i) => self.check_ingest(header, i),
            Body::RoleBind(b) => self.check_role_bind(payload, b),
            Body::QueueKeep(q) => self.check_queue_keep(payload, q),
            Body::QueueBuild(q) => self.check_queue_build(payload, q),
            Body::Gate(g) => self.check_gate(payload, g),
            Body::Means(m) => self.check_means(payload, m),
            Body::Advice(a) => self.check_refs(&a.inputs, "A-05"),
            Body::Policy(p) => self.check_policy(payload, p),
            Body::Event(e) => self.check_event(payload, e),
            Body::Correction(c) => self.check_correction(payload, c),
            Body::Review(r) => self.check_review(payload, r),
            Body::EventEnvelope(e) => self.check_envelope(e),
        }
    }

    /// Check, then fold. On rejection the state is unchanged.
    pub fn admit(&mut self, header: &ObjectHeader, payload: &Payload) -> Result<(), Rejection> {
        self.check(header, payload)?;
        self.apply(header, payload);
        Ok(())
    }

    fn check_authority(&self, id: ObjectId) -> Result<(), Rejection> {
        if self.advice.contains(&id) {
            return Err(Rejection::new(
                "A-01",
                "advice cannot be the authority of a transition",
            ));
        }
        if self.artifacts.get(&id).is_some_and(|a| a.replica) {
            return Err(Rejection::new(
                "A-08",
                "a replica cannot be the authority of a transition",
            ));
        }
        if !self.exists(id) {
            return Err(Rejection::new(
                "X-05.3",
                format!("authority {id} does not exist"),
            ));
        }
        Ok(())
    }

    fn check_refs(&self, refs: &[ObjectId], rule_id: &'static str) -> Result<(), Rejection> {
        match refs.iter().find(|r| !self.exists(**r)) {
            Some(missing) => Err(Rejection::new(
                rule_id,
                format!("reference {missing} does not exist"),
            )),
            None => Ok(()),
        }
    }

    fn check_ingest(&self, header: &ObjectHeader, i: &Ingest) -> Result<(), Rejection> {
        let restricted = header.classification == Classification::Restricted;
        match i.outcome {
            IngestOutcome::Admit | IngestOutcome::AdmitRestricted => {
                if restricted != (i.outcome == IngestOutcome::AdmitRestricted) {
                    return Err(Rejection::new(
                        "C-04",
                        "restricted classification requires outcome admit_restricted (and vice versa)",
                    ));
                }
                if self.digests.contains(&i.digest) {
                    return Err(Rejection::new(
                        "C-03",
                        "these bytes are already admitted (append-only, no duplicate)",
                    ));
                }
                if let Some(id) = i.artifact_id {
                    if self.exists(id.inner()) {
                        return Err(Rejection::new("C-01", "artifact id already used"));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn check_role_bind(&self, payload: &Payload, b: &RoleBind) -> Result<(), Rejection> {
        if payload.role != Role::Continuity {
            return Err(Rejection::new("R-09", "only continuity binds roles"));
        }
        match b.principal {
            None => {
                if self.holders(b.bound_role).is_empty() {
                    return Err(Rejection::new("R-09", "role is already vacant"));
                }
            }
            Some(p) => {
                let known = self.principals.get(&p);
                let kind = match (known, b.principal_kind) {
                    (Some(k), Some(given)) if k.kind != given => {
                        return Err(Rejection::new("R-08", "a principal's kind cannot change"))
                    }
                    (Some(k), _) => k.kind,
                    (None, Some(given)) => given,
                    (None, None) => {
                        return Err(Rejection::new(
                            "R-08",
                            "a new principal must declare its kind",
                        ))
                    }
                };
                if known.is_none() && b.principal_name.as_deref().unwrap_or("").trim().is_empty() {
                    return Err(Rejection::new("R-06", "a new principal needs a name"));
                }
                if let Some(name) = &b.principal_name {
                    if let Some((other, _)) = self.principal_by_name(name) {
                        if other != p {
                            return Err(Rejection::new(
                                "R-06",
                                format!("name {name} belongs to another principal"),
                            ));
                        }
                    }
                }
                if matches!(kind, PrincipalKind::Agent | PrincipalKind::ExternalModel)
                    && !matches!(b.bound_role, Role::Execution | Role::Adviser)
                {
                    return Err(Rejection::new(
                        "R-08",
                        format!(
                            "agents may hold only execution or adviser, not {}",
                            b.bound_role
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_queue_keep(&self, payload: &Payload, q: &QueueKeep) -> Result<(), Rejection> {
        let Some(id) = payload.subject_id else {
            // Creation: signalled, by execution.
            if payload.role != Role::Execution {
                return Err(Rejection::new("K-06", "execution queues keep work"));
            }
            if !self.agreement_covers(&q.asset_ref) {
                return Err(Rejection::new(
                    "K-01",
                    format!("no active keep agreement covers asset {}", q.asset_ref),
                ));
            }
            return Ok(());
        };
        let item = self
            .items
            .get(&id)
            .ok_or_else(|| Rejection::new("O-01", format!("work item {id} does not exist")))?;
        let ItemState::Keep(current) = item.state else {
            return Err(Rejection::new("K-06", "queue_keep on a build item"));
        };
        if q.from != Some(current) {
            return Err(Rejection::new(
                "K-06",
                format!("item is {current:?}, not {:?}", q.from),
            ));
        }
        use KeepState::*;
        let allowed = match (current, q.to) {
            (Signalled, Outlook | Planned | Firm) => true,
            (Outlook, Planned) => true,
            (Outlook, Firm) => q.urgent_safety,
            (Planned, Firm) => true,
            (Firm, Executing) => true,
            (Executing, Closed) => true,
            (Signalled | Outlook | Planned | Firm | Executing, Deferred | Reclassified) => true,
            _ => false,
        };
        if !allowed {
            return Err(Rejection::new(
                "K-06",
                format!("no keep transition {current:?} → {:?}", q.to),
            ));
        }
        let role_ok = match q.to {
            Reclassified => matches!(payload.role, Role::Execution | Role::Policy),
            _ => payload.role == Role::Execution,
        };
        if !role_ok {
            return Err(Rejection::new(
                "K-06",
                format!("{} may not move keep items to {:?}", payload.role, q.to),
            ));
        }
        if q.to == Executing && (item.adds_function || q.adds_function) {
            return Err(Rejection::new(
                "K-07",
                "item adds function; reclassify it to build",
            ));
        }
        if q.to == Deferred
            && self
                .last_displacement
                .get(&id)
                .is_none_or(|seq| *seq <= item.last_seq)
        {
            return Err(Rejection::new(
                "K-09",
                "deferring needs a displacement event on this item after its last transition",
            ));
        }
        if q.to == Closed && payload.evidence.is_empty() {
            return Err(Rejection::new(
                "K-06",
                "closing needs a completion artefact",
            ));
        }
        Ok(())
    }

    fn check_queue_build(&self, payload: &Payload, q: &QueueBuild) -> Result<(), Rejection> {
        if payload.role != Role::Policy {
            return Err(Rejection::new("B-03", "policy opens build items"));
        }
        if let Some(prog) = q.programme_ref {
            if !matches!(
                self.items.get(&prog).map(|i| i.state),
                Some(ItemState::Build(_))
            ) {
                return Err(Rejection::new("B-12", "programme_ref must be a build item"));
            }
        }
        Ok(())
    }

    fn check_gate(&self, payload: &Payload, g: &Gate) -> Result<(), Rejection> {
        let id = payload.subject_id.expect("validated");
        let item = self
            .items
            .get(&id)
            .ok_or_else(|| Rejection::new("O-01", format!("work item {id} does not exist")))?;
        let ItemState::Build(current) = item.state else {
            return Err(Rejection::new("K-06", "keep items never pass build gates"));
        };
        let expected = match g.gate {
            GateKind::Start => BuildState::Proposed,
            GateKind::Preferred => BuildState::Exploring,
            GateKind::Project => BuildState::Designing,
            GateKind::Handover => BuildState::Realising,
        };
        if current != expected {
            return Err(Rejection::new(
                "B-05",
                format!(
                    "{:?} gate needs the item in {expected:?}; it is {current:?}",
                    g.gate
                ),
            ));
        }
        let outcome_ok = match g.gate {
            GateKind::Start => matches!(
                g.outcome,
                GateOutcome::Go | GateOutcome::NoGo | GateOutcome::Hold | GateOutcome::Stop
            ),
            GateKind::Preferred => matches!(
                g.outcome,
                GateOutcome::Go | GateOutcome::NoGo | GateOutcome::Stop
            ),
            GateKind::Project => matches!(
                g.outcome,
                GateOutcome::Go | GateOutcome::Return | GateOutcome::Stop
            ),
            GateKind::Handover => matches!(g.outcome, GateOutcome::Go | GateOutcome::Stop),
        };
        if !outcome_ok {
            return Err(Rejection::new(
                "B-03",
                format!("{:?} is not an outcome of the {:?} gate", g.outcome, g.gate),
            ));
        }

        // Signers (B-02).
        let required = match g.gate {
            GateKind::Handover => Role::MandateHolder,
            _ => Role::Policy,
        };
        if payload.role != required {
            return Err(Rejection::new(
                "B-02",
                format!("{:?} gate is signed as {required}", g.gate),
            ));
        }
        if !g
            .signers
            .iter()
            .any(|s| s.principal == payload.signer && s.role == required)
        {
            return Err(Rejection::new(
                "B-02",
                "receipt signer is not among the gate signers",
            ));
        }
        if let Some(s) = g.signers.iter().find(|s| !self.holds(s.principal, s.role)) {
            return Err(Rejection::new(
                "B-02",
                format!("gate signer {} does not hold {}", s.principal, s.role),
            ));
        }

        // Evidence must not rest only on replicas or synthetic output (C-05).
        let solid = payload.evidence.iter().any(|ev| {
            self.artifacts
                .get(&ev.inner())
                .is_some_and(|a| !a.replica && a.tag != EvidenceTag::E)
        });
        if !solid {
            return Err(Rejection::new(
                "C-05",
                "gate evidence is only replicas or synthetic (E) material",
            ));
        }

        if g.outcome == GateOutcome::Go && g.gate != GateKind::Handover {
            let mc = g
                .means_check
                .ok_or_else(|| Rejection::new("M-06", "go needs a means_check"))?;
            let env = self
                .envelopes
                .get(&mc.envelope_ref)
                .ok_or_else(|| Rejection::new("M-01", "means_check envelope does not exist"))?;
            if env.envelope.queue != QueueKind::Build {
                return Err(Rejection::new(
                    "K-04",
                    "build gates draw on a build envelope",
                ));
            }
            let computed = self.available_for(mc.envelope_ref, id).unwrap_or(0);
            if mc.available > computed {
                return Err(Rejection::new(
                    "M-05",
                    format!(
                        "claimed available {} exceeds envelope availability {computed}",
                        mc.available
                    ),
                ));
            }
            match g.gate {
                GateKind::Start => {
                    if let Some(bp) = self.start_funding_share_bp {
                        if u128::from(mc.available) * 10_000
                            < u128::from(mc.required) * u128::from(bp)
                        {
                            return Err(Rejection::new(
                                "M-06",
                                "funding sight below start_funding_share",
                            ));
                        }
                    }
                }
                _ => {
                    if mc.available < mc.required {
                        return Err(Rejection::new(
                            "B-02",
                            "means available do not cover the requirement",
                        ));
                    }
                }
            }
        }

        if g.gate == GateKind::Handover && g.outcome == GateOutcome::Go {
            let d = g.discharge.expect("validated");
            if d.role != Role::Execution || !self.holds(d.principal, Role::Execution) {
                return Err(Rejection::new(
                    "B-10",
                    "discharge must name the execution role holder",
                ));
            }
            let keep = g.keep.expect("validated");
            if self.policies.get(&keep.agreement_ref) != Some(&PolicySubKind::Agreement) {
                return Err(Rejection::new(
                    "B-10",
                    "keep.agreement_ref is not an adopted keep agreement",
                ));
            }
            if let Some(env) = keep.envelope_ref {
                if self.envelopes.get(&env).map(|e| e.envelope.queue) != Some(QueueKind::Keep) {
                    return Err(Rejection::new(
                        "B-10",
                        "keep.envelope_ref is not a keep envelope",
                    ));
                }
            }
            if !payload.self_certified && g.signers.iter().any(|s| s.principal == d.principal) {
                return Err(Rejection::new(
                    "B-14.7",
                    "execution does not sign its own discharge",
                ));
            }
        }
        Ok(())
    }

    fn check_means(&self, payload: &Payload, m: &Means) -> Result<(), Rejection> {
        for r in [m.gate_ref, m.agreement_ref].into_iter().flatten() {
            self.check_authority(r)?;
        }
        let env = self
            .envelopes
            .get(&m.envelope_ref)
            .ok_or_else(|| Rejection::new("M-01", "envelope does not exist"))?;
        if m.unit != env.envelope.unit {
            return Err(Rejection::new(
                "O-09",
                format!(
                    "unit {} is not the envelope unit {}",
                    m.unit, env.envelope.unit
                ),
            ));
        }
        let role_ok = match m.to_state {
            MeansTarget::Spent => payload.role == Role::Execution,
            MeansTarget::Bound => matches!(
                payload.role,
                Role::Policy | Role::Execution | Role::Continuity
            ),
            _ => matches!(payload.role, Role::Policy | Role::Continuity),
        };
        if !role_ok {
            return Err(Rejection::new(
                "R-01",
                format!("{} may not move means to {:?}", payload.role, m.to_state),
            ));
        }
        let item =
            match m.item_ref {
                Some(id) => Some(self.items.get(&id).ok_or_else(|| {
                    Rejection::new("O-01", format!("work item {id} does not exist"))
                })?),
                None => None,
            };
        let (item_committed, item_bound) = m
            .item_ref
            .and_then(|id| env.per_item.get(&id).copied())
            .unwrap_or((0, 0));

        use MeansState as S;
        use MeansTarget as T;
        match (m.from_state, m.to_state) {
            (S::Reserved, T::Committed) => {
                let item =
                    item.ok_or_else(|| Rejection::new("M-05", "commitment needs an item_ref"))?;
                match (m.gate_ref, m.agreement_ref) {
                    (Some(g), _) => {
                        let rec = self.gates.get(&g).ok_or_else(|| {
                            Rejection::new("M-05", "gate_ref is not a gate receipt")
                        })?;
                        if rec.item != item.id || rec.outcome != GateOutcome::Go {
                            return Err(Rejection::new(
                                "M-05",
                                "gate_ref must be a go gate on the same item",
                            ));
                        }
                    }
                    (None, Some(a)) => {
                        if self.policies.get(&a) != Some(&PolicySubKind::Agreement) {
                            return Err(Rejection::new(
                                "M-05",
                                "agreement_ref is not an agreement",
                            ));
                        }
                    }
                    (None, None) => {
                        return Err(Rejection::new(
                            "M-05",
                            "reserved → committed needs a gate or agreement",
                        ))
                    }
                }
                if env.envelope.buckets.reserved < m.amount {
                    return Err(Rejection::new("M-01", "not enough reserved means"));
                }
            }
            (S::Committed, T::Reserved) => {
                if item.is_none() || item_committed < m.amount {
                    return Err(Rejection::new(
                        "M-05",
                        "release needs an item with enough committed means",
                    ));
                }
            }
            (S::Committed, T::Bound) => {
                let item =
                    item.ok_or_else(|| Rejection::new("M-05", "binding needs an item_ref"))?;
                if let ItemState::Build(s) = item.state {
                    if !matches!(s, BuildState::Realising | BuildState::HandedOver) {
                        return Err(Rejection::new(
                            "M-05",
                            "build means bind only after the project gate",
                        ));
                    }
                }
                if item_committed < m.amount {
                    return Err(Rejection::new(
                        "M-01",
                        "not enough committed means on the item",
                    ));
                }
            }
            (S::Reserved, T::Bound) => {
                let a = m.agreement_ref.ok_or_else(|| {
                    Rejection::new("M-05", "reserved → bound only through a keep agreement")
                })?;
                if self.policies.get(&a) != Some(&PolicySubKind::Agreement)
                    || env.envelope.queue != QueueKind::Keep
                {
                    return Err(Rejection::new(
                        "M-05",
                        "reserved → bound only for a keep envelope under an agreement",
                    ));
                }
                if env.envelope.buckets.reserved < m.amount {
                    return Err(Rejection::new("M-01", "not enough reserved means"));
                }
            }
            (S::Committed, T::Spent) => {
                if item.is_none() || item_committed < m.amount {
                    return Err(Rejection::new(
                        "M-02",
                        "spend needs an item with enough committed means",
                    ));
                }
            }
            (S::Bound, T::Spent) => {
                let have = if item.is_some() {
                    item_bound
                } else {
                    env.envelope.buckets.bound
                };
                if have < m.amount {
                    return Err(Rejection::new("M-01", "not enough bound means"));
                }
            }
            (from, to) => {
                return Err(Rejection::new(
                    "M-11",
                    format!("no means movement {from:?} → {to:?}"),
                ));
            }
        }
        Ok(())
    }

    fn check_policy(&self, payload: &Payload, p: &Policy) -> Result<(), Rejection> {
        for s in &p.signers {
            if !self.holds(s.principal, s.role) {
                return Err(Rejection::new(
                    "O-04",
                    format!("policy signer {} does not hold {}", s.principal, s.role),
                ));
            }
        }
        match p.sub_kind {
            PolicySubKind::Envelope => {
                if payload.role != Role::Continuity {
                    return Err(Rejection::new("O-09", "continuity creates envelopes"));
                }
                let e = p.envelope.as_ref().expect("validated");
                if self.exists(e.id) {
                    return Err(Rejection::new("O-09", "envelope id already used"));
                }
            }
            PolicySubKind::Agreement => {
                for role in [Role::Continuity, Role::Policy, Role::Execution] {
                    if !p.signers.iter().any(|s| s.role == role) {
                        return Err(Rejection::new(
                            "K-01",
                            format!("agreement lacks a {role} signer"),
                        ));
                    }
                }
            }
            PolicySubKind::Rule | PolicySubKind::Framework => {
                if payload.role != Role::Policy {
                    return Err(Rejection::new("O-04", "policy adopts rules and frameworks"));
                }
            }
        }
        for id in &p.adopts {
            if !self.advice.contains(id) {
                return Err(Rejection::new("A-02", format!("{id} is not advice")));
            }
        }
        for id in [p.supersedes, p.reaffirms].into_iter().flatten() {
            if !self.policies.contains_key(&id) {
                return Err(Rejection::new("V-03", format!("{id} is not a policy")));
            }
        }
        if let Some(body) = p.body_ref {
            if !self.artifacts.contains_key(&body.inner()) {
                return Err(Rejection::new(
                    "O-04",
                    "body_ref is not an admitted artefact",
                ));
            }
        }
        Ok(())
    }

    fn check_event(&self, payload: &Payload, e: &Event) -> Result<(), Rejection> {
        if let Some(subject) = payload.subject_id {
            if !self.exists(subject) {
                return Err(Rejection::new(
                    "F-01",
                    format!("subject {subject} does not exist"),
                ));
            }
        }
        if e.event == EventKind::ReportBackResolved {
            if let Some(subject) = payload.subject_id {
                let is_ticket = self
                    .events
                    .iter()
                    .any(|r| r.id == subject && r.ticket.is_some());
                if is_ticket && !self.open_tickets().iter().any(|t| t.id == subject) {
                    return Err(Rejection::new("A-07", "ticket is already resolved"));
                }
            }
        }
        self.check_refs(&e.refs, "F-01")
    }

    /// Lifecycle transitions per `event_id` (Stage 1, NAP-corpus #13).
    ///
    /// - New `event_id`: may start at `draft`, `signed` or `sealed` (sealed-at-creation
    ///   still needs a valid signature, ENV-04).
    /// - ENV-07 a `sealed` object never changes again (same pattern as K-01).
    /// - ENV-05 `draft` → `draft` | `signed`; `signed` → `sealed` only, and sealing
    ///   changes nothing but `lifecycle`. `signed` is one-way; `draft` → `sealed` is refused.
    /// - `anchored` is refused statelessly (ENV-08).
    fn check_envelope(&self, e: &EventEnvelope) -> Result<(), Rejection> {
        self.check_envelope_transition(e)?;
        self.check_envelope_cid(e)
    }

    /// ENV-09: one CID, one content object. A second event with the same payload
    /// bytes is kept as a reference: it must name the first owner in `duplicate_of`.
    /// An envelope whose CID is new carries no `duplicate_of`.
    fn check_envelope_cid(&self, e: &EventEnvelope) -> Result<(), Rejection> {
        let expected = self
            .envelope_cids
            .get(&e.cid)
            .filter(|owner| **owner != e.event_id);
        match (expected, &e.duplicate_of) {
            (None, None) => Ok(()),
            (Some(owner), Some(d)) if d == owner => Ok(()),
            (Some(owner), _) => Err(Rejection::new(
                "ENV-09",
                format!(
                    "cid {} is already carried by event_id {owner}; set duplicate_of to it",
                    e.cid
                ),
            )),
            (None, Some(d)) => Err(Rejection::new(
                "ENV-09",
                format!(
                    "duplicate_of {d} given, but cid {} is not on the chain under another event_id",
                    e.cid
                ),
            )),
        }
    }

    fn check_envelope_transition(&self, e: &EventEnvelope) -> Result<(), Rejection> {
        let Some(prev) = self.envelope_events.get(&e.event_id) else {
            return Ok(());
        };
        let id = &e.event_id;
        match (prev.lifecycle, e.lifecycle) {
            (Lifecycle::Sealed | Lifecycle::Anchored, _) => Err(Rejection::new(
                "ENV-07",
                format!("event_id {id} is sealed; it cannot change"),
            )),
            (Lifecycle::Draft, Lifecycle::Draft | Lifecycle::Signed) => Ok(()),
            (Lifecycle::Draft, _) => Err(Rejection::new(
                "ENV-05",
                format!("event_id {id} is a draft; sign it before sealing"),
            )),
            (Lifecycle::Signed, Lifecycle::Sealed) => {
                let mut unsealed = e.clone();
                unsealed.lifecycle = Lifecycle::Signed;
                if &unsealed != prev {
                    return Err(Rejection::new(
                        "ENV-05",
                        format!("sealing event_id {id} may change only its lifecycle"),
                    ));
                }
                Ok(())
            }
            (Lifecycle::Signed, _) => Err(Rejection::new(
                "ENV-05",
                format!("event_id {id} is signed; the only next state is sealed"),
            )),
        }
    }

    fn check_correction(&self, payload: &Payload, c: &Correction) -> Result<(), Rejection> {
        if !self.exists(c.corrects) {
            return Err(Rejection::new("C-06", "corrected object does not exist"));
        }
        if let Some(origin) = self.origin_role.get(&c.corrects) {
            if role_rank(payload.role) < role_rank(*origin) {
                return Err(Rejection::new(
                    "C-06",
                    format!(
                        "{} cannot correct an object signed as {origin}",
                        payload.role
                    ),
                ));
            }
        }
        if let Some(r) = c.replacement {
            if !self.exists(r) {
                return Err(Rejection::new("C-06", "replacement does not exist"));
            }
        }
        Ok(())
    }

    fn check_review(&self, payload: &Payload, r: &Review) -> Result<(), Rejection> {
        if let Some(subject) = payload.subject_id {
            if !self.exists(subject) {
                return Err(Rejection::new(
                    "V-01",
                    format!("review subject {subject} does not exist"),
                ));
            }
        }
        if let Some(t) = &r.trigger {
            if !self.exists(t.trigger_ref) {
                return Err(Rejection::new("V-04", "trigger ref does not exist"));
            }
        }
        if let Some(a) = r.artefact_ref {
            if !self.artifacts.contains_key(&a.inner()) {
                return Err(Rejection::new(
                    "V-02",
                    "artefact_ref is not an admitted artefact",
                ));
            }
        }
        Ok(())
    }

    /// Fold an already-checked receipt into the state.
    pub fn apply(&mut self, header: &ObjectHeader, payload: &Payload) {
        let id = header.id;
        if self.project_id.is_none() {
            self.project_id = Some(header.project_id);
        }
        self.receipts.insert(id, payload.body.kind());
        let seq = self.seq;
        self.seq += 1;
        self.origin_role.insert(id, payload.role);
        match &payload.body {
            Body::Ingest(i) => {
                if matches!(
                    i.outcome,
                    IngestOutcome::Admit | IngestOutcome::AdmitRestricted
                ) {
                    if let Some(aid) = i.artifact_id {
                        self.origin_role.insert(aid.inner(), payload.role);
                        self.artifacts.insert(
                            aid.inner(),
                            ArtifactInfo {
                                digest: i.digest.clone(),
                                replica: i.replica,
                                tag: i.evidence_tag,
                            },
                        );
                        self.digests.insert(i.digest.clone());
                    }
                }
            }
            Body::RoleBind(b) => match b.principal {
                None => {
                    self.roles.remove(&b.bound_role);
                }
                Some(p) => {
                    let entry = self.principals.entry(p).or_insert_with(|| PrincipalInfo {
                        name: b.principal_name.clone().unwrap_or_default(),
                        kind: b.principal_kind.unwrap_or(PrincipalKind::Human),
                    });
                    if let Some(name) = &b.principal_name {
                        entry.name = name.clone();
                    }
                    let holders = self.roles.entry(b.bound_role).or_default();
                    if matches!(b.bound_role, Role::Continuity | Role::Execution) {
                        holders.clear();
                    }
                    if !holders.contains(&p) {
                        holders.push(p);
                    }
                }
            },
            Body::QueueKeep(q) => match payload.subject_id {
                None => {
                    self.items.insert(
                        id,
                        WorkItem {
                            id,
                            state: ItemState::Keep(KeepState::Signalled),
                            asset_ref: q.asset_ref.clone(),
                            lead: q.lead.or(Some(payload.signer)),
                            adds_function: q.adds_function,
                            last_seq: seq,
                        },
                    );
                }
                Some(item) => {
                    if let Some(w) = self.items.get_mut(&item) {
                        w.state = ItemState::Keep(q.to);
                        w.adds_function |= q.adds_function;
                        w.last_seq = seq;
                    }
                }
            },
            Body::QueueBuild(q) => {
                self.items.insert(
                    id,
                    WorkItem {
                        id,
                        state: ItemState::Build(BuildState::Proposed),
                        asset_ref: q.asset_ref.clone(),
                        lead: q.lead.or(Some(payload.signer)),
                        adds_function: true,
                        last_seq: seq,
                    },
                );
            }
            Body::Gate(g) => {
                let item = payload.subject_id.expect("validated");
                self.gates.insert(
                    id,
                    GateRecord {
                        item,
                        gate: g.gate,
                        outcome: g.outcome,
                    },
                );
                let next = match (g.gate, g.outcome) {
                    (_, GateOutcome::Stop | GateOutcome::NoGo) => Some(BuildState::Stopped),
                    (_, GateOutcome::Hold) => None,
                    (GateKind::Project, GateOutcome::Return) => Some(BuildState::Exploring),
                    (GateKind::Start, GateOutcome::Go) => Some(BuildState::Exploring),
                    (GateKind::Preferred, GateOutcome::Go) => Some(BuildState::Designing),
                    (GateKind::Project, GateOutcome::Go) => Some(BuildState::Realising),
                    (GateKind::Handover, GateOutcome::Go) => Some(BuildState::HandedOver),
                    _ => None,
                };
                if let Some(w) = self.items.get_mut(&item) {
                    if let Some(next) = next {
                        w.state = ItemState::Build(next);
                    }
                    w.last_seq = seq;
                }
            }
            Body::Means(m) => {
                if let Some(env) = self.envelopes.get_mut(&m.envelope_ref) {
                    let b = &mut env.envelope.buckets;
                    match m.from_state {
                        MeansState::Reserved => b.reserved = b.reserved.saturating_sub(m.amount),
                        MeansState::Committed => b.committed = b.committed.saturating_sub(m.amount),
                        MeansState::Bound => b.bound = b.bound.saturating_sub(m.amount),
                    }
                    match m.to_state {
                        MeansTarget::Reserved => b.reserved += m.amount,
                        MeansTarget::Committed => b.committed += m.amount,
                        MeansTarget::Bound => b.bound += m.amount,
                        MeansTarget::Spent => env.spent += m.amount,
                    }
                    if let Some(item) = m.item_ref {
                        let slot = env.per_item.entry(item).or_insert((0, 0));
                        match m.from_state {
                            MeansState::Committed => slot.0 = slot.0.saturating_sub(m.amount),
                            MeansState::Bound => slot.1 = slot.1.saturating_sub(m.amount),
                            MeansState::Reserved => {}
                        }
                        match m.to_state {
                            MeansTarget::Committed => slot.0 += m.amount,
                            MeansTarget::Bound => slot.1 += m.amount,
                            MeansTarget::Reserved | MeansTarget::Spent => {}
                        }
                    }
                }
            }
            Body::Advice(_) => {
                self.advice.insert(id);
            }
            Body::Policy(p) => {
                self.policies.insert(id, p.sub_kind);
                if let Some(old) = p.supersedes {
                    self.agreements.remove(&old);
                }
                if p.sub_kind == PolicySubKind::Agreement {
                    self.agreements.insert(id, p.asset_scope.clone());
                }
                if let Some(e) = &p.envelope {
                    self.origin_role.insert(e.id, payload.role);
                    self.envelopes.insert(
                        e.id,
                        EnvelopeState {
                            envelope: e.clone(),
                            spent: 0,
                            per_item: BTreeMap::new(),
                        },
                    );
                }
                if let Some(bp) = p.start_funding_share_bp {
                    self.start_funding_share_bp = Some(bp);
                }
            }
            Body::Event(e) => {
                if let (EventKind::Displacement, Some(subject)) = (e.event, payload.subject_id) {
                    self.last_displacement.insert(subject, seq);
                }
                self.events.push(EventRecord {
                    id,
                    event: e.event,
                    subject: payload.subject_id,
                    impact: e.impact.clone(),
                    next_decision: e.next_decision.clone(),
                    detail: e.detail.clone(),
                    ticket: e.ticket.clone(),
                });
            }
            Body::EventEnvelope(e) => {
                self.envelope_cids
                    .entry(e.cid.clone())
                    .or_insert_with(|| e.event_id.clone());
                self.envelope_events.insert(e.event_id.clone(), e.clone());
            }
            Body::Correction(_) | Body::Review(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apparatus_types::{ArtifactId, Provenance};

    struct Harness {
        state: State,
        project: ProjectId,
        clock: u64,
        owner: PrincipalId,
    }

    impl Harness {
        /// Solo project: owner holds continuity, policy, execution, mandate_holder.
        fn solo() -> Self {
            let mut h = Harness {
                state: State::new(),
                project: ProjectId::new(),
                clock: 1_000,
                owner: PrincipalId::new(),
            };
            for role in [
                Role::Continuity,
                Role::Policy,
                Role::Execution,
                Role::MandateHolder,
            ] {
                h.ok(
                    h.owner,
                    Role::Continuity,
                    None,
                    vec![],
                    bind(role, h.owner, "owner", PrincipalKind::Human),
                );
            }
            h.agreement(&["*"]);
            h
        }

        /// Solo roles but no keep agreement yet.
        fn solo_without_agreement() -> Self {
            let mut h = Harness {
                state: State::new(),
                project: ProjectId::new(),
                clock: 1_000,
                owner: PrincipalId::new(),
            };
            for role in [Role::Continuity, Role::Policy, Role::Execution] {
                h.ok(
                    h.owner,
                    Role::Continuity,
                    None,
                    vec![],
                    bind(role, h.owner, "owner", PrincipalKind::Human),
                );
            }
            h
        }

        fn agreement(&mut self, assets: &[&str]) -> ObjectId {
            self.agreement_superseding(assets, None)
        }

        fn agreement_superseding(
            &mut self,
            assets: &[&str],
            supersedes: Option<ObjectId>,
        ) -> ObjectId {
            let signers = [Role::Continuity, Role::Policy, Role::Execution]
                .into_iter()
                .map(|role| Signer {
                    principal: self.owner,
                    role,
                })
                .collect();
            self.ok(
                self.owner,
                Role::Continuity,
                None,
                vec![],
                Body::Policy(Policy {
                    sub_kind: PolicySubKind::Agreement,
                    body_ref: None,
                    signers,
                    adopts: vec![],
                    advice_response: vec![],
                    supersedes,
                    reaffirms: None,
                    envelope: None,
                    asset_scope: assets.iter().map(|a| a.to_string()).collect(),
                    start_funding_share_bp: None,
                }),
            )
        }

        fn keep_on(&mut self, asset: &str) -> Result<ObjectId, Rejection> {
            self.try_(
                self.owner,
                Role::Execution,
                None,
                vec![],
                Body::QueueKeep(QueueKeep {
                    from: None,
                    to: KeepState::Signalled,
                    asset_ref: asset.into(),
                    lead: None,
                    adds_function: false,
                    urgent_safety: false,
                }),
            )
        }

        fn move_keep(
            &mut self,
            item: ObjectId,
            from: KeepState,
            to: KeepState,
        ) -> Result<ObjectId, Rejection> {
            self.try_(
                self.owner,
                Role::Execution,
                Some(item),
                vec![],
                Body::QueueKeep(QueueKeep {
                    from: Some(from),
                    to,
                    asset_ref: "demo".into(),
                    lead: None,
                    adds_function: false,
                    urgent_safety: false,
                }),
            )
        }

        fn failure(&mut self, kind: EventKind, subject: ObjectId) -> ObjectId {
            self.ok(
                self.owner,
                Role::Execution,
                Some(subject),
                vec![],
                Body::Event(Event {
                    event: kind,
                    impact: Some("test".into()),
                    next_decision: Some(NextDecision {
                        role: Role::Policy,
                        by: 10,
                        options: vec![],
                    }),
                    refs: vec![],
                    detail: None,
                    ticket: None,
                }),
            )
        }

        fn header(&mut self, signer: PrincipalId) -> ObjectHeader {
            self.clock += 1;
            ObjectHeader {
                id: ObjectId::new_v7(),
                created_at: self.clock,
                project_id: self.project,
                classification: Classification::Internal,
                provenance: Some(Provenance {
                    source: "test".into(),
                    creator: signer,
                }),
                correlation_id: None,
            }
        }

        fn payload(
            &self,
            signer: PrincipalId,
            role: Role,
            subject: Option<ObjectId>,
            evidence: Vec<ArtifactId>,
            body: Body,
        ) -> Payload {
            Payload {
                rws: RWS_VERSION.into(),
                subject_id: subject,
                signer,
                role,
                authority_ref: None,
                evidence,
                self_certified: true,
                body,
            }
        }

        fn try_(
            &mut self,
            signer: PrincipalId,
            role: Role,
            subject: Option<ObjectId>,
            evidence: Vec<ArtifactId>,
            body: Body,
        ) -> Result<ObjectId, Rejection> {
            let h = self.header(signer);
            let p = self.payload(signer, role, subject, evidence, body);
            self.state.admit(&h, &p).map(|_| h.id)
        }

        fn ok(
            &mut self,
            signer: PrincipalId,
            role: Role,
            subject: Option<ObjectId>,
            evidence: Vec<ArtifactId>,
            body: Body,
        ) -> ObjectId {
            self.try_(signer, role, subject, evidence, body)
                .expect("admitted")
        }

        fn ingest(&mut self, bytes: &[u8], tag: EvidenceTag) -> ArtifactId {
            let aid = ArtifactId::new();
            let digest = format!(
                "{:064x}",
                u128::from_le_bytes({
                    let mut b = [0u8; 16];
                    for (i, x) in bytes.iter().take(16).enumerate() {
                        b[i] = *x;
                    }
                    b
                })
            );
            self.ok(
                self.owner,
                Role::Execution,
                None,
                vec![],
                Body::Ingest(Ingest {
                    outcome: IngestOutcome::Admit,
                    purpose: "test".into(),
                    artifact_id: Some(aid),
                    digest,
                    size_bytes: bytes.len() as u64,
                    media_type: "text/plain".into(),
                    source_locator: "mem".into(),
                    evidence_tag: tag,
                    reason: None,
                    replica: false,
                }),
            );
            aid
        }

        fn keep_item(&mut self) -> ObjectId {
            self.ok(
                self.owner,
                Role::Execution,
                None,
                vec![],
                Body::QueueKeep(QueueKeep {
                    from: None,
                    to: KeepState::Signalled,
                    asset_ref: "demo".into(),
                    lead: None,
                    adds_function: false,
                    urgent_safety: false,
                }),
            )
        }

        fn build_item(&mut self) -> ObjectId {
            self.ok(
                self.owner,
                Role::Policy,
                None,
                vec![],
                Body::QueueBuild(QueueBuild {
                    asset_ref: "demo".into(),
                    lead: None,
                    adds_function: true,
                    programme_ref: None,
                }),
            )
        }

        fn envelope(
            &mut self,
            queue: QueueKind,
            ceiling: u64,
            overprogramming: bool,
        ) -> Result<ObjectId, Rejection> {
            let id = ObjectId::new_v7();
            self.try_(
                self.owner,
                Role::Continuity,
                None,
                vec![],
                Body::Policy(Policy {
                    sub_kind: PolicySubKind::Envelope,
                    body_ref: None,
                    signers: vec![Signer {
                        principal: self.owner,
                        role: Role::Continuity,
                    }],
                    adopts: vec![],
                    advice_response: vec![],
                    supersedes: None,
                    reaffirms: None,
                    envelope: Some(Envelope {
                        id,
                        means_kind: "money".into(),
                        unit: "EUR".into(),
                        period: Period {
                            from_ms: 0,
                            until_ms: 10,
                        },
                        ceiling,
                        buckets: Buckets {
                            bound: 0,
                            committed: 0,
                            reserved: ceiling,
                        },
                        queue,
                        carry_over: CarryOver::Full,
                        overprogramming,
                    }),
                    asset_scope: vec![],
                    start_funding_share_bp: None,
                }),
            )?;
            Ok(id)
        }

        fn gate(
            &mut self,
            item: ObjectId,
            gate: GateKind,
            outcome: GateOutcome,
            evidence: Vec<ArtifactId>,
            mc: Option<MeansCheck>,
        ) -> Result<ObjectId, Rejection> {
            let role = if gate == GateKind::Handover {
                Role::MandateHolder
            } else {
                Role::Policy
            };
            self.try_(
                self.owner,
                role,
                Some(item),
                evidence,
                Body::Gate(Gate {
                    gate,
                    outcome,
                    signers: vec![Signer {
                        principal: self.owner,
                        role,
                    }],
                    means_check: mc,
                    discharge: None,
                    keep: None,
                    valid_until: None,
                }),
            )
        }
    }

    fn bind(role: Role, p: PrincipalId, name: &str, kind: PrincipalKind) -> Body {
        Body::RoleBind(RoleBind {
            bound_role: role,
            principal: Some(p),
            principal_name: Some(name.into()),
            principal_kind: Some(kind),
        })
    }

    #[test]
    fn chain_must_start_with_continuity_bootstrap() {
        let mut h = Harness {
            state: State::new(),
            project: ProjectId::new(),
            clock: 1,
            owner: PrincipalId::new(),
        };
        let r = h.try_(
            h.owner,
            Role::Execution,
            None,
            vec![],
            Body::QueueKeep(QueueKeep {
                from: None,
                to: KeepState::Signalled,
                asset_ref: "x".into(),
                lead: None,
                adds_function: false,
                urgent_safety: false,
            }),
        );
        assert_eq!(r.unwrap_err().rule, "R-09");
    }

    #[test]
    fn agent_cannot_be_bound_to_policy_or_continuity() {
        let mut h = Harness::solo();
        let agent = PrincipalId::new();
        for role in [Role::Policy, Role::Continuity, Role::MandateHolder] {
            let r = h.try_(
                h.owner,
                Role::Continuity,
                None,
                vec![],
                bind(role, agent, "bot", PrincipalKind::Agent),
            );
            assert_eq!(r.unwrap_err().rule, "R-08", "{role}");
        }
        h.ok(
            h.owner,
            Role::Continuity,
            None,
            vec![],
            bind(Role::Execution, agent, "bot", PrincipalKind::Agent),
        );
    }

    #[test]
    fn keep_item_never_passes_a_build_gate() {
        let mut h = Harness::solo();
        let ev = h.ingest(b"dossier", EvidenceTag::A);
        let item = h.keep_item();
        let r = h.gate(item, GateKind::Start, GateOutcome::Go, vec![ev], None);
        assert_eq!(r.unwrap_err().rule, "K-06");
    }

    #[test]
    fn gate_needs_signers_and_evidence() {
        let mut h = Harness::solo();
        let item = h.build_item();
        let no_ev = h.gate(item, GateKind::Start, GateOutcome::NoGo, vec![], None);
        assert_eq!(no_ev.unwrap_err().rule, "B-02");
        let ev = h.ingest(b"start doc", EvidenceTag::A);
        let r = h.try_(
            h.owner,
            Role::Policy,
            Some(item),
            vec![ev],
            Body::Gate(Gate {
                gate: GateKind::Start,
                outcome: GateOutcome::NoGo,
                signers: vec![],
                means_check: None,
                discharge: None,
                keep: None,
                valid_until: None,
            }),
        );
        assert_eq!(r.unwrap_err().rule, "B-02");
    }

    #[test]
    fn build_gates_follow_the_table_and_check_means() {
        let mut h = Harness::solo();
        let ev = h.ingest(b"start doc", EvidenceTag::A);
        let env = h.envelope(QueueKind::Build, 100, false).unwrap();
        let item = h.build_item();
        // Skipping a phase is refused.
        assert_eq!(
            h.gate(item, GateKind::Project, GateOutcome::Go, vec![ev], None)
                .unwrap_err()
                .rule,
            "B-05"
        );
        // Go without means check is refused.
        assert_eq!(
            h.gate(item, GateKind::Start, GateOutcome::Go, vec![ev], None)
                .unwrap_err()
                .rule,
            "M-06"
        );
        // Overclaiming availability is refused.
        let over = MeansCheck {
            envelope_ref: env,
            required: 80,
            available: 150,
        };
        assert_eq!(
            h.gate(item, GateKind::Start, GateOutcome::Go, vec![ev], Some(over))
                .unwrap_err()
                .rule,
            "M-05"
        );
        let mc = MeansCheck {
            envelope_ref: env,
            required: 80,
            available: 100,
        };
        let g1 = h
            .gate(item, GateKind::Start, GateOutcome::Go, vec![ev], Some(mc))
            .unwrap();
        assert_eq!(
            h.state.item(item).unwrap().state,
            ItemState::Build(BuildState::Exploring)
        );
        // Preferred gate needs 100%.
        let short = MeansCheck {
            envelope_ref: env,
            required: 120,
            available: 100,
        };
        assert_eq!(
            h.gate(
                item,
                GateKind::Preferred,
                GateOutcome::Go,
                vec![ev],
                Some(short)
            )
            .unwrap_err()
            .rule,
            "B-02"
        );
        // Commit via the start gate, then spending from reserved is refused.
        h.ok(
            h.owner,
            Role::Policy,
            None,
            vec![],
            Body::Means(Means {
                envelope_ref: env,
                from_state: MeansState::Reserved,
                to_state: MeansTarget::Committed,
                amount: 10,
                unit: "EUR".into(),
                item_ref: Some(item),
                gate_ref: Some(g1),
                agreement_ref: None,
            }),
        );
        assert_eq!(
            h.state.envelope(env).unwrap().envelope.buckets.committed,
            10
        );
    }

    #[test]
    fn handover_needs_discharge_and_keep_agreement() {
        let mut h = Harness::solo();
        let ev = h.ingest(b"report", EvidenceTag::A);
        let item = h.build_item();
        // Force the item into realising for this test.
        h.state.items.get_mut(&item).unwrap().state = ItemState::Build(BuildState::Realising);
        let r = h.gate(item, GateKind::Handover, GateOutcome::Go, vec![ev], None);
        assert_eq!(r.unwrap_err().rule, "B-10");
    }

    #[test]
    fn spending_from_reserved_is_refused() {
        let mut h = Harness::solo();
        let env = h.envelope(QueueKind::Build, 100, false).unwrap();
        let r = h.try_(
            h.owner,
            Role::Execution,
            None,
            vec![],
            Body::Means(Means {
                envelope_ref: env,
                from_state: MeansState::Reserved,
                to_state: MeansTarget::Spent,
                amount: 5,
                unit: "EUR".into(),
                item_ref: None,
                gate_ref: None,
                agreement_ref: None,
            }),
        );
        assert_eq!(r.unwrap_err().rule, "M-02");
    }

    #[test]
    fn keep_envelope_cannot_overprogram() {
        let mut h = Harness::solo();
        assert_eq!(
            h.envelope(QueueKind::Keep, 100, true).unwrap_err().rule,
            "K-04"
        );
        h.envelope(QueueKind::Build, 100, true).unwrap();
    }

    #[test]
    fn advice_is_never_authority() {
        let mut h = Harness::solo();
        let advice = h.ok(
            h.owner,
            Role::Execution,
            None,
            vec![],
            Body::Advice(Advice {
                producer: h.owner,
                producer_kind: PrincipalKind::Human,
                summary: "build it".into(),
                inputs: vec![],
                addressed_to: None,
                respond_by: None,
            }),
        );
        let env = h.envelope(QueueKind::Build, 100, false).unwrap();
        let item = h.build_item();
        let r = h.try_(
            h.owner,
            Role::Policy,
            None,
            vec![],
            Body::Means(Means {
                envelope_ref: env,
                from_state: MeansState::Reserved,
                to_state: MeansTarget::Committed,
                amount: 5,
                unit: "EUR".into(),
                item_ref: Some(item),
                gate_ref: Some(advice),
                agreement_ref: None,
            }),
        );
        assert_eq!(r.unwrap_err().rule, "A-01");
        // Also as authority_ref on a gate.
        let ev = h.ingest(b"x", EvidenceTag::A);
        let hdr = h.header(h.owner);
        let mut p = h.payload(
            h.owner,
            Role::Policy,
            Some(item),
            vec![ev],
            Body::Gate(Gate {
                gate: GateKind::Start,
                outcome: GateOutcome::NoGo,
                signers: vec![Signer {
                    principal: h.owner,
                    role: Role::Policy,
                }],
                means_check: None,
                discharge: None,
                keep: None,
                valid_until: None,
            }),
        );
        p.authority_ref = Some(advice);
        assert_eq!(h.state.check(&hdr, &p).unwrap_err().rule, "A-01");
    }

    #[test]
    fn keep_transition_table_is_enforced() {
        let mut h = Harness::solo();
        let item = h.keep_item();
        let r = h.try_(
            h.owner,
            Role::Execution,
            Some(item),
            vec![],
            Body::QueueKeep(QueueKeep {
                from: Some(KeepState::Signalled),
                to: KeepState::Closed,
                asset_ref: "demo".into(),
                lead: None,
                adds_function: false,
                urgent_safety: false,
            }),
        );
        assert_eq!(r.unwrap_err().rule, "K-06");
        h.ok(
            h.owner,
            Role::Execution,
            Some(item),
            vec![],
            Body::QueueKeep(QueueKeep {
                from: Some(KeepState::Signalled),
                to: KeepState::Firm,
                asset_ref: "demo".into(),
                lead: None,
                adds_function: false,
                urgent_safety: false,
            }),
        );
        let r = h.try_(
            h.owner,
            Role::Execution,
            Some(item),
            vec![],
            Body::QueueKeep(QueueKeep {
                from: Some(KeepState::Firm),
                to: KeepState::Executing,
                asset_ref: "demo".into(),
                lead: None,
                adds_function: true,
                urgent_safety: false,
            }),
        );
        assert_eq!(r.unwrap_err().rule, "K-07");
    }

    #[test]
    fn failure_events_need_impact_and_next_decision() {
        let mut h = Harness::solo();
        let item = h.keep_item();
        let r = h.try_(
            h.owner,
            Role::Execution,
            Some(item),
            vec![],
            Body::Event(Event {
                event: EventKind::TargetDropped,
                impact: None,
                next_decision: None,
                refs: vec![],
                detail: None,
                ticket: None,
            }),
        );
        assert_eq!(r.unwrap_err().rule, "F-01");
        h.ok(
            h.owner,
            Role::Execution,
            Some(item),
            vec![],
            Body::Event(Event {
                event: EventKind::TargetDropped,
                impact: Some("date missed".into()),
                next_decision: Some(NextDecision {
                    role: Role::Policy,
                    by: 10,
                    options: vec![],
                }),
                refs: vec![],
                detail: None,
                ticket: None,
            }),
        );
        assert_eq!(h.state.open_failures().len(), 1);
    }

    #[test]
    fn duplicate_bytes_are_refused_and_header_rules_hold() {
        let mut h = Harness::solo();
        h.ingest(b"same", EvidenceTag::B);
        let hdr = h.header(h.owner);
        let p = h.payload(
            h.owner,
            Role::Execution,
            None,
            vec![],
            Body::Ingest(Ingest {
                outcome: IngestOutcome::Admit,
                purpose: "again".into(),
                artifact_id: Some(ArtifactId::new()),
                digest: h.state.artifacts().next().unwrap().1.digest.clone(),
                size_bytes: 4,
                media_type: "text/plain".into(),
                source_locator: "mem".into(),
                evidence_tag: EvidenceTag::B,
                reason: None,
                replica: false,
            }),
        );
        assert_eq!(h.state.check(&hdr, &p).unwrap_err().rule, "C-03");

        let mut zero = h.header(h.owner);
        zero.created_at = 0;
        assert_eq!(h.state.check(&zero, &p).unwrap_err().rule, "C-07");

        let mut restricted = h.header(h.owner);
        restricted.classification = Classification::Restricted;
        restricted.provenance = None;
        assert_eq!(h.state.check(&restricted, &p).unwrap_err().rule, "C-04");
    }

    #[test]
    fn unknown_event_kind_does_not_decode() {
        let v = serde_json::json!({
            "rws": "1.0", "kind": "event", "event": "vibe_shift", "subject_id": null,
            "signer": PrincipalId::new(), "role": "execution", "authority_ref": null,
            "impact": null, "next_decision": null, "detail": null
        });
        assert_eq!(decode_payload(&v).unwrap_err().rule, "X-05.1");
    }

    #[test]
    fn keep_needs_an_agreement_covering_the_asset() {
        let mut h = Harness::solo_without_agreement();
        assert_eq!(h.keep_on("demo").unwrap_err().rule, "K-01");
        let agr = h.agreement(&["demo"]);
        h.keep_on("demo").unwrap();
        assert_eq!(h.keep_on("other").unwrap_err().rule, "K-01");
        // Superseding the agreement with a narrower one removes coverage.
        h.agreement_superseding(&["bridge"], Some(agr));
        assert_eq!(h.keep_on("demo").unwrap_err().rule, "K-01");
        h.keep_on("bridge").unwrap();
        // The wildcard covers every asset.
        h.agreement(&["*"]);
        h.keep_on("anything").unwrap();
    }

    #[test]
    fn deferring_needs_a_fresh_displacement_event() {
        let mut h = Harness::solo();
        let item = h.keep_item();
        assert_eq!(
            h.move_keep(item, KeepState::Signalled, KeepState::Deferred)
                .unwrap_err()
                .rule,
            "K-09"
        );
        h.failure(EventKind::Displacement, item);
        h.move_keep(item, KeepState::Signalled, KeepState::Planned)
            .unwrap();
        // The displacement predates the latest transition: refused again.
        assert_eq!(
            h.move_keep(item, KeepState::Planned, KeepState::Deferred)
                .unwrap_err()
                .rule,
            "K-09"
        );
        h.failure(EventKind::Displacement, item);
        h.move_keep(item, KeepState::Planned, KeepState::Deferred)
            .unwrap();
    }

    #[test]
    fn corrector_must_rank_at_least_the_original_signer() {
        let mut h = Harness::solo();
        let env = h.envelope(QueueKind::Build, 100, false).unwrap();
        let correction = |c: ObjectId| {
            Body::Correction(Correction {
                corrects: c,
                reason: "typo".into(),
                replacement: None,
                removal: false,
            })
        };
        // Envelope was created by continuity; execution cannot correct it.
        let r = h.try_(h.owner, Role::Execution, Some(env), vec![], correction(env));
        assert_eq!(r.unwrap_err().rule, "C-06");
        h.ok(
            h.owner,
            Role::Continuity,
            Some(env),
            vec![],
            correction(env),
        );
        // An ingest signed by execution can be corrected by policy.
        let art = h.ingest(b"draft", EvidenceTag::C);
        h.ok(
            h.owner,
            Role::Policy,
            Some(art.inner()),
            vec![],
            correction(art.inner()),
        );
        assert_eq!(h.state.origin_role(art.inner()), Some(Role::Execution));
    }

    #[test]
    fn review_subject_must_exist() {
        let mut h = Harness::solo();
        let review = Body::Review(Review {
            review: ReviewKind::Condition,
            rhythm: Rhythm::Scheduled,
            step: None,
            artefact_ref: None,
            trigger: None,
        });
        let r = h.try_(
            h.owner,
            Role::Execution,
            Some(ObjectId::new_v7()),
            vec![],
            review.clone(),
        );
        assert_eq!(r.unwrap_err().rule, "V-01");
        let item = h.keep_item();
        h.ok(h.owner, Role::Execution, Some(item), vec![], review);
    }

    #[test]
    fn tickets_open_and_resolve_once() {
        let mut h = Harness::solo();
        let item = h.keep_item();
        let ticket = |event, t: Option<Ticket>| {
            Body::Event(Event {
                event,
                impact: None,
                next_decision: None,
                refs: vec![],
                detail: None,
                ticket: t,
            })
        };
        let t = Ticket {
            title: "pump does not start".into(),
            body: None,
            reporter: "qwen".into(),
        };
        // A ticket only rides on report_back.
        let r = h.try_(
            h.owner,
            Role::Execution,
            Some(item),
            vec![],
            ticket(EventKind::ConditionReported, Some(t.clone())),
        );
        assert_eq!(r.unwrap_err().rule, "A-07");
        let id = h.ok(
            h.owner,
            Role::Execution,
            Some(item),
            vec![],
            ticket(EventKind::ReportBack, Some(t)),
        );
        assert_eq!(h.state.open_tickets().len(), 1);
        h.ok(
            h.owner,
            Role::Execution,
            Some(id),
            vec![],
            ticket(EventKind::ReportBackResolved, None),
        );
        assert!(h.state.open_tickets().is_empty());
        assert_eq!(h.state.tickets().len(), 1);
        let again = h.try_(
            h.owner,
            Role::Execution,
            Some(id),
            vec![],
            ticket(EventKind::ReportBackResolved, None),
        );
        assert_eq!(again.unwrap_err().rule, "A-07");
    }

    // ---- Envelope Stage 1: lifecycle transitions (NAP-corpus #13) ----

    fn env(event_id: &str, payload: serde_json::Value, lifecycle: Lifecycle) -> EventEnvelope {
        let (payload_hash, cid) =
            crate::envelope::address(&crate::envelope::payload_bytes(&payload).unwrap());
        EventEnvelope {
            event_id: event_id.into(),
            unix_timestamp: 1_790_401_238,
            source: "sensor".into(),
            module: "dexos.test".into(),
            provenance: ProvenanceKind::Stated,
            evidence_tag: None,
            source_ref: None,
            payload,
            payload_hash,
            cid,
            duplicate_of: None,
            lifecycle,
            signature_scheme: SignatureScheme::Ed25519,
            signature: None,
        }
    }

    fn signed(mut e: EventEnvelope, key: &apparatus_crypto::Ed25519Key) -> EventEnvelope {
        if e.lifecycle == Lifecycle::Draft {
            e.lifecycle = Lifecycle::Signed;
        }
        crate::envelope::sign(&mut e, key).unwrap();
        e
    }

    fn put(h: &mut Harness, e: EventEnvelope) -> Result<ObjectId, Rejection> {
        h.try_(
            h.owner,
            Role::Execution,
            None,
            vec![],
            Body::EventEnvelope(e),
        )
    }

    #[test]
    fn envelope_walks_draft_signed_sealed() {
        let mut h = Harness::solo();
        let key = apparatus_crypto::Ed25519Key::generate().unwrap();
        let draft = env("e1", serde_json::json!({ "n": 1 }), Lifecycle::Draft);
        put(&mut h, draft.clone()).unwrap();
        // draft → draft: still editable.
        let redraft = env("e1", serde_json::json!({ "n": 2 }), Lifecycle::Draft);
        put(&mut h, redraft.clone()).unwrap();
        // draft → signed needs a signature the Ed25519 path accepts.
        let s = signed(redraft, &key);
        put(&mut h, s.clone()).unwrap();
        // signed → sealed changes only the lifecycle.
        let mut sealed = s.clone();
        sealed.lifecycle = Lifecycle::Sealed;
        put(&mut h, sealed).unwrap();
        assert_eq!(
            h.state.event_envelope("e1").unwrap().lifecycle,
            Lifecycle::Sealed
        );
    }

    #[test]
    fn draft_to_signed_needs_a_verifying_signature() {
        let mut h = Harness::solo();
        put(&mut h, env("e1", serde_json::json!({}), Lifecycle::Draft)).unwrap();
        // Claimed signed without a signature: ENV-04.
        let bare = env("e1", serde_json::json!({}), Lifecycle::Signed);
        assert_eq!(put(&mut h, bare).unwrap_err().rule, "ENV-04");
        // A forged Ed25519 signature: ENV-04.
        let key = apparatus_crypto::Ed25519Key::generate().unwrap();
        let mut forged = signed(env("e1", serde_json::json!({}), Lifecycle::Draft), &key);
        forged.signature.as_mut().unwrap().value = "00".repeat(64);
        assert_eq!(put(&mut h, forged).unwrap_err().rule, "ENV-04");
        // The ML-DSA-65 stub still verifies as true (Stage 2 replaces it under a new name).
        let mut stub = env("e1", serde_json::json!({}), Lifecycle::Signed);
        stub.signature_scheme = SignatureScheme::MlDsa65Stub;
        crate::envelope::sign(&mut stub, &apparatus_crypto::MlDsa65Stub::new(vec![1; 32])).unwrap();
        put(&mut h, stub).unwrap();
    }

    #[test]
    fn signed_is_one_way_and_sealing_changes_only_lifecycle() {
        let mut h = Harness::solo();
        let key = apparatus_crypto::Ed25519Key::generate().unwrap();
        let s = signed(
            env("e1", serde_json::json!({ "n": 1 }), Lifecycle::Draft),
            &key,
        );
        put(&mut h, s.clone()).unwrap();
        // signed → draft and signed → signed: ENV-05.
        let back = env("e1", serde_json::json!({ "n": 1 }), Lifecycle::Draft);
        assert_eq!(put(&mut h, back).unwrap_err().rule, "ENV-05");
        let resign = signed(
            env("e1", serde_json::json!({ "n": 9 }), Lifecycle::Draft),
            &key,
        );
        assert_eq!(put(&mut h, resign).unwrap_err().rule, "ENV-05");
        // signed → sealed with different content: ENV-05.
        let mut changed = signed(
            env("e1", serde_json::json!({ "n": 2 }), Lifecycle::Draft),
            &key,
        );
        changed.lifecycle = Lifecycle::Sealed;
        assert_eq!(put(&mut h, changed).unwrap_err().rule, "ENV-05");
        // draft → sealed skips signing: refused.
        put(&mut h, env("e2", serde_json::json!({}), Lifecycle::Draft)).unwrap();
        let mut skip = signed(env("e2", serde_json::json!({}), Lifecycle::Draft), &key);
        skip.lifecycle = Lifecycle::Sealed;
        assert_eq!(put(&mut h, skip).unwrap_err().rule, "ENV-05");
    }

    #[test]
    fn sealed_objects_never_change_and_anchoring_is_refused() {
        let mut h = Harness::solo();
        let key = apparatus_crypto::Ed25519Key::generate().unwrap();
        let mut sealed = signed(
            env("e1", serde_json::json!({ "n": 1 }), Lifecycle::Draft),
            &key,
        );
        sealed.lifecycle = Lifecycle::Sealed;
        put(&mut h, sealed.clone()).unwrap();
        // Any later receipt for the sealed event_id: ENV-07, whatever it carries.
        let mutation = signed(
            env("e1", serde_json::json!({ "n": 2 }), Lifecycle::Draft),
            &key,
        );
        assert_eq!(put(&mut h, mutation).unwrap_err().rule, "ENV-07");
        assert_eq!(put(&mut h, sealed.clone()).unwrap_err().rule, "ENV-07");
        // sealed → anchored is Stage 3: ENV-08 (stateless, before the sealed rule).
        let mut anchored = sealed;
        anchored.lifecycle = Lifecycle::Anchored;
        assert_eq!(put(&mut h, anchored.clone()).unwrap_err().rule, "ENV-08");
        let mut fresh = signed(env("fresh", serde_json::json!({}), Lifecycle::Draft), &key);
        fresh.lifecycle = Lifecycle::Anchored;
        assert_eq!(put(&mut h, fresh).unwrap_err().rule, "ENV-08");
    }

    #[test]
    fn provenance_needs_its_evidence() {
        let mut h = Harness::solo();
        let mut m = env("m", serde_json::json!({ "p": "m" }), Lifecycle::Draft);
        m.provenance = ProvenanceKind::Measured;
        assert_eq!(put(&mut h, m.clone()).unwrap_err().rule, "ENV-06");
        for tag in [
            EvidenceTag::A,
            EvidenceTag::B,
            EvidenceTag::C,
            EvidenceTag::E,
            EvidenceTag::Nf,
        ] {
            let mut ok = m.clone();
            ok.evidence_tag = Some(tag);
            ok.event_id = format!("m-{tag:?}");
            // Distinct bytes per event: a repeat would be an ENV-09 reference case.
            ok.payload = serde_json::json!({ "tag": format!("{tag:?}") });
            let (hash, cid) =
                crate::envelope::address(&crate::envelope::payload_bytes(&ok.payload).unwrap());
            ok.payload_hash = hash;
            ok.cid = cid;
            put(&mut h, ok).unwrap();
        }
        let mut i = env("i", serde_json::json!({ "p": "i" }), Lifecycle::Draft);
        i.provenance = ProvenanceKind::Inferred;
        assert_eq!(put(&mut h, i.clone()).unwrap_err().rule, "ENV-06");
        i.source_ref = Some("  ".into());
        assert_eq!(put(&mut h, i.clone()).unwrap_err().rule, "ENV-06");
        i.source_ref = Some("model:planner@run-7".into());
        put(&mut h, i).unwrap();
        // stated needs neither.
        put(
            &mut h,
            env("s", serde_json::json!({ "p": "s" }), Lifecycle::Draft),
        )
        .unwrap();
    }

    #[test]
    fn same_bytes_twice_is_one_cid_and_a_reference() {
        let mut h = Harness::solo();
        let key = apparatus_crypto::Ed25519Key::generate().unwrap();
        let first = signed(
            env("a", serde_json::json!({ "x": 1 }), Lifecycle::Draft),
            &key,
        );
        put(&mut h, first.clone()).unwrap();
        assert_eq!(h.state.cid_owner(&first.cid), Some("a"));

        // Same bytes under another event_id without the reference: ENV-09.
        let bare = signed(
            env("b", serde_json::json!({ "x": 1 }), Lifecycle::Draft),
            &key,
        );
        assert_eq!(bare.cid, first.cid, "same bytes, same CID");
        assert_eq!(put(&mut h, bare).unwrap_err().rule, "ENV-09");
        // Pointing at the wrong owner: ENV-09.
        let mut wrong = env("b", serde_json::json!({ "x": 1 }), Lifecycle::Draft);
        wrong.duplicate_of = Some("nobody".into());
        let wrong = signed(wrong, &key);
        assert_eq!(put(&mut h, wrong).unwrap_err().rule, "ENV-09");
        // With duplicate_of = first owner: accepted, still one owner for the CID.
        let mut reference = env("b", serde_json::json!({ "x": 1 }), Lifecycle::Draft);
        reference.duplicate_of = Some("a".into());
        put(&mut h, signed(reference, &key)).unwrap();
        assert_eq!(h.state.cid_owner(&first.cid), Some("a"));
        // A reference on a new CID: ENV-09.
        let mut stray = env("c", serde_json::json!({ "x": 2 }), Lifecycle::Draft);
        stray.duplicate_of = Some("a".into());
        assert_eq!(put(&mut h, stray).unwrap_err().rule, "ENV-09");
        // The same event_id moving through its lifecycle is not a duplicate of itself.
        let mut sealed = first;
        sealed.lifecycle = Lifecycle::Sealed;
        put(&mut h, sealed).unwrap();
    }
}

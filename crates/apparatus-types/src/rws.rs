//! RWS 2.0 vocabulary and typed receipt payloads.
//!
//! These types are the shape of `Receipt::operation_payload` as defined in
//! RWS-2.0 §12 (X-02, X-03) and RWS-2.0-MAPPING §2. They carry no validation
//! logic; `apparatus-schema` owns the rules.

use crate::{ArtifactId, ObjectId, PrincipalId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Payload format version written into every receipt.
pub const RWS_VERSION: &str = "1.0";

/// A function a principal may hold (RWS-2.0 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Continuity,
    Policy,
    Execution,
    MandateHolder,
    Adviser,
    Coordinator,
}

impl Role {
    /// The snake_case wire name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Continuity => "continuity",
            Role::Policy => "policy",
            Role::Execution => "execution",
            Role::MandateHolder => "mandate_holder",
            Role::Adviser => "adviser",
            Role::Coordinator => "coordinator",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What kind of actor a principal is. Agents and external models are limited
/// to `execution` and `adviser` (R-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    Human,
    Agent,
    Service,
    ExternalModel,
}

/// The two pipelines (RWS-2.0 O-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueKind {
    Keep,
    Build,
}

/// The three states of means (RWS-2.0 §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeansState {
    Bound,
    Committed,
    Reserved,
}

/// Build gates G1..G4 (RWS-2.0 B-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateKind {
    Start,
    Preferred,
    Project,
    Handover,
}

/// Outcome of a gate decision (RWS-2.0 B-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateOutcome {
    Go,
    NoGo,
    Hold,
    Return,
    Stop,
}

/// Evidence strength tag (RWS-2.0 C-05). Wire form keeps the corpus letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceTag {
    A,
    B,
    C,
    E,
    #[serde(rename = "NF")]
    Nf,
}

/// States of a keep WorkItem (RWS-2.0 K-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeepState {
    Signalled,
    Outlook,
    Planned,
    Firm,
    Executing,
    Closed,
    Deferred,
    Reclassified,
}

/// States of a build WorkItem (RWS-2.0 B-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildState {
    Proposed,
    Exploring,
    Designing,
    Realising,
    HandedOver,
    Stopped,
}

/// Carry-over rule for unspent means at period end (RWS-2.0 M-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CarryOver {
    Full,
    None,
    WithinQueue,
}

/// Validity period in Unix milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Period {
    pub from_ms: u64,
    pub until_ms: u64,
}

/// Amounts per means state, in integer units.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Buckets {
    pub bound: u64,
    pub committed: u64,
    pub reserved: u64,
}

impl Buckets {
    /// Sum of the three buckets.
    pub fn total(&self) -> u64 {
        self.bound
            .saturating_add(self.committed)
            .saturating_add(self.reserved)
    }
}

/// A bounded pool of means (RWS-2.0 O-09).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub id: ObjectId,
    pub means_kind: String,
    pub unit: String,
    pub period: Period,
    pub ceiling: u64,
    pub buckets: Buckets,
    pub queue: QueueKind,
    pub carry_over: CarryOver,
    pub overprogramming: bool,
}

/// A receipt payload: common fields (X-03) plus one kind-specific body (X-02).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payload {
    pub rws: String,
    /// Object the receipt is about. `None` means the receipt creates its subject.
    pub subject_id: Option<ObjectId>,
    pub signer: PrincipalId,
    pub role: Role,
    pub authority_ref: Option<ObjectId>,
    #[serde(default)]
    pub evidence: Vec<ArtifactId>,
    #[serde(default)]
    pub self_certified: bool,
    #[serde(flatten)]
    pub body: Body,
}

/// The payload kinds: the eleven of RWS-2.0 X-02 plus `envelope` (NAP-corpus #12,
/// Envelope Stage 0; within the "~12" limit of RWS-2.0-MAPPING §2). No others exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Body {
    Ingest(Ingest),
    RoleBind(RoleBind),
    QueueKeep(QueueKeep),
    QueueBuild(QueueBuild),
    Gate(Gate),
    Means(Means),
    Advice(Advice),
    Policy(Policy),
    Event(Event),
    Correction(Correction),
    Review(Review),
    Envelope(EventEnvelope),
}

impl Body {
    /// The snake_case kind name.
    pub fn kind(&self) -> &'static str {
        match self {
            Body::Ingest(_) => "ingest",
            Body::RoleBind(_) => "role_bind",
            Body::QueueKeep(_) => "queue_keep",
            Body::QueueBuild(_) => "queue_build",
            Body::Gate(_) => "gate",
            Body::Means(_) => "means",
            Body::Advice(_) => "advice",
            Body::Policy(_) => "policy",
            Body::Event(_) => "event",
            Body::Correction(_) => "correction",
            Body::Review(_) => "review",
            Body::Envelope(_) => "envelope",
        }
    }
}

/// Ingest decision (RWS-2.0 C-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestOutcome {
    Admit,
    AdmitRestricted,
    ReferenceOnly,
    Quarantine,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ingest {
    pub outcome: IngestOutcome,
    pub purpose: String,
    pub artifact_id: Option<ArtifactId>,
    /// Lowercase hex SHA-256 of the bytes.
    pub digest: String,
    pub size_bytes: u64,
    pub media_type: String,
    pub source_locator: String,
    pub evidence_tag: EvidenceTag,
    pub reason: Option<String>,
    #[serde(default)]
    pub replica: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleBind {
    pub bound_role: Role,
    /// `None` vacates the role.
    pub principal: Option<PrincipalId>,
    pub principal_name: Option<String>,
    pub principal_kind: Option<PrincipalKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueKeep {
    /// `None` on creation.
    pub from: Option<KeepState>,
    pub to: KeepState,
    pub asset_ref: String,
    pub lead: Option<PrincipalId>,
    #[serde(default)]
    pub adds_function: bool,
    #[serde(default)]
    pub urgent_safety: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueBuild {
    pub asset_ref: String,
    pub lead: Option<PrincipalId>,
    pub adds_function: bool,
    pub programme_ref: Option<ObjectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signer {
    pub principal: PrincipalId,
    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeansCheck {
    pub envelope_ref: ObjectId,
    pub required: u64,
    pub available: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discharge {
    pub role: Role,
    pub principal: PrincipalId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeepLink {
    pub agreement_ref: ObjectId,
    pub envelope_ref: Option<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gate {
    pub gate: GateKind,
    pub outcome: GateOutcome,
    #[serde(default)]
    pub signers: Vec<Signer>,
    pub means_check: Option<MeansCheck>,
    pub discharge: Option<Discharge>,
    pub keep: Option<KeepLink>,
    pub valid_until: Option<u64>,
}

/// Destination of a means movement. `Spent` leaves the envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeansTarget {
    Bound,
    Committed,
    Reserved,
    Spent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Means {
    pub envelope_ref: ObjectId,
    pub from_state: MeansState,
    pub to_state: MeansTarget,
    pub amount: u64,
    pub unit: String,
    pub item_ref: Option<ObjectId>,
    pub gate_ref: Option<ObjectId>,
    pub agreement_ref: Option<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advice {
    pub producer: PrincipalId,
    pub producer_kind: PrincipalKind,
    pub summary: String,
    #[serde(default)]
    pub inputs: Vec<ObjectId>,
    pub addressed_to: Option<Role>,
    pub respond_by: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicySubKind {
    Rule,
    Agreement,
    Envelope,
    Framework,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdviceResponseKind {
    Followed,
    Partial,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdviceResponse {
    pub advice_id: ObjectId,
    pub response: AdviceResponseKind,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub sub_kind: PolicySubKind,
    pub body_ref: Option<ArtifactId>,
    #[serde(default)]
    pub signers: Vec<Signer>,
    #[serde(default)]
    pub adopts: Vec<ObjectId>,
    #[serde(default)]
    pub advice_response: Vec<AdviceResponse>,
    pub supersedes: Option<ObjectId>,
    pub reaffirms: Option<ObjectId>,
    /// Required when `sub_kind` is `envelope`.
    pub envelope: Option<Envelope>,
    /// Assets covered when `sub_kind` is `agreement`.
    #[serde(default)]
    pub asset_scope: Vec<String>,
    /// Start-gate funding sight in basis points (M-06), when `sub_kind` is `rule`.
    pub start_funding_share_bp: Option<u32>,
}

/// Closed v1 event vocabulary: RWS-2.0 F-02 (failures) ∪ X-12 (non-failures).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    // F-02
    TargetDropped,
    TargetMissed,
    CapacityExceedsEnvelope,
    EnvelopeBelowFloor,
    EnvelopeOverrun,
    Displacement,
    ScopeReclassified,
    GateAbandoned,
    GateBypassed,
    IllegalTransitionRejected,
    DeadlineMissed,
    AdviceUnanswered,
    MeansNotReleased,
    NotFound,
    RestoreFailed,
    IntegrityFailed,
    // X-12
    ConditionReported,
    DisruptionReported,
    ReportBack,
    ReportBackResolved,
    RestoreTested,
    TargetMet,
    StopRecorded,
    RoleVacated,
    Escalated,
    DeEscalated,
    WorkStarted,
    Spent,
    Deployed,
    ContractSigned,
}

impl EventKind {
    /// True for the failure kinds of RWS-2.0 F-02.
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            EventKind::TargetDropped
                | EventKind::TargetMissed
                | EventKind::CapacityExceedsEnvelope
                | EventKind::EnvelopeBelowFloor
                | EventKind::EnvelopeOverrun
                | EventKind::Displacement
                | EventKind::ScopeReclassified
                | EventKind::GateAbandoned
                | EventKind::GateBypassed
                | EventKind::IllegalTransitionRejected
                | EventKind::DeadlineMissed
                | EventKind::AdviceUnanswered
                | EventKind::MeansNotReleased
                | EventKind::NotFound
                | EventKind::RestoreFailed
                | EventKind::IntegrityFailed
        )
    }

    /// Default role that owns the next decision (RWS-2.0 F-02 table).
    pub fn default_next_role(&self) -> Role {
        match self {
            EventKind::CapacityExceedsEnvelope
            | EventKind::EnvelopeBelowFloor
            | EventKind::IntegrityFailed => Role::Continuity,
            EventKind::Displacement
            | EventKind::IllegalTransitionRejected
            | EventKind::NotFound
            | EventKind::RestoreFailed => Role::Execution,
            _ => Role::Policy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextDecision {
    pub role: Role,
    pub by: u64,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub event: EventKind,
    pub impact: Option<String>,
    pub next_decision: Option<NextDecision>,
    #[serde(default)]
    pub refs: Vec<ObjectId>,
    /// Free-text detail, e.g. the rule id and attempted payload of a refusal.
    pub detail: Option<String>,
    /// Present on a ticket: a `report_back` event anyone bound may file (M3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<Ticket>,
}

/// A ticket: a defect or "this does not work" report. Carried by a
/// `report_back` event; closed by a `report_back_resolved` event whose subject
/// is the ticket's receipt id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    pub title: String,
    pub body: Option<String>,
    /// Principal name of whoever filed it.
    pub reporter: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Correction {
    pub corrects: ObjectId,
    pub reason: String,
    pub replacement: Option<ObjectId>,
    #[serde(default)]
    pub removal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewKind {
    ProgrammeRefresh,
    Condition,
    Account,
    Replica,
    SourceAudit,
    PolicyEvaluation,
    Quality,
    Strategic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rhythm {
    Scheduled,
    OutOfCycle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(rename = "type")]
    pub trigger_type: String,
    #[serde(rename = "ref")]
    pub trigger_ref: ObjectId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub review: ReviewKind,
    pub rhythm: Rhythm,
    pub step: Option<u8>,
    pub artefact_ref: Option<ArtifactId>,
    pub trigger: Option<Trigger>,
}

/// How the content of an envelope came to be known (ENVELOP.md §1, mandatory).
/// Named `ProvenanceKind` because `apparatus_types::Provenance` is the header's lineage struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceKind {
    Stated,
    Measured,
    Inferred,
}

/// Envelope lifecycle (ENVELOP.md §2, NAP T6 B.10/C). Stored, not enforced, in
/// Stage 0: `sealed` and `anchored` are accepted as declared (sealing is #13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Draft,
    Signed,
    Sealed,
    Anchored,
}

/// Signature scheme of an envelope. Additive only: a scheme name is never
/// reused, so receipts signed under it stay verifiable as what they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureScheme {
    /// Ed25519 (RFC 8032). The real Stage 0 path.
    #[serde(rename = "ed25519")]
    Ed25519,
    /// ML-DSA-65 placeholder: verification always succeeds. Replaced by a new
    /// `ml_dsa_65` scheme in Stage 2 (#14), not by changing this one.
    #[serde(rename = "ml_dsa_65_stub")]
    MlDsa65Stub,
}

impl SignatureScheme {
    /// The wire name.
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureScheme::Ed25519 => "ed25519",
            SignatureScheme::MlDsa65Stub => "ml_dsa_65_stub",
        }
    }
}

/// A signature over an envelope's signing bytes (see `apparatus_schema::envelope`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeSignature {
    /// `ARCHIVE-KID-…`, bound to the scheme (NAP T6 D.3).
    pub signing_key_id: String,
    /// Lowercase hex public key.
    pub public_key: String,
    /// Lowercase hex signature.
    pub value: String,
}

/// An event envelope (ENVELOP.md §1, NAP-corpus #12): a typed record on the
/// existing chain. The chain's SHA-256 link still covers the whole receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    /// Seconds since the Unix epoch.
    pub unix_timestamp: u64,
    pub source: String,
    pub module: String,
    pub provenance: ProvenanceKind,
    /// Inline JSON payload, at most 4 KB canonical (NAP T6 B.4).
    pub payload: serde_json::Value,
    /// Lowercase hex SHA-256 of the canonical payload bytes.
    pub payload_hash: String,
    /// CID v1 (raw, BLAKE3-256, base32 lowercase) of the canonical payload bytes.
    pub cid: String,
    pub lifecycle: Lifecycle,
    pub signature_scheme: SignatureScheme,
    /// Absent only while `lifecycle` is `draft`.
    pub signature: Option<EnvelopeSignature>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(body: Body) -> Payload {
        Payload {
            rws: RWS_VERSION.to_string(),
            subject_id: None,
            signer: PrincipalId::new(),
            role: Role::Execution,
            authority_ref: None,
            evidence: vec![],
            self_certified: false,
            body,
        }
    }

    #[test]
    fn payload_roundtrips_with_kind_tag() {
        let p = payload(Body::QueueKeep(QueueKeep {
            from: None,
            to: KeepState::Signalled,
            asset_ref: "demo".into(),
            lead: None,
            adds_function: false,
            urgent_safety: false,
        }));
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["kind"], "queue_keep");
        assert_eq!(v["role"], "execution");
        let back: Payload = serde_json::from_value(v).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn unknown_kind_and_event_are_rejected() {
        let mut v = serde_json::to_value(payload(Body::Event(Event {
            event: EventKind::TargetDropped,
            impact: None,
            next_decision: None,
            refs: vec![],
            detail: None,
            ticket: None,
        })))
        .unwrap();
        assert_eq!(v["event"], "target_dropped");
        v["event"] = "mood_changed".into();
        assert!(serde_json::from_value::<Payload>(v.clone()).is_err());
        v["kind"] = "vision_deck".into();
        assert!(serde_json::from_value::<Payload>(v).is_err());
    }

    #[test]
    fn envelope_kind_and_enums_on_the_wire() {
        let p = payload(Body::Envelope(EventEnvelope {
            event_id: "e1".into(),
            unix_timestamp: 1,
            source: "sensor".into(),
            module: "dexos.test".into(),
            provenance: ProvenanceKind::Measured,
            payload: serde_json::json!({ "n": 1 }),
            payload_hash: String::new(),
            cid: String::new(),
            lifecycle: Lifecycle::Draft,
            signature_scheme: SignatureScheme::MlDsa65Stub,
            signature: None,
        }));
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["kind"], "envelope");
        assert_eq!(v["provenance"], "measured");
        assert_eq!(v["lifecycle"], "draft");
        assert_eq!(v["signature_scheme"], "ml_dsa_65_stub");
        assert_eq!(serde_json::from_value::<Payload>(v.clone()).unwrap(), p);
        // Provenance is a closed enum, not a free string.
        let mut bad = v.clone();
        bad["provenance"] = "guessed".into();
        assert!(serde_json::from_value::<Payload>(bad).is_err());
        let mut bad = v;
        bad["lifecycle"] = "frozen".into();
        assert!(serde_json::from_value::<Payload>(bad).is_err());
    }

    #[test]
    fn evidence_tag_keeps_corpus_letters() {
        assert_eq!(serde_json::to_value(EvidenceTag::Nf).unwrap(), "NF");
        assert_eq!(serde_json::to_value(EvidenceTag::C).unwrap(), "C");
    }
}

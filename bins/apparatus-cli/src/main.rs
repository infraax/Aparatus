//! Apparatus CLI: `doctor` plus the `rws` subcommands of the M1 runtime.
//!
//! Receipts are written to `<project>/.apparatus/` (see `kernel.rs`). Output is
//! one line per receipt: `<id> <kind> <hash>`. Exit codes: 0 ok, 1 error or
//! integrity failure, 2 refused by an RWS rule (the refusal is on the chain).

mod kernel;

use anyhow::{anyhow, bail, Context, Result};
use apparatus_crypto::Sha256Digest;
use apparatus_schema::rws::{role_rank, ItemState};
use apparatus_types::rws::*;
use apparatus_types::{ArtifactId, Classification, ObjectId};
use clap::{Args, Parser, Subcommand};
use kernel::{apparatus_dir, buffer_path, line, BatchPart, Kernel, Submit, DAY_MS};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "apparatus", version = env!("CARGO_PKG_VERSION"), about = "Apparatus local node")]
struct Cli {
    /// Project directory (contains `.apparatus/`).
    #[arg(long, global = true, default_value = ".")]
    project: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Perform diagnostic checks on the Apparatus environment.
    Doctor,
    /// RWS 2.0 runtime commands.
    #[command(subcommand)]
    Rws(Rws),
}

/// Who signs: `--as <name>` (default: the project's default principal) and an optional role override.
#[derive(Args, Debug, Clone)]
struct Signing {
    /// Principal name to sign as.
    #[arg(long = "as")]
    as_name: Option<String>,
    /// Role to sign under (default depends on the command).
    #[arg(long, value_parser = parse_enum::<Role>)]
    role: Option<Role>,
}

#[derive(Subcommand, Debug)]
enum Rws {
    /// Create `.apparatus/` and bind roles (or import `rws/receipts.jsonl` once).
    Init {
        /// One principal holds continuity, policy and execution; receipts are self-certified.
        #[arg(long)]
        solo: bool,
        #[arg(long, default_value = "owner")]
        principal: String,
        #[arg(long)]
        name: Option<String>,
    },
    /// Bind a role to a principal (signed by continuity).
    RoleBind {
        #[arg(long = "role", value_parser = parse_enum::<Role>)]
        bound_role: Role,
        #[arg(long)]
        principal: String,
        #[arg(long, default_value = "human", value_parser = parse_enum::<PrincipalKind>)]
        kind: PrincipalKind,
        #[arg(long = "as")]
        as_name: Option<String>,
    },
    /// Admit a file into the CAS and record an ingest receipt.
    Ingest {
        path: PathBuf,
        #[arg(long)]
        purpose: String,
        #[arg(long, value_parser = parse_enum::<EvidenceTag>)]
        tag: EvidenceTag,
        #[arg(long, default_value = "internal", value_parser = parse_classification)]
        classification: Classification,
        /// Mark as a replica (overview, index, dashboard); never authority.
        #[arg(long)]
        replica: bool,
        #[command(flatten)]
        signing: Signing,
    },
    /// Create a work item, or move a keep item (`--item <id> --to <state>`).
    Queue {
        #[arg(value_parser = parse_enum::<QueueKind>)]
        queue: QueueKind,
        #[arg(long)]
        asset: Option<String>,
        #[arg(long)]
        lead: Option<String>,
        #[arg(long)]
        item: Option<ObjectId>,
        #[arg(long, value_parser = parse_enum::<KeepState>)]
        to: Option<KeepState>,
        #[arg(long)]
        adds_function: bool,
        #[arg(long)]
        urgent_safety: bool,
        #[arg(long)]
        evidence: Vec<ArtifactId>,
        /// With `--to deferred`: the work item that displaces this one.
        #[arg(long)]
        by: Option<ObjectId>,
        /// With `--to deferred`: why it is displaced (the displacement impact).
        #[arg(long)]
        reason: Option<String>,
        #[command(flatten)]
        signing: Signing,
    },
    /// Take a build gate decision on a work item.
    Gate {
        #[arg(long)]
        item: ObjectId,
        #[arg(long, value_parser = parse_enum::<GateKind>)]
        gate: GateKind,
        #[arg(long, value_parser = parse_enum::<GateOutcome>)]
        outcome: GateOutcome,
        #[arg(long)]
        evidence: Vec<ArtifactId>,
        /// Build envelope for the means check.
        #[arg(long)]
        envelope: Option<ObjectId>,
        #[arg(long)]
        required: Option<u64>,
        /// Claimed availability (default: computed from the envelope).
        #[arg(long)]
        available: Option<u64>,
        /// Handover: principal holding execution to discharge.
        #[arg(long)]
        discharge: Option<String>,
        /// Handover: keep agreement that takes over the asset.
        #[arg(long)]
        agreement: Option<ObjectId>,
        #[arg(long)]
        keep_envelope: Option<ObjectId>,
        #[command(flatten)]
        signing: Signing,
    },
    /// Move means between states (or spend).
    Means {
        #[arg(long)]
        envelope: ObjectId,
        #[arg(long, value_parser = parse_enum::<MeansState>)]
        from: MeansState,
        #[arg(long, value_parser = parse_enum::<MeansTarget>)]
        to: MeansTarget,
        #[arg(long)]
        amount: u64,
        #[arg(long)]
        unit: Option<String>,
        #[arg(long)]
        item: Option<ObjectId>,
        #[arg(long)]
        gate: Option<ObjectId>,
        #[arg(long)]
        agreement: Option<ObjectId>,
        #[command(flatten)]
        signing: Signing,
    },
    /// Record an event (failure kinds per RWS-2.0 F-02, others per X-12).
    Event {
        #[arg(value_parser = parse_enum::<EventKind>)]
        kind: EventKind,
        #[arg(long)]
        subject: Option<ObjectId>,
        #[arg(long)]
        impact: Option<String>,
        #[arg(long, value_parser = parse_enum::<Role>)]
        next_role: Option<Role>,
        #[arg(long, default_value_t = 7)]
        by_days: u64,
        #[arg(long = "ref")]
        refs: Vec<ObjectId>,
        #[command(flatten)]
        signing: Signing,
    },
    /// Record advice (never authority).
    Advice {
        #[arg(long)]
        summary: String,
        #[arg(long)]
        input: Vec<ObjectId>,
        #[arg(long, default_value = "human", value_parser = parse_enum::<PrincipalKind>)]
        producer_kind: PrincipalKind,
        #[command(flatten)]
        signing: Signing,
    },
    /// Correct an earlier object (C-06). The corrector must rank at least the original signer.
    Correction {
        #[arg(long)]
        corrects: ObjectId,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        replacement: Option<ObjectId>,
        /// Lawful removal (bytes replaced by a tombstone later; the receipt stays).
        #[arg(long)]
        removal: bool,
        #[command(flatten)]
        signing: Signing,
    },
    /// Record a review (V-01..V-04).
    Review {
        #[arg(long = "kind", value_parser = parse_enum::<ReviewKind>)]
        review: ReviewKind,
        #[arg(long)]
        subject: ObjectId,
        /// Out-of-cycle review; needs `--trigger`.
        #[arg(long)]
        out_of_cycle: bool,
        #[arg(long)]
        trigger: Option<ObjectId>,
        #[arg(long, default_value = "event")]
        trigger_type: String,
        /// Strategic review step 1..6.
        #[arg(long)]
        step: Option<u8>,
        #[arg(long)]
        artefact: Option<ArtifactId>,
        #[command(flatten)]
        signing: Signing,
    },
    /// Adopt a policy: envelope, agreement or rule.
    #[command(subcommand)]
    Policy(PolicyCmd),
    /// Show an object: work item, envelope, artefact or receipt.
    Show { id: ObjectId },
    /// Verify the chain, CAS and buffer; list refusals and open failures.
    Check,
    /// Import `rws/receipts.jsonl` (or another buffer) into an empty chain, once.
    ImportJsonl { path: PathBuf },
}

#[derive(Subcommand, Debug)]
enum PolicyCmd {
    /// Create an envelope (signed by continuity).
    Envelope {
        #[arg(long, value_parser = parse_enum::<QueueKind>)]
        queue: QueueKind,
        #[arg(long)]
        ceiling: u64,
        #[arg(long, default_value = "money")]
        means_kind: String,
        #[arg(long, default_value = "EUR")]
        unit: String,
        #[arg(long, default_value_t = 365)]
        period_days: u64,
        #[arg(long, default_value = "full", value_parser = parse_enum::<CarryOver>)]
        carry_over: CarryOver,
        #[arg(long)]
        overprogramming: bool,
        #[arg(long = "as")]
        as_name: Option<String>,
    },
    /// Adopt a keep agreement (signed under continuity, policy and execution).
    Agreement {
        #[arg(long = "asset", required = true)]
        assets: Vec<String>,
        #[arg(long)]
        body: Option<ArtifactId>,
        /// Signers as `name:role` (default: the signing principal in all three roles).
        #[arg(long = "signer")]
        signers: Vec<String>,
        #[arg(long = "as")]
        as_name: Option<String>,
    },
    /// Adopt a rule (signed by policy).
    Rule {
        #[arg(long)]
        body: Option<ArtifactId>,
        #[arg(long)]
        start_funding_share_bp: Option<u32>,
        /// Adopt advice: `<advice-id>:followed|partial|rejected:<reason>`.
        #[arg(long = "adopt")]
        adopts: Vec<String>,
        #[arg(long = "as")]
        as_name: Option<String>,
    },
}

fn parse_enum<T: DeserializeOwned>(s: &str) -> Result<T, String> {
    serde_json::from_value(serde_json::Value::String(s.replace('-', "_")))
        .or_else(|_| serde_json::from_value(serde_json::Value::String(s.to_string())))
        .map_err(|_| format!("invalid value '{s}'"))
}

fn parse_classification(s: &str) -> Result<Classification, String> {
    match s {
        "public" => Ok(Classification::Public),
        "internal" => Ok(Classification::Internal),
        "restricted" => Ok(Classification::Restricted),
        _ => Err(format!("invalid classification '{s}'")),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Commands::Doctor => {
            println!("Apparatus Doctor");
            println!("----------------");
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Architecture: {}", std::env::consts::ARCH);
            println!("OS: {}", std::env::consts::OS);
            println!("\nDiagnostics complete. System is capable of running Apparatus foundations.");
            Ok(ExitCode::SUCCESS)
        }
        Commands::Rws(cmd) => rws(&cli.project, cmd),
    }
}

fn print_lines(lines: Vec<String>) -> ExitCode {
    for l in lines {
        println!("{l}");
    }
    ExitCode::SUCCESS
}

fn report(result: Submit) -> ExitCode {
    match result {
        Submit::Accepted { id, kind, hash } => {
            println!("{}", line(id, kind, &hash));
            ExitCode::SUCCESS
        }
        Submit::Refused(r) => {
            eprintln!("REFUSED {}", r.rejection);
            if let Some((id, hash)) = r.recorded {
                println!("{}", line(id, "event", &hash));
            }
            ExitCode::from(2)
        }
    }
}

fn report_all(results: Vec<Submit>) -> ExitCode {
    let mut code = ExitCode::SUCCESS;
    for r in results {
        let c = report(r);
        if c != ExitCode::SUCCESS {
            code = c;
        }
    }
    code
}

fn rws(root: &Path, cmd: Rws) -> Result<ExitCode> {
    match cmd {
        Rws::Init {
            solo,
            principal,
            name,
        } => Ok(print_lines(Kernel::init(root, name, principal, solo)?)),
        Rws::ImportJsonl { path } => {
            Ok(print_lines(Kernel::import_jsonl(root, &path, None, false)?))
        }
        Rws::Check => check(root),
        Rws::Show { id } => show(&Kernel::open(root)?, id),
        other => {
            let mut k = Kernel::open(root)?;
            let results = write(&mut k, other)?;
            Ok(report_all(results))
        }
    }
}

/// Write one command. Deferring a keep item is two receipts in one locked unit:
/// the `displacement` event K-09 requires, then the `queue_keep` transition.
fn write(k: &mut Kernel, cmd: Rws) -> Result<Vec<Submit>> {
    if let Rws::Queue {
        queue: QueueKind::Keep,
        item: Some(item),
        to: Some(KeepState::Deferred),
        by,
        reason,
        lead,
        signing,
        evidence,
        ..
    } = cmd
    {
        let (signer, _) = k.principal(signing.as_name.as_deref());
        let role = signing.role.unwrap_or(Role::Execution);
        let (current, asset_ref) = match k.state.item(item) {
            Some(i) => (
                match i.state {
                    ItemState::Keep(s) => Some(s),
                    _ => None,
                },
                i.asset_ref.clone(),
            ),
            None => (None, String::new()),
        };
        let lead = lead.map(|l| k.principal(Some(&l)).0);
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
        });
        let defer = Body::QueueKeep(QueueKeep {
            from: current,
            to: KeepState::Deferred,
            asset_ref,
            lead,
            adds_function: false,
            urgent_safety: false,
        });
        return k.submit_batch(vec![
            BatchPart {
                signer,
                role,
                subject: Some(item),
                evidence: vec![],
                body: displacement,
            },
            BatchPart {
                signer,
                role,
                subject: Some(item),
                evidence,
                body: defer,
            },
        ]);
    }
    write_one(k, cmd).map(|s| vec![s])
}

fn write_one(k: &mut Kernel, cmd: Rws) -> Result<Submit> {
    use Role::*;
    match cmd {
        Rws::RoleBind {
            bound_role,
            principal,
            kind,
            as_name,
        } => {
            let (signer, _) = k.principal(as_name.as_deref());
            let (target, name) = k.principal(Some(&principal));
            k.submit(
                signer,
                Continuity,
                None,
                vec![],
                Body::RoleBind(RoleBind {
                    bound_role,
                    principal: Some(target),
                    principal_name: Some(name),
                    principal_kind: Some(kind),
                }),
            )
        }
        Rws::Ingest {
            path,
            purpose,
            tag,
            classification,
            replica,
            signing,
        } => {
            let bytes =
                std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
            let digest = Sha256Digest::compute(&bytes);
            let (signer, _) = k.principal(signing.as_name.as_deref());
            let role = k.pick_role(
                signer,
                signing.role,
                &[
                    Execution,
                    Policy,
                    Continuity,
                    Adviser,
                    MandateHolder,
                    Coordinator,
                ],
            );
            let artifact_id = ArtifactId::new();
            let restricted = classification == Classification::Restricted;
            let body = Body::Ingest(Ingest {
                outcome: if restricted {
                    IngestOutcome::AdmitRestricted
                } else {
                    IngestOutcome::Admit
                },
                purpose,
                artifact_id: Some(artifact_id),
                digest: format!("{digest:x}"),
                size_bytes: bytes.len() as u64,
                media_type: media_type(&path).into(),
                source_locator: path.display().to_string(),
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
            if let Submit::Accepted { .. } = result {
                println!("artifact {artifact_id} sha256 {digest:x}");
            }
            Ok(result)
        }
        Rws::Queue {
            queue,
            asset,
            lead,
            item,
            to,
            adds_function,
            urgent_safety,
            evidence,
            signing,
            ..
        } => {
            let (signer, _) = k.principal(signing.as_name.as_deref());
            let lead = lead.map(|l| k.principal(Some(&l)).0);
            match (queue, item) {
                (QueueKind::Keep, None) => {
                    let role = signing.role.unwrap_or(Execution);
                    k.submit(
                        signer,
                        role,
                        None,
                        evidence,
                        Body::QueueKeep(QueueKeep {
                            from: None,
                            to: KeepState::Signalled,
                            asset_ref: asset.ok_or_else(|| {
                                anyhow!("--asset is required to create a keep item")
                            })?,
                            lead,
                            adds_function,
                            urgent_safety,
                        }),
                    )
                }
                (QueueKind::Keep, Some(id)) => {
                    let current = match k.state.item(id).map(|i| i.state) {
                        Some(ItemState::Keep(s)) => Some(s),
                        _ => None,
                    };
                    let asset_ref = asset
                        .or_else(|| k.state.item(id).map(|i| i.asset_ref.clone()))
                        .unwrap_or_default();
                    let to = to.ok_or_else(|| anyhow!("--to is required to move a keep item"))?;
                    let role = signing.role.unwrap_or(Execution);
                    k.submit(
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
                    )
                }
                (QueueKind::Build, None) => {
                    let role = signing.role.unwrap_or(Policy);
                    k.submit(
                        signer,
                        role,
                        None,
                        evidence,
                        Body::QueueBuild(QueueBuild {
                            asset_ref: asset.ok_or_else(|| anyhow!("--asset is required"))?,
                            lead,
                            adds_function: true,
                            programme_ref: None,
                        }),
                    )
                }
                (QueueKind::Build, Some(_)) => {
                    bail!("build items move only through `rws gate` (B-03)")
                }
            }
        }
        Rws::Gate {
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
            signing,
        } => {
            let (signer, _) = k.principal(signing.as_name.as_deref());
            let default_role = if gate == GateKind::Handover {
                MandateHolder
            } else {
                Policy
            };
            let role = signing.role.unwrap_or(default_role);
            let means_check = match envelope {
                Some(env) => Some(MeansCheck {
                    envelope_ref: env,
                    required: required
                        .ok_or_else(|| anyhow!("--required is needed with --envelope"))?,
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
            k.submit(
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
            )
        }
        Rws::Means {
            envelope,
            from,
            to,
            amount,
            unit,
            item,
            gate,
            agreement,
            signing,
        } => {
            let (signer, _) = k.principal(signing.as_name.as_deref());
            let default_role = if to == MeansTarget::Spent {
                Execution
            } else {
                Policy
            };
            let role = signing.role.unwrap_or(default_role);
            let unit = unit
                .or_else(|| k.state.envelope(envelope).map(|e| e.envelope.unit.clone()))
                .unwrap_or_default();
            k.submit(
                signer,
                role,
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
            )
        }
        Rws::Event {
            kind,
            subject,
            impact,
            next_role,
            by_days,
            refs,
            signing,
        } => {
            let (signer, _) = k.principal(signing.as_name.as_deref());
            let role = k.pick_role(
                signer,
                signing.role,
                &[
                    Execution,
                    Policy,
                    Continuity,
                    Adviser,
                    MandateHolder,
                    Coordinator,
                ],
            );
            let (impact, next_decision) = if kind.is_failure() {
                (
                    // Recorded at detection; an unassessed impact stays visible in `rws check`.
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
            k.submit(
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
                }),
            )
        }
        Rws::Advice {
            summary,
            input,
            producer_kind,
            signing,
        } => {
            let (signer, _) = k.principal(signing.as_name.as_deref());
            let role = k.pick_role(
                signer,
                signing.role,
                &[Adviser, Execution, Policy, Continuity],
            );
            k.submit(
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
            )
        }
        Rws::Correction {
            corrects,
            reason,
            replacement,
            removal,
            signing,
        } => {
            let (signer, _) = k.principal(signing.as_name.as_deref());
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
            let role = signing.role.unwrap_or_else(|| {
                candidates
                    .into_iter()
                    .find(|r| role_rank(*r) >= floor && k.state.holds(signer, *r))
                    .unwrap_or(Continuity)
            });
            k.submit(
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
            )
        }
        Rws::Review {
            review,
            subject,
            out_of_cycle,
            trigger,
            trigger_type,
            step,
            artefact,
            signing,
        } => {
            let (signer, _) = k.principal(signing.as_name.as_deref());
            let preferred: &[Role] = match review {
                ReviewKind::Condition | ReviewKind::Replica => &[Execution, Policy, Continuity],
                ReviewKind::Account => &[Continuity, Policy, Execution],
                _ => &[Policy, Continuity, Execution],
            };
            let role = k.pick_role(signer, signing.role, preferred);
            k.submit(
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
            )
        }
        Rws::Policy(p) => write_policy(k, p),
        Rws::Init { .. } | Rws::Check | Rws::Show { .. } | Rws::ImportJsonl { .. } => {
            unreachable!("handled in rws()")
        }
    }
}

fn write_policy(k: &mut Kernel, cmd: PolicyCmd) -> Result<Submit> {
    let empty = |sub_kind| Policy {
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
    };
    match cmd {
        PolicyCmd::Envelope {
            queue,
            ceiling,
            means_kind,
            unit,
            period_days,
            carry_over,
            overprogramming,
            as_name,
        } => {
            let (signer, _) = k.principal(as_name.as_deref());
            let now = k.now();
            let id = ObjectId::new_v7();
            let mut p = empty(PolicySubKind::Envelope);
            p.signers = vec![Signer {
                principal: signer,
                role: Role::Continuity,
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
            let result = k.submit(signer, Role::Continuity, None, vec![], Body::Policy(p))?;
            if let Submit::Accepted { .. } = result {
                println!("envelope {id}");
            }
            Ok(result)
        }
        PolicyCmd::Agreement {
            assets,
            body,
            signers,
            as_name,
        } => {
            let (signer, _) = k.principal(as_name.as_deref());
            let signers = if signers.is_empty() {
                [Role::Continuity, Role::Policy, Role::Execution]
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
                            .ok_or_else(|| anyhow!("--signer is name:role"))?;
                        Ok(Signer {
                            principal: k.principal(Some(name)).0,
                            role: parse_enum(role).map_err(|e| anyhow!(e))?,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
            };
            let role = signers
                .iter()
                .find(|s| s.principal == signer)
                .map(|s| s.role)
                .unwrap_or(Role::Continuity);
            let mut p = empty(PolicySubKind::Agreement);
            p.signers = signers;
            p.asset_scope = assets;
            p.body_ref = body;
            k.submit(signer, role, None, vec![], Body::Policy(p))
        }
        PolicyCmd::Rule {
            body,
            start_funding_share_bp,
            adopts,
            as_name,
        } => {
            let (signer, _) = k.principal(as_name.as_deref());
            let mut p = empty(PolicySubKind::Rule);
            p.signers = vec![Signer {
                principal: signer,
                role: Role::Policy,
            }];
            p.body_ref = body;
            p.start_funding_share_bp = start_funding_share_bp;
            for a in adopts {
                let mut parts = a.splitn(3, ':');
                let (Some(id), Some(resp), Some(reason)) =
                    (parts.next(), parts.next(), parts.next())
                else {
                    bail!("--adopt is <advice-id>:followed|partial|rejected:<reason>");
                };
                let id: ObjectId = id.parse().map_err(|e| anyhow!("{e}"))?;
                p.adopts.push(id);
                p.advice_response.push(AdviceResponse {
                    advice_id: id,
                    response: parse_enum(resp).map_err(|e| anyhow!(e))?,
                    reason: reason.into(),
                });
            }
            k.submit(signer, Role::Policy, None, vec![], Body::Policy(p))
        }
    }
}

fn media_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "md" => "text/markdown",
        "txt" => "text/plain",
        "json" | "jsonl" => "application/json",
        "toml" => "application/toml",
        "rs" => "text/x-rust",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

fn show(k: &Kernel, id: ObjectId) -> Result<ExitCode> {
    let mut found = false;
    if let Some(item) = k.state.item(id) {
        found = true;
        let (queue, state) = match item.state {
            ItemState::Keep(s) => ("keep", format!("{s:?}")),
            ItemState::Build(s) => ("build", format!("{s:?}")),
        };
        println!(
            "work_item {id} queue={queue} state={state} asset={} adds_function={}",
            item.asset_ref, item.adds_function
        );
    }
    if let Some(env) = k.state.envelope(id) {
        found = true;
        let e = &env.envelope;
        println!(
            "envelope {id} queue={:?} ceiling={} {} reserved={} committed={} bound={} spent={}",
            e.queue,
            e.ceiling,
            e.unit,
            e.buckets.reserved,
            e.buckets.committed,
            e.buckets.bound,
            env.spent
        );
    }
    if let Some(a) = k.state.artifact(id) {
        found = true;
        println!(
            "artifact {id} sha256={} tag={:?} replica={}",
            a.digest, a.tag, a.replica
        );
    }
    if let Some(entry) = k.receipt(id) {
        found = true;
        println!("receipt {id} hash={:x}", entry.hash);
        println!("{}", serde_json::to_string_pretty(&entry.receipt)?);
    }
    if !found {
        bail!("{id} not found");
    }
    Ok(ExitCode::SUCCESS)
}

fn check(root: &Path) -> Result<ExitCode> {
    let mut problems: Vec<String> = Vec::new();
    let k = match Kernel::open(root) {
        Ok(k) => k,
        Err(e) => {
            println!("FAIL chain: {e:#}");
            return Ok(ExitCode::from(1));
        }
    };
    println!("project {} ({})", k.meta.name, k.meta.project_id);
    match k.head() {
        Some((id, hash)) => println!("chain ok: {} receipts, head {id} {hash:x}", k.entries.len()),
        None => println!("chain ok: empty"),
    }

    // One chain only (C-02, X-11).
    if buffer_path(root).exists() && k.head().is_some() {
        problems.push("rws/receipts.jsonl exists next to a live chain: two chains (X-11)".into());
    }

    // CAS: every admitted artefact is present and matches its digest.
    let mut verified = 0;
    for (id, a) in k.state.artifacts() {
        match Sha256Digest::from_hex(&a.digest).map(|d| k.cas.verify(&d)) {
            Ok(Ok(true)) => verified += 1,
            _ => problems.push(format!("artefact {id}: bytes missing or corrupt in CAS")),
        }
    }
    println!("cas ok: {verified} artefacts verified");

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
            println!("role {role}: {}", names.join(", "));
        }
    }
    if k.state.holders(Role::Execution).is_empty() {
        println!("note: no execution holder bound (R-02); keep work cannot be queued");
    }

    for item in k.state.items() {
        let (queue, state) = match item.state {
            ItemState::Keep(s) => ("keep", format!("{s:?}")),
            ItemState::Build(s) => ("build", format!("{s:?}")),
        };
        println!("item {} {queue} {state} asset={}", item.id, item.asset_ref);
    }

    let refusals: Vec<_> = k
        .state
        .events()
        .iter()
        .filter(|e| e.event == EventKind::IllegalTransitionRejected)
        .collect();
    println!("refusals recorded: {}", refusals.len());
    for r in &refusals {
        println!("  refused {} {}", r.id, r.impact.as_deref().unwrap_or(""));
    }
    let open = k.state.open_failures();
    let open_other: Vec<_> = open
        .iter()
        .filter(|e| e.event != EventKind::IllegalTransitionRejected)
        .collect();
    println!("open failures: {}", open_other.len());
    for f in open_other {
        let nd = f
            .next_decision
            .as_ref()
            .map(|n| format!("next: {} by {}", n.role, n.by))
            .unwrap_or_default();
        let impact = f.impact.as_deref().unwrap_or("");
        let flag = if impact == "unassessed" {
            " [impact unassessed]"
        } else {
            ""
        };
        println!(
            "  {:?} {} subject={} {nd}{flag}",
            f.event,
            f.id,
            f.subject
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into())
        );
    }

    if !apparatus_dir(root).join("HEAD").exists() && k.head().is_some() {
        problems.push("HEAD file missing".into());
    }
    if problems.is_empty() {
        println!("check ok");
        Ok(ExitCode::SUCCESS)
    } else {
        for p in &problems {
            println!("FAIL {p}");
        }
        Ok(ExitCode::from(1))
    }
}

//! Apparatus CLI: `doctor` plus the `rws` subcommands.
//!
//! Every `rws` command is a `Request`. If `apparatusd` is listening on
//! `<project>/.apparatus/apparatusd.sock` the request goes over RPC and this
//! process takes no lock; otherwise it runs locally under `.apparatus/LOCK`.
//! Output is one line per receipt: `<id> <kind> <hash>`. Exit codes: 0 ok,
//! 1 error/busy/integrity failure, 2 refused by an RWS rule (on the chain).

use anyhow::{Context, Result};
use apparatus_kernel::api::{ExecOptions, ListWhat, Op, Request, Response};
use apparatus_kernel::{execute, Kernel};
use apparatus_types::rws::*;
use apparatus_types::{ArtifactId, Classification, ObjectId};
use clap::{Args, Parser, Subcommand};
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
        /// Declared kind of the signing principal (must match its binding).
        #[arg(long, value_parser = parse_enum::<PrincipalKind>)]
        producer_kind: Option<PrincipalKind>,
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
    /// File a ticket (defect, "this does not work"): a `report_back` event.
    Ticket {
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        subject: Option<ObjectId>,
        #[arg(long)]
        body: Option<String>,
        /// Close this ticket instead (a `report_back_resolved` event).
        #[arg(long)]
        close: Option<ObjectId>,
        #[arg(long)]
        reason: Option<String>,
        #[command(flatten)]
        signing: Signing,
    },
    /// Record an event envelope (Stage 0): JSON payload, provenance, lifecycle, CID, signature.
    Envelope {
        /// JSON payload file, or `-` for stdin. Alternative to `--data`.
        #[arg(long, conflicts_with = "data")]
        payload: Option<PathBuf>,
        /// Inline JSON payload.
        #[arg(long)]
        data: Option<String>,
        #[arg(long)]
        source: String,
        #[arg(long)]
        module: String,
        /// stated | measured | inferred
        #[arg(long, value_parser = parse_enum::<ProvenanceKind>)]
        provenance: ProvenanceKind,
        /// Default: a fresh UUIDv7.
        #[arg(long)]
        event_id: Option<String>,
        /// Seconds since the epoch. Default: now.
        #[arg(long)]
        unix_timestamp: Option<u64>,
        /// draft | signed | sealed | anchored (default: signed; sealing is not enforced yet).
        #[arg(long, value_parser = parse_enum::<Lifecycle>)]
        lifecycle: Option<Lifecycle>,
        /// ed25519 | ml_dsa_65_stub
        #[arg(long, default_value = "ed25519", value_parser = parse_enum::<SignatureScheme>)]
        scheme: SignatureScheme,
        #[command(flatten)]
        signing: Signing,
    },
    /// List tickets.
    Tickets {
        #[arg(long)]
        open: bool,
    },
    /// List items, events or tickets.
    List {
        #[arg(value_parser = parse_enum::<ListWhat>)]
        what: ListWhat,
    },
    /// Print the chain head: `<id> <hash> <receipts>`.
    Head,
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

fn rws(root: &Path, cmd: Rws) -> Result<ExitCode> {
    match cmd {
        Rws::Init {
            solo,
            principal,
            name,
        } => {
            refuse_if_daemon(root)?;
            print_lines(Kernel::init(root, name, principal, solo)?);
            Ok(ExitCode::SUCCESS)
        }
        Rws::ImportJsonl { path } => {
            refuse_if_daemon(root)?;
            print_lines(Kernel::import_jsonl(root, &path, None, false)?);
            Ok(ExitCode::SUCCESS)
        }
        other => {
            let req = to_request(other)?;
            Ok(ExitCode::from(dispatch(root, req)?))
        }
    }
}

fn print_lines(lines: Vec<String>) {
    for l in lines {
        println!("{l}");
    }
}

/// `init` and `import-jsonl` create chains; they never run beside a daemon.
fn refuse_if_daemon(root: &Path) -> Result<()> {
    #[cfg(unix)]
    if apparatus_kernel::rpc::connect(root)?.is_some() {
        anyhow::bail!("apparatusd is running for this project; stop it first");
    }
    let _ = root;
    Ok(())
}

/// Daemon first: RPC if a daemon listens, else run locally under the lock.
fn dispatch(root: &Path, req: Request) -> Result<u8> {
    #[cfg(unix)]
    if let Some(stream) = apparatus_kernel::rpc::connect(root)? {
        let mut req = req;
        if req.principal.is_none() {
            req.principal = Some(default_principal(root)?);
        }
        let mut client = apparatus_kernel::rpc::Client::new(stream)?;
        return Ok(print_response(client.call(&req)?));
    }
    let is_check = matches!(req.op, Op::Check);
    let mut kernel = match Kernel::open(root) {
        Ok(k) => k,
        Err(e) if is_check => {
            println!("FAIL chain: {e:#}");
            return Ok(1);
        }
        Err(e) => return Err(e),
    };
    Ok(print_response(execute(
        &mut kernel,
        req,
        ExecOptions::default(),
    )))
}

/// The daemon needs an explicit principal; take the project default.
#[cfg(unix)]
fn default_principal(root: &Path) -> Result<String> {
    let meta: apparatus_kernel::kernel::ProjectMeta = serde_json::from_slice(
        &std::fs::read(apparatus_kernel::apparatus_dir(root).join("project.json"))
            .context("reading .apparatus/project.json")?,
    )?;
    Ok(meta.default_principal)
}

fn print_response(resp: Response) -> u8 {
    for l in &resp.lines {
        println!("{l}");
    }
    if let Some(e) = &resp.error {
        if resp.status == apparatus_kernel::Status::Refused {
            eprintln!("{e}");
        } else {
            eprintln!("error: {e}");
        }
    }
    resp.exit_code()
}

fn req(signing: &Signing, op: Op) -> Request {
    Request {
        principal: signing.as_name.clone(),
        role: signing.role,
        producer_kind: None,
        op,
    }
}

fn plain(as_name: Option<String>, op: Op) -> Request {
    Request {
        principal: as_name,
        role: None,
        producer_kind: None,
        op,
    }
}

fn to_request(cmd: Rws) -> Result<Request> {
    Ok(match cmd {
        Rws::RoleBind {
            bound_role,
            principal,
            kind,
            as_name,
        } => plain(
            as_name,
            Op::RoleBind {
                bound_role,
                target: principal,
                kind,
            },
        ),
        Rws::Ingest {
            path,
            purpose,
            tag,
            classification,
            replica,
            signing,
        } => {
            // Absolute, so a daemon with another working directory reads the same file.
            let abs = std::fs::canonicalize(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            req(
                &signing,
                Op::Ingest {
                    path: Some(abs.display().to_string()),
                    content: None,
                    name: None,
                    purpose,
                    tag,
                    classification: Some(
                        match classification {
                            Classification::Public => "public",
                            Classification::Internal => "internal",
                            Classification::Restricted => "restricted",
                        }
                        .into(),
                    ),
                    replica,
                },
            )
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
            by,
            reason,
            signing,
        } => match queue {
            QueueKind::Keep => req(
                &signing,
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
                },
            ),
            QueueKind::Build => {
                if item.is_some() {
                    anyhow::bail!("build items move only through `rws gate` (B-03)");
                }
                req(
                    &signing,
                    Op::QueueBuild {
                        asset: asset.ok_or_else(|| anyhow::anyhow!("--asset is required"))?,
                        lead,
                        evidence,
                    },
                )
            }
        },
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
        } => req(
            &signing,
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
            },
        ),
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
        } => req(
            &signing,
            Op::Means {
                envelope,
                from,
                to,
                amount,
                unit,
                item,
                gate,
                agreement,
            },
        ),
        Rws::Event {
            kind,
            subject,
            impact,
            next_role,
            by_days,
            refs,
            signing,
        } => req(
            &signing,
            Op::Event {
                kind,
                subject,
                impact,
                next_role,
                by_days,
                refs,
            },
        ),
        Rws::Advice {
            summary,
            input,
            producer_kind,
            signing,
        } => {
            let mut r = req(&signing, Op::Advice { summary, input });
            r.producer_kind = producer_kind;
            r
        }
        Rws::Correction {
            corrects,
            reason,
            replacement,
            removal,
            signing,
        } => req(
            &signing,
            Op::Correction {
                corrects,
                reason,
                replacement,
                removal,
            },
        ),
        Rws::Review {
            review,
            subject,
            out_of_cycle,
            trigger,
            trigger_type,
            step,
            artefact,
            signing,
        } => req(
            &signing,
            Op::Review {
                review,
                subject,
                out_of_cycle,
                trigger,
                trigger_type,
                step,
                artefact,
            },
        ),
        Rws::Ticket {
            title,
            subject,
            body,
            close,
            reason,
            signing,
        } => match close {
            Some(ticket) => req(&signing, Op::TicketClose { ticket, reason }),
            None => req(
                &signing,
                Op::Ticket {
                    title: title.ok_or_else(|| anyhow::anyhow!("--title is required"))?,
                    subject,
                    body,
                },
            ),
        },
        Rws::Envelope {
            payload,
            data,
            source,
            module,
            provenance,
            event_id,
            unix_timestamp,
            lifecycle,
            scheme,
            signing,
        } => {
            let text = match (payload, data) {
                (_, Some(d)) => d,
                (Some(p), None) if p.as_os_str() == "-" => {
                    let mut s = String::new();
                    std::io::Read::read_to_string(&mut std::io::stdin(), &mut s)?;
                    s
                }
                (Some(p), None) => std::fs::read_to_string(&p)
                    .with_context(|| format!("reading {}", p.display()))?,
                (None, None) => anyhow::bail!("give --payload <file|-> or --data <json>"),
            };
            let payload: serde_json::Value =
                serde_json::from_str(&text).context("payload is not JSON")?;
            req(
                &signing,
                Op::Envelope {
                    payload,
                    source,
                    module,
                    provenance,
                    event_id,
                    unix_timestamp,
                    lifecycle,
                    signature_scheme: scheme,
                    payload_hash: None,
                    cid: None,
                },
            )
        }
        Rws::Tickets { open } => plain(None, Op::Tickets { open }),
        Rws::List { what } => plain(None, Op::List { what }),
        Rws::Head => plain(None, Op::Head),
        Rws::Show { id } => plain(None, Op::Show { id }),
        Rws::Check => plain(None, Op::Check),
        Rws::Policy(p) => match p {
            PolicyCmd::Envelope {
                queue,
                ceiling,
                means_kind,
                unit,
                period_days,
                carry_over,
                overprogramming,
                as_name,
            } => plain(
                as_name,
                Op::PolicyEnvelope {
                    queue,
                    ceiling,
                    means_kind,
                    unit,
                    period_days,
                    carry_over,
                    overprogramming,
                },
            ),
            PolicyCmd::Agreement {
                assets,
                body,
                signers,
                as_name,
            } => plain(
                as_name,
                Op::PolicyAgreement {
                    assets,
                    body,
                    signers,
                },
            ),
            PolicyCmd::Rule {
                body,
                start_funding_share_bp,
                adopts,
                as_name,
            } => plain(
                as_name,
                Op::PolicyRule {
                    body,
                    start_funding_share_bp,
                    adopts,
                },
            ),
        },
        Rws::Init { .. } | Rws::ImportJsonl { .. } => unreachable!("handled in rws()"),
    })
}

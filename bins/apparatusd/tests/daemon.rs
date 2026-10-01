//! End-to-end tests: `apparatusd` in a temp project, clients over the socket,
//! and the `apparatus` CLI as a daemon-first client.

use apparatus_kernel::api::{Op, Request, Status};
use apparatus_kernel::rpc::{socket_path, Client};
use apparatus_types::rws::EvidenceTag;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

fn tempdir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Short path: Unix socket paths are limited to ~100 bytes.
    let dir = std::env::temp_dir().join(format!(
        "ad-{tag}-{}-{}",
        std::process::id(),
        nanos % 1_000_000_000
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn daemon_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_apparatusd"))
}

/// The CLI binary sits next to the daemon in the same target directory.
fn cli_bin() -> PathBuf {
    let path = daemon_bin().with_file_name(format!("apparatus{}", std::env::consts::EXE_SUFFIX));
    if !path.exists() {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let status = Command::new(cargo)
            .args(["build", "-p", "apparatus-cli", "--bin", "apparatus"])
            .status()
            .expect("cargo build runs");
        assert!(status.success(), "building the CLI for daemon tests failed");
    }
    path
}

fn cli(dir: &Path, args: &[&str]) -> Output {
    Command::new(cli_bin())
        .arg("--project")
        .arg(dir)
        .args(args)
        .output()
        .expect("cli runs")
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn start(dir: &Path) -> Child {
    let child = Command::new(daemon_bin())
        .arg("--project")
        .arg(dir)
        .env("APPARATUSD_ALLOW_DEBUG_PANIC", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("daemon starts");
    let deadline = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(socket_path(dir)).is_err() {
        assert!(Instant::now() < deadline, "daemon did not open its socket");
        std::thread::sleep(Duration::from_millis(20));
    }
    child
}

fn terminate(mut child: Child) -> std::process::ExitStatus {
    let status = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(status.success());
    child.wait().unwrap()
}

fn client(dir: &Path) -> Client {
    Client::new(UnixStream::connect(socket_path(dir)).unwrap()).unwrap()
}

fn as_owner(op: Op) -> Request {
    Request {
        principal: Some("owner".into()),
        role: None,
        producer_kind: None,
        op,
    }
}

fn ingest_text(text: &str) -> Request {
    as_owner(Op::Ingest {
        path: None,
        content: Some(text.into()),
        name: Some(format!("inline:{text}")),
        purpose: "test".into(),
        tag: EvidenceTag::C,
        classification: None,
        replica: false,
    })
}

fn solo_project(tag: &str) -> PathBuf {
    let dir = tempdir(tag);
    assert!(cli(&dir, &["rws", "init", "--solo"]).status.success());
    dir
}

#[test]
fn rpc_ingest_cli_show_and_restart_keep_the_chain() {
    let dir = solo_project("restart");
    let daemon = start(&dir);

    let mut c = client(&dir);
    let resp = c.call(&ingest_text("hello")).unwrap();
    assert_eq!(resp.status, Status::Ok, "{resp:?}");
    let receipt = resp.id.unwrap().to_string();

    // The CLI finds the socket and asks the daemon.
    let show = cli(&dir, &["rws", "show", &receipt]);
    assert!(show.status.success(), "{}", err(&show));
    assert!(out(&show).contains(&format!("receipt {receipt}")));
    let head_before = out(&cli(&dir, &["rws", "head"]));

    let status = terminate(daemon);
    assert!(status.success(), "graceful stop exits 0");
    assert!(!socket_path(&dir).exists(), "socket removed on stop");

    let daemon = start(&dir);
    let head_after = out(&cli(&dir, &["rws", "head"]));
    assert_eq!(head_before, head_after);
    assert!(out(&cli(&dir, &["rws", "check"])).contains("check ok"));
    terminate(daemon);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn concurrent_clients_are_serialized_by_one_writer() {
    let dir = solo_project("conc");
    let daemon = start(&dir);
    let per_client = 10;
    let threads: Vec<_> = (0..4)
        .map(|t| {
            let dir = dir.clone();
            std::thread::spawn(move || {
                let mut c = client(&dir);
                (0..per_client)
                    .map(|i| {
                        c.call(&ingest_text(&format!("client {t} item {i}")))
                            .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    let mut ok = 0;
    for t in threads {
        for resp in t.join().unwrap() {
            assert!(
                matches!(resp.status, Status::Ok | Status::Refused),
                "{resp:?}"
            );
            if resp.status == Status::Ok {
                ok += 1;
            }
        }
    }
    assert_eq!(ok, 4 * per_client);
    let check = client(&dir).call(&as_owner(Op::Check)).unwrap();
    assert_eq!(check.status, Status::Ok, "{:?}", check.lines);
    assert!(check
        .lines
        .iter()
        .any(|l| l == &format!("cas ok: {ok} artefacts verified")));
    terminate(daemon);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn bad_input_and_panics_do_not_stop_the_daemon() {
    let dir = solo_project("panic");
    let mut daemon = start(&dir);
    let mut c = client(&dir);

    let bad = c.call_raw("{not json\n").unwrap();
    assert_eq!(bad.status, Status::Error);
    assert!(bad.error.unwrap().contains("invalid request"));

    let panic = c.call(&as_owner(Op::DebugPanic)).unwrap();
    assert_eq!(panic.status, Status::Error);
    assert!(panic.error.unwrap().contains("internal error"));

    // Same connection keeps working; the failure became a ticket.
    let head = c.call(&as_owner(Op::Head)).unwrap();
    assert_eq!(head.status, Status::Ok);
    let tickets = c.call(&as_owner(Op::Tickets { open: true })).unwrap();
    assert!(
        tickets
            .lines
            .iter()
            .any(|l| l.contains("debug_panic failed")),
        "{:?}",
        tickets.lines
    );

    // Unknown principals are refused, not crashed on.
    let stranger = c
        .call(&Request {
            principal: Some("stranger".into()),
            role: None,
            producer_kind: None,
            op: Op::Ticket {
                title: "hi".into(),
                subject: None,
                body: None,
            },
        })
        .unwrap();
    assert_eq!(stranger.status, Status::Refused);
    assert_eq!(stranger.rule.as_deref(), Some("X-10"));

    // A client that disconnects mid-way is not a stop signal.
    drop(c);
    std::thread::sleep(Duration::from_millis(100));
    assert!(daemon.try_wait().unwrap().is_none(), "daemon still running");
    assert!(client(&dir).call(&as_owner(Op::Head)).unwrap().ok);
    terminate(daemon);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn tickets_open_list_and_close_via_cli() {
    let dir = solo_project("tickets");
    let daemon = start(&dir);
    let filed = cli(
        &dir,
        &[
            "rws",
            "ticket",
            "--title",
            "gauge reads zero",
            "--body",
            "since monday",
        ],
    );
    assert!(filed.status.success(), "{}", err(&filed));
    let ticket = out(&filed).split_whitespace().next().unwrap().to_string();
    assert!(out(&filed).contains(" event "));

    let open = out(&cli(&dir, &["rws", "tickets", "--open"]));
    assert!(open.contains(&format!("ticket {ticket} open")), "{open}");

    let closed = cli(
        &dir,
        &[
            "rws",
            "ticket",
            "--close",
            &ticket,
            "--reason",
            "sensor replaced",
        ],
    );
    assert!(closed.status.success(), "{}", err(&closed));
    assert!(out(&cli(&dir, &["rws", "tickets", "--open"]))
        .trim()
        .is_empty());
    assert!(out(&cli(&dir, &["rws", "tickets"])).contains("resolved"));
    // Closing twice is refused (A-07).
    assert_eq!(
        cli(&dir, &["rws", "ticket", "--close", &ticket])
            .status
            .code(),
        Some(2)
    );
    terminate(daemon);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn stale_socket_and_second_daemon() {
    let dir = solo_project("stale");
    let mut daemon = start(&dir);
    // A second daemon on the same project refuses to start.
    let second = Command::new(daemon_bin())
        .arg("--project")
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(1));

    // Hard kill leaves the socket file behind.
    daemon.kill().unwrap();
    daemon.wait().unwrap();
    assert!(socket_path(&dir).exists());

    // The CLI notices, removes it, and runs locally.
    let head = cli(&dir, &["rws", "head"]);
    assert!(head.status.success(), "{}", err(&head));
    assert!(err(&head).contains("removed stale socket"));
    assert!(!socket_path(&dir).exists());

    // A daemon also starts cleanly over a stale socket.
    std::os::unix::net::UnixListener::bind(socket_path(&dir)).unwrap();
    let daemon = start(&dir);
    assert!(client(&dir).call(&as_owner(Op::Head)).unwrap().ok);
    terminate(daemon);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn envelopes_over_rpc_signed_by_the_daemon_and_refused_on_chain() {
    use apparatus_types::rws::{ProvenanceKind, SignatureScheme};
    let dir = solo_project("env");
    let daemon = start(&dir);
    let mut c = client(&dir);
    let env = |payload: serde_json::Value, cid: Option<String>| {
        as_owner(Op::EventEnvelope {
            payload,
            source: "agent:planner".into(),
            module: "dexos.plan".into(),
            provenance: ProvenanceKind::Inferred,
            evidence_tag: None,
            source_ref: Some("planner:run-1".into()),
            event_id: None,
            unix_timestamp: None,
            lifecycle: None,
            signature_scheme: SignatureScheme::Ed25519,
            payload_hash: None,
            cid,
        })
    };
    let ok = c
        .call(&env(serde_json::json!({ "step": 1 }), None))
        .unwrap();
    assert_eq!(ok.status, Status::Ok, "{ok:?}");
    assert_eq!(ok.kind.as_deref(), Some("event_envelope"));
    assert!(
        dir.join(".apparatus/keys/ed25519.seed").exists(),
        "the daemon (the only writer) owns the key"
    );

    // A client that states a wrong CID is refused, and the refusal is on the chain.
    let bad = c
        .call(&env(
            serde_json::json!({ "step": 2 }),
            Some("bafkr4iwrong".into()),
        ))
        .unwrap();
    assert_eq!(bad.status, Status::Refused, "{bad:?}");
    assert_eq!(bad.rule.as_deref(), Some("ENV-03"));
    assert_eq!(bad.lines.len(), 1, "the refusal event receipt");

    // Daemon still up; CLI goes through it and check is green.
    let check = cli(&dir, &["rws", "check"]);
    assert!(check.status.success(), "{}", out(&check));
    assert!(out(&check).contains("refusals recorded: 1"));
    assert!(terminate(daemon).success());
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn event_envelope_advance_over_rpc_and_sealed_refusal() {
    use apparatus_types::rws::{Lifecycle, ProvenanceKind, SignatureScheme};
    let dir = solo_project("adv");
    let daemon = start(&dir);
    let mut c = client(&dir);
    let draft = c
        .call(&as_owner(Op::EventEnvelope {
            payload: serde_json::json!({ "step": "plan" }),
            source: "agent:planner".into(),
            module: "dexos.plan".into(),
            provenance: ProvenanceKind::Stated,
            evidence_tag: None,
            source_ref: None,
            event_id: Some("rpc-1".into()),
            unix_timestamp: None,
            lifecycle: Some(Lifecycle::Draft),
            signature_scheme: SignatureScheme::Ed25519,
            payload_hash: None,
            cid: None,
        }))
        .unwrap();
    assert_eq!(draft.status, Status::Ok, "{draft:?}");
    for to in [Lifecycle::Signed, Lifecycle::Sealed] {
        let r = c
            .call(&as_owner(Op::EventEnvelopeAdvance {
                event_id: "rpc-1".into(),
                to,
            }))
            .unwrap();
        assert_eq!(r.status, Status::Ok, "{to:?}: {r:?}");
    }
    let again = c
        .call(&as_owner(Op::EventEnvelopeAdvance {
            event_id: "rpc-1".into(),
            to: Lifecycle::Signed,
        }))
        .unwrap();
    assert_eq!(again.status, Status::Refused);
    assert_eq!(again.rule.as_deref(), Some("ENV-07"));
    assert_eq!(again.lines.len(), 1, "the refusal receipt");
    assert!(terminate(daemon).success());
    std::fs::remove_dir_all(dir).ok();
}

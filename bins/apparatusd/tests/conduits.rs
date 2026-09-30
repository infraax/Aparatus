//! M4 against a live daemon: RPC `ingest_enqueue` into the pipe, queue-full
//! `busy` with one ticket, dampened signal tickets, backup over RPC.

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
        "m4-{tag}-{}-{}",
        std::process::id(),
        nanos % 1_000_000_000
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn daemon_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_apparatusd"))
}

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

fn start(dir: &Path, extra: &[&str]) -> Child {
    let child = Command::new(daemon_bin())
        .arg("--project")
        .arg(dir)
        .args(extra)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("daemon starts");
    let deadline = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(socket_path(dir)).is_err() {
        assert!(Instant::now() < deadline, "daemon did not open its socket");
        std::thread::sleep(Duration::from_millis(20));
    }
    child
}

fn terminate(mut child: Child) {
    Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(child.wait().unwrap().success());
}

fn client(dir: &Path) -> Client {
    Client::new(UnixStream::connect(socket_path(dir)).unwrap()).unwrap()
}

fn owner(op: Op) -> Request {
    Request {
        principal: Some("owner".into()),
        role: None,
        producer_kind: None,
        op,
    }
}

fn enqueue(name: &str, text: &str, tag: Option<EvidenceTag>) -> Request {
    owner(Op::IngestEnqueue {
        path: None,
        content: Some(text.into()),
        name: Some(name.into()),
        purpose: None,
        tag,
        classification: None,
        replica: false,
    })
}

fn solo_project(tag: &str) -> PathBuf {
    let dir = tempdir(tag);
    assert!(cli(&dir, &["rws", "init", "--solo"]).status.success());
    dir
}

fn count(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| !n.starts_with('.') && !n.ends_with(".json"))
        .count()
}

fn wait_until(what: &str, secs: u64, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn open_tickets_titled(dir: &Path, needle: &str) -> usize {
    out(&cli(dir, &["rws", "tickets", "--open"]))
        .lines()
        .filter(|l| l.contains(needle))
        .count()
}

#[test]
fn rpc_enqueue_feeds_the_pipe() {
    let dir = solo_project("pipe");
    let daemon = start(&dir, &["--ingest-poll-ms", "50"]);
    let mut c = client(&dir);
    for (name, text) in [("a.md", "alpha"), ("b.md", "alpha"), ("e.md", "")] {
        let r = c.call(&enqueue(name, text, Some(EvidenceTag::B))).unwrap();
        assert_eq!(r.status, Status::Ok, "{r:?}");
    }
    let inbox = dir.join(".apparatus/in");
    wait_until("the pipe to settle", 10, || {
        count(&inbox.join("ok")) + count(&inbox.join("refuse")) == 3
    });
    assert_eq!(count(&inbox.join("ok")), 2);
    assert_eq!(count(&inbox.join("refuse")), 1);
    assert_eq!(count(&inbox.join("new")), 0);
    assert_eq!(count(&inbox.join("work")), 0);
    let check = c.call(&owner(Op::Check)).unwrap();
    assert_eq!(check.status, Status::Ok, "{:?}", check.lines);
    assert!(check
        .lines
        .iter()
        .any(|l| l == "cas ok: 1 artefacts verified"));

    // Unknown principals cannot enqueue (same identity gate as every write).
    let mut r = enqueue("x.md", "x", None);
    r.principal = Some("stranger".into());
    assert_eq!(c.call(&r).unwrap().rule.as_deref(), Some("X-10"));

    // Backup over RPC while the daemon holds the lock.
    let bdir = tempdir("bk").join("b");
    let r = c
        .call(&owner(Op::Backup {
            to: bdir.display().to_string(),
        }))
        .unwrap();
    assert_eq!(r.status, Status::Ok, "{r:?}");
    let head = c.call(&owner(Op::Head)).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bdir.join("MANIFEST.json")).unwrap()).unwrap();
    assert_eq!(manifest["head_hash"], head.data.unwrap()["hash"]);
    terminate(daemon);
}

#[test]
fn full_queue_answers_busy_with_one_ticket() {
    let dir = solo_project("full");
    // The pipe looks once at start, then not for a minute: the queue stays full.
    let daemon = start(
        &dir,
        &["--ingest-capacity", "1", "--ingest-poll-ms", "60000"],
    );
    std::thread::sleep(Duration::from_millis(300));
    let mut c = client(&dir);
    assert_eq!(
        c.call(&enqueue("1.md", "one", None)).unwrap().status,
        Status::Ok
    );
    for i in 2..5 {
        let r = c.call(&enqueue(&format!("{i}.md"), "more", None)).unwrap();
        assert_eq!(r.status, Status::Busy, "{r:?}");
    }
    assert_eq!(open_tickets_titled(&dir, "ingest queue full"), 1);
    // The CLI maps busy to exit 1.
    let o = cli(&dir, &["rws", "signals"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(out(&o).contains("cause ingest queue full"), "{}", out(&o));
    terminate(daemon);
}

#[test]
fn signal_tick_files_one_ticket_per_cause_and_closes_it() {
    let dir = solo_project("tick");
    let src = dir.join("f.md");
    std::fs::write(&src, b"fragile").unwrap();
    assert!(cli(
        &dir,
        &[
            "rws",
            "ingest",
            src.to_str().unwrap(),
            "--purpose",
            "t",
            "--tag",
            "C"
        ]
    )
    .status
    .success());
    let daemon = start(&dir, &["--signal-every", "1"]);
    let signal = dir.join(".apparatus/SIGNAL");
    wait_until("the first SIGNAL", 5, || signal.exists());

    // Tamper with the admitted blob: the check fails on every tick.
    let blob = walk_blob(&dir.join(".apparatus/cas/sha256"));
    std::fs::write(&blob, b"tampered").unwrap();
    wait_until("a signal ticket", 10, || {
        open_tickets_titled(&dir, "signal: artefact") == 1
    });
    // Several more ticks: still exactly one ticket, and the daemon is up.
    std::thread::sleep(Duration::from_millis(3_500));
    assert_eq!(open_tickets_titled(&dir, "signal: artefact"), 1);
    let s: serde_json::Value = serde_json::from_slice(&std::fs::read(&signal).unwrap()).unwrap();
    assert_eq!(s["status"], "fail");
    assert_eq!(s["source"], "apparatusd");

    // Repair: the next tick closes the ticket and SIGNAL goes green.
    std::fs::write(&blob, b"fragile").unwrap();
    wait_until("the ticket to close", 10, || {
        open_tickets_titled(&dir, "signal: artefact") == 0
    });
    wait_until("a green SIGNAL", 5, || {
        let s: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&signal).unwrap()).unwrap();
        s["status"] == "ok"
    });
    let all = out(&cli(&dir, &["rws", "tickets"]));
    assert_eq!(
        all.lines()
            .filter(|l| l.contains("signal: artefact"))
            .count(),
        1,
        "{all}"
    );
    terminate(daemon);
}

fn walk_blob(dir: &Path) -> PathBuf {
    let p = std::fs::read_dir(dir)
        .unwrap()
        .next()
        .unwrap_or_else(|| panic!("no blob under {}", dir.display()))
        .unwrap()
        .path();
    if p.is_dir() {
        walk_blob(&p)
    } else {
        p
    }
}

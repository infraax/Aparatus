//! End-to-end tests of the `apparatus rws` commands in temporary directories.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn tempdir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "apparatus-cli-{tag}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_apparatus"))
        .arg("--project")
        .arg(dir)
        .args(args)
        .output()
        .expect("binary runs")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// First token of the last stdout line: the receipt id.
fn last_id(o: &Output) -> String {
    stdout(o)
        .lines()
        .last()
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string()
}

#[test]
fn init_ingest_check_exit_zero() {
    let dir = tempdir("basic");
    fs::write(dir.join("README.md"), b"# demo\n").unwrap();
    assert!(run(&dir, &["rws", "init"]).status.success());
    let readme = dir.join("README.md");
    let ingest = run(
        &dir,
        &[
            "rws",
            "ingest",
            readme.to_str().unwrap(),
            "--purpose",
            "corpus",
            "--tag",
            "C",
        ],
    );
    assert!(
        ingest.status.success(),
        "{}",
        String::from_utf8_lossy(&ingest.stderr)
    );
    // The bytes are in the CAS at their digest path.
    let text = stdout(&ingest);
    let sha = text
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(3)
        .unwrap()
        .to_string();
    let blob = dir
        .join(".apparatus/cas/sha256")
        .join(&sha[0..2])
        .join(&sha[2..4])
        .join(&sha);
    assert_eq!(fs::read(blob).unwrap(), b"# demo\n");
    let check = run(&dir, &["rws", "check"]);
    assert!(check.status.success(), "{}", stdout(&check));
    assert!(stdout(&check).contains("check ok"));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn solo_demo_and_refusals_are_recorded() {
    let dir = tempdir("solo");
    fs::write(dir.join("doc.md"), b"evidence\n").unwrap();
    assert!(run(&dir, &["rws", "init", "--solo"]).status.success());
    let doc = dir.join("doc.md");
    let doc = doc.to_str().unwrap();
    let ingest = run(
        &dir,
        &["rws", "ingest", doc, "--purpose", "corpus", "--tag", "A"],
    );
    let artifact = stdout(&ingest)
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_string();

    // Same bytes again: refused (exit 2) and recorded.
    let again = run(
        &dir,
        &["rws", "ingest", doc, "--purpose", "again", "--tag", "A"],
    );
    assert_eq!(again.status.code(), Some(2));

    let keep = run(&dir, &["rws", "queue", "keep", "--asset", "demo"]);
    assert!(keep.status.success());
    let item = last_id(&keep);

    let ev = run(
        &dir,
        &["rws", "event", "target_dropped", "--subject", &item],
    );
    assert!(
        ev.status.success(),
        "{}",
        String::from_utf8_lossy(&ev.stderr)
    );

    // A keep item through a build gate: refused with K-06.
    let gate = run(
        &dir,
        &[
            "rws",
            "gate",
            "--item",
            &item,
            "--gate",
            "start",
            "--outcome",
            "go",
            "--evidence",
            &artifact,
        ],
    );
    assert_eq!(gate.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&gate.stderr).contains("K-06"));

    // An agent cannot be bound to policy.
    let bind = run(
        &dir,
        &[
            "rws",
            "role-bind",
            "--role",
            "policy",
            "--principal",
            "bot",
            "--kind",
            "agent",
        ],
    );
    assert_eq!(bind.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bind.stderr).contains("R-08"));

    let check = run(&dir, &["rws", "check"]);
    assert!(check.status.success(), "{}", stdout(&check));
    let out = stdout(&check);
    assert!(out.contains("refusals recorded: 3"), "{out}");
    assert!(out.contains("TargetDropped"), "{out}");
    fs::remove_dir_all(dir).ok();
}

#[test]
fn check_fails_on_tampered_ledger() {
    let dir = tempdir("tamper");
    assert!(run(&dir, &["rws", "init", "--solo"]).status.success());
    assert!(run(&dir, &["rws", "queue", "keep", "--asset", "demo"])
        .status
        .success());
    let path = dir.join(".apparatus/ledger.jsonl");
    let text = fs::read_to_string(&path)
        .unwrap()
        .replace("\"asset_ref\":\"demo\"", "\"asset_ref\":\"evil\"");
    fs::write(&path, text).unwrap();
    assert_eq!(run(&dir, &["rws", "check"]).status.code(), Some(1));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn buffer_is_imported_once_then_frozen() {
    let src = tempdir("buffer-src");
    assert!(run(&src, &["rws", "init", "--solo"]).status.success());
    assert!(run(&src, &["rws", "queue", "keep", "--asset", "demo"])
        .status
        .success());

    let dst = tempdir("buffer-dst");
    fs::create_dir_all(dst.join("rws")).unwrap();
    fs::copy(
        src.join(".apparatus/ledger.jsonl"),
        dst.join("rws/receipts.jsonl"),
    )
    .unwrap();

    let init = run(&dst, &["rws", "init", "--solo"]);
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(!dst.join("rws/receipts.jsonl").exists());
    assert!(dst.join("rws/receipts.jsonl.imported").exists());
    assert_eq!(
        fs::read_to_string(src.join(".apparatus/HEAD")).unwrap(),
        fs::read_to_string(dst.join(".apparatus/HEAD")).unwrap()
    );

    // A second import into a live chain is refused.
    fs::copy(src.join(".apparatus/ledger.jsonl"), dst.join("again.jsonl")).unwrap();
    let again = run(
        &dst,
        &[
            "rws",
            "import-jsonl",
            dst.join("again.jsonl").to_str().unwrap(),
        ],
    );
    assert_eq!(again.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&again.stderr).contains("HEAD"));
    assert!(run(&dst, &["rws", "check"]).status.success());
    fs::remove_dir_all(src).ok();
    fs::remove_dir_all(dst).ok();
}

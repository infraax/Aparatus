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

    assert!(
        run(&dir, &["rws", "policy", "agreement", "--asset", "demo"])
            .status
            .success()
    );
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
    assert!(
        run(&dir, &["rws", "policy", "agreement", "--asset", "demo"])
            .status
            .success()
    );
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
    assert!(
        run(&src, &["rws", "policy", "agreement", "--asset", "demo"])
            .status
            .success()
    );
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

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// Second token of the first stdout line (artifact id / envelope id).
fn first_object(o: &Output) -> String {
    stdout(o)
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_string()
}

#[test]
fn keep_is_refused_until_an_agreement_covers_the_asset() {
    let dir = tempdir("k01");
    assert!(run(&dir, &["rws", "init"]).status.success());
    for role in ["policy", "execution", "mandate_holder"] {
        let o = run(
            &dir,
            &["rws", "role-bind", "--role", role, "--principal", "owner"],
        );
        assert!(o.status.success(), "{}", stderr(&o));
    }
    let early = run(&dir, &["rws", "queue", "keep", "--asset", "demo"]);
    assert_eq!(early.status.code(), Some(2));
    assert!(stderr(&early).contains("K-01"));
    assert!(
        stdout(&early).contains(" event "),
        "refusal is recorded on the chain"
    );

    assert!(
        run(&dir, &["rws", "policy", "agreement", "--asset", "demo"])
            .status
            .success()
    );
    assert!(run(&dir, &["rws", "queue", "keep", "--asset", "demo"])
        .status
        .success());
    let other = run(&dir, &["rws", "queue", "keep", "--asset", "other"]);
    assert_eq!(other.status.code(), Some(2));
    assert!(stderr(&other).contains("K-01"));

    let check = stdout(&run(&dir, &["rws", "check"]));
    assert!(check.contains("refusals recorded: 2"), "{check}");
    assert!(check.contains("K-01"), "{check}");
    fs::remove_dir_all(dir).ok();
}

#[test]
fn solo_init_binds_four_roles_and_handover_needs_no_extra_bind() {
    let dir = tempdir("solo-g4");
    let init = run(&dir, &["rws", "init", "--solo"]);
    assert_eq!(
        stdout(&init)
            .lines()
            .filter(|l| l.contains(" role_bind "))
            .count(),
        4
    );
    assert!(stdout(&run(&dir, &["rws", "check"])).contains("role mandate_holder: owner"));

    fs::write(dir.join("dossier.md"), b"project dossier\n").unwrap();
    let dossier = dir.join("dossier.md");
    let ev = first_object(&run(
        &dir,
        &[
            "rws",
            "ingest",
            dossier.to_str().unwrap(),
            "--purpose",
            "gate",
            "--tag",
            "A",
        ],
    ));
    let env = first_object(&run(
        &dir,
        &[
            "rws",
            "policy",
            "envelope",
            "--queue",
            "build",
            "--ceiling",
            "1000",
        ],
    ));
    let agreement = last_id(&run(
        &dir,
        &["rws", "policy", "agreement", "--asset", "bridge"],
    ));
    let item = last_id(&run(&dir, &["rws", "queue", "build", "--asset", "bridge"]));
    for gate in ["start", "preferred", "project"] {
        let o = run(
            &dir,
            &[
                "rws",
                "gate",
                "--item",
                &item,
                "--gate",
                gate,
                "--outcome",
                "go",
                "--evidence",
                &ev,
                "--envelope",
                &env,
                "--required",
                "800",
            ],
        );
        assert!(o.status.success(), "{gate}: {}", stderr(&o));
    }
    let g4 = run(
        &dir,
        &[
            "rws",
            "gate",
            "--item",
            &item,
            "--gate",
            "handover",
            "--outcome",
            "go",
            "--evidence",
            &ev,
            "--discharge",
            "owner",
            "--agreement",
            &agreement,
        ],
    );
    assert!(g4.status.success(), "{}", stderr(&g4));
    assert!(stdout(&run(&dir, &["rws", "show", &item])).contains("state=HandedOver"));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn correction_and_review_commands() {
    let dir = tempdir("corr");
    assert!(run(&dir, &["rws", "init", "--solo"]).status.success());
    fs::write(dir.join("a.md"), b"draft\n").unwrap();
    let a = dir.join("a.md");
    let ingest = run(
        &dir,
        &[
            "rws",
            "ingest",
            a.to_str().unwrap(),
            "--purpose",
            "corpus",
            "--tag",
            "C",
        ],
    );
    let receipt = last_id(&ingest);
    let fix = run(
        &dir,
        &[
            "rws",
            "correction",
            "--corrects",
            &receipt,
            "--reason",
            "tag should be B",
        ],
    );
    assert!(fix.status.success(), "{}", stderr(&fix));
    assert!(stdout(&fix).contains(" correction "));

    // An envelope is created by continuity: execution may not correct it (C-06).
    let env = first_object(&run(
        &dir,
        &[
            "rws",
            "policy",
            "envelope",
            "--queue",
            "build",
            "--ceiling",
            "10",
        ],
    ));
    let low = run(
        &dir,
        &[
            "rws",
            "correction",
            "--corrects",
            &env,
            "--reason",
            "x",
            "--role",
            "execution",
        ],
    );
    assert_eq!(low.status.code(), Some(2));
    assert!(stderr(&low).contains("C-06"));

    assert!(run(&dir, &["rws", "policy", "agreement", "--asset", "*"])
        .status
        .success());
    let item = last_id(&run(&dir, &["rws", "queue", "keep", "--asset", "pump"]));
    let review = run(
        &dir,
        &["rws", "review", "--kind", "condition", "--subject", &item],
    );
    assert!(review.status.success(), "{}", stderr(&review));
    assert!(stdout(&review).contains(" review "));
    // Out of cycle without a named trigger: refused (V-04).
    let ooc = run(
        &dir,
        &[
            "rws",
            "review",
            "--kind",
            "condition",
            "--subject",
            &item,
            "--out-of-cycle",
        ],
    );
    assert_eq!(ooc.status.code(), Some(2));
    assert!(stderr(&ooc).contains("V-04"));
    assert!(run(&dir, &["rws", "check"]).status.success());
    fs::remove_dir_all(dir).ok();
}

#[test]
fn deferring_writes_displacement_then_transition() {
    let dir = tempdir("defer");
    assert!(run(&dir, &["rws", "init", "--solo"]).status.success());
    assert!(run(&dir, &["rws", "policy", "agreement", "--asset", "*"])
        .status
        .success());
    let a = last_id(&run(
        &dir,
        &["rws", "queue", "keep", "--asset", "lock-gate"],
    ));
    let b = last_id(&run(&dir, &["rws", "queue", "keep", "--asset", "bridge"]));
    let defer = run(
        &dir,
        &[
            "rws",
            "queue",
            "keep",
            "--item",
            &a,
            "--to",
            "deferred",
            "--by",
            &b,
            "--reason",
            "bridge first",
        ],
    );
    assert!(defer.status.success(), "{}", stderr(&defer));
    let lines: Vec<String> = stdout(&defer).lines().map(String::from).collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains(" event ") && lines[1].contains(" queue_keep "));
    let check = stdout(&run(&dir, &["rws", "check"]));
    assert!(check.contains("Displacement"), "{check}");
    assert!(check.contains("keep Deferred"), "{check}");
    fs::remove_dir_all(dir).ok();
}

#[test]
fn parallel_ingests_never_corrupt_the_chain() {
    let dir = tempdir("parallel");
    assert!(run(&dir, &["rws", "init", "--solo"]).status.success());
    let n = 8;
    for i in 0..n {
        fs::write(dir.join(format!("f{i}.txt")), format!("file {i}\n")).unwrap();
    }
    // Start every process before waiting on any, so they contend for the lock.
    let children: Vec<_> = (0..n)
        .map(|i| {
            Command::new(env!("CARGO_BIN_EXE_apparatus"))
                .arg("--project")
                .arg(&dir)
                .args(["rws", "ingest"])
                .arg(dir.join(format!("f{i}.txt")))
                .args(["--purpose", "load", "--tag", "C"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    let mut accepted = 0;
    for child in children {
        let o = child.wait_with_output().unwrap();
        match o.status.code() {
            Some(0) => accepted += 1,
            // Only a clean lock timeout is an acceptable failure.
            Some(1) => assert!(
                stderr(&o).contains("another apparatus process"),
                "{}",
                stderr(&o)
            ),
            other => panic!("unexpected exit {other:?}: {}", stderr(&o)),
        }
    }
    assert!(accepted > 0);
    let check = run(&dir, &["rws", "check"]);
    assert!(check.status.success(), "{}", stdout(&check));
    assert!(stdout(&check).contains(&format!("cas ok: {accepted} artefacts verified")));
    let ledger = fs::read_to_string(dir.join(".apparatus/ledger.jsonl")).unwrap();
    assert_eq!(ledger.matches("\"kind\":\"ingest\"").count(), accepted);
    fs::remove_dir_all(dir).ok();
}

fn envelope(dir: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "rws",
        "event-envelope",
        "--source",
        "sensor:bme280",
        "--module",
        "dexos.climate",
        "--provenance",
        "measured",
        "--evidence-tag",
        "C",
    ];
    args.extend_from_slice(extra);
    run(dir, &args)
}

/// The receipt JSON of the last envelope line on the chain.
fn ledger_envelopes(dir: &Path) -> Vec<serde_json::Value> {
    fs::read_to_string(dir.join(".apparatus/ledger.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .map(|r| r["operation_payload"].clone())
        .filter(|p| p["kind"] == "event_envelope")
        .collect()
}

#[test]
fn envelope_receipts_signed_stub_and_draft() {
    let dir = tempdir("envelope");
    assert!(run(&dir, &["rws", "init", "--solo"]).status.success());
    fs::write(dir.join("p.json"), br#"{"temp_c":21,"room":"lab"}"#).unwrap();
    let p = dir.join("p.json");

    let signed = envelope(
        &dir,
        &["--payload", p.to_str().unwrap(), "--event-id", "evt-1"],
    );
    assert!(
        signed.status.success(),
        "{}",
        String::from_utf8_lossy(&signed.stderr)
    );
    let text = stdout(&signed);
    assert!(
        text.starts_with("event_envelope evt-1 cid bafkr4i"),
        "{text}"
    );
    assert!(text.lines().last().unwrap().contains(" event_envelope "));

    let stub = envelope(
        &dir,
        &["--data", r#"{"n":1}"#, "--scheme", "ml-dsa-65-stub"],
    );
    assert!(
        stub.status.success(),
        "{}",
        String::from_utf8_lossy(&stub.stderr)
    );
    let draft = envelope(&dir, &["--data", r#"{"n":1}"#, "--lifecycle", "draft"]);
    assert!(draft.status.success());
    let sealed = envelope(&dir, &["--data", r#"{"n":2}"#, "--lifecycle", "sealed"]);
    assert!(sealed.status.success(), "sealing is stored, not enforced");

    let envs = ledger_envelopes(&dir);
    assert_eq!(envs.len(), 4);
    assert_eq!(envs[0]["event_id"], "evt-1");
    assert_eq!(envs[0]["provenance"], "measured");
    assert_eq!(envs[0]["lifecycle"], "signed");
    assert_eq!(envs[0]["signature_scheme"], "ed25519");
    assert_eq!(envs[0]["payload"]["temp_c"], 21);
    assert!(envs[0]["signature"]["signing_key_id"]
        .as_str()
        .unwrap()
        .starts_with("ARCHIVE-KID-"));
    assert_eq!(envs[1]["signature_scheme"], "ml_dsa_65_stub");
    assert_eq!(envs[2]["lifecycle"], "draft");
    assert!(envs[2]["signature"].is_null());
    assert_eq!(envs[3]["lifecycle"], "sealed");
    // Same payload, same content address.
    assert_eq!(envs[1]["cid"], envs[2]["cid"]);
    assert_eq!(envs[1]["payload_hash"], envs[2]["payload_hash"]);

    // The node key is private to the writer.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(dir.join(".apparatus/keys/ed25519.seed"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    // Replay verifies every signature again: check stays green.
    let check = run(&dir, &["rws", "check"]);
    assert!(check.status.success(), "{}", stdout(&check));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn bad_envelopes_are_refusals_on_the_chain() {
    let dir = tempdir("envbad");
    assert!(run(&dir, &["rws", "init", "--solo"]).status.success());
    assert!(envelope(&dir, &["--data", "{}", "--event-id", "dup"])
        .status
        .success());

    let cases: [(&[&str], &str); 4] = [
        (&["--data", r#"{"x":1.5}"#], "ENV-02"),
        (&["--data", "{}", "--event-id", "dup"], "ENV-05"),
        (&["--data", "{}", "--unix-timestamp", "0"], "ENV-01"),
        (&["--data", "{}", "--event-id", " "], "ENV-01"),
    ];
    for (extra, rule) in cases {
        let o = envelope(&dir, extra);
        assert_eq!(o.status.code(), Some(2), "{rule}: {}", stdout(&o));
        let stderr = String::from_utf8_lossy(&o.stderr);
        assert!(stderr.contains(&format!("REFUSED {rule}")), "{stderr}");
        // The refusal itself is a receipt: one `event` line on stdout.
        assert!(stdout(&o).contains(" event "), "{}", stdout(&o));
    }
    let big = format!(r#"{{"blob":"{}"}}"#, "x".repeat(5000));
    let o = envelope(&dir, &["--data", &big]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("REFUSED ENV-02"));

    // Not JSON at all: a usage error before any receipt, exit 1.
    let o = envelope(&dir, &["--data", "not json"]);
    assert_eq!(o.status.code(), Some(1));

    let check = run(&dir, &["rws", "check"]);
    assert!(check.status.success(), "{}", stdout(&check));
    let text = stdout(&check);
    assert!(text.contains("refusals recorded: 5"), "{text}");
    assert_eq!(
        ledger_envelopes(&dir).len(),
        1,
        "only the good envelope landed"
    );
    fs::remove_dir_all(dir).ok();
}

#[test]
fn sealed_event_envelope_mutation_is_refused_on_chain() {
    let dir = tempdir("envsealed");
    assert!(run(&dir, &["rws", "init", "--solo"]).status.success());
    let sealed = envelope(
        &dir,
        &[
            "--data",
            r#"{"n":1}"#,
            "--event-id",
            "s1",
            "--lifecycle",
            "sealed",
        ],
    );
    assert!(
        sealed.status.success(),
        "{}",
        String::from_utf8_lossy(&sealed.stderr)
    );

    // Same event_id, new content: ENV-07, exit 2, refusal receipt on the chain.
    let o = envelope(&dir, &["--data", r#"{"n":2}"#, "--event-id", "s1"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("REFUSED ENV-07"));
    assert!(stdout(&o).contains(" event "), "{}", stdout(&o));

    // Anchoring is Stage 3: ENV-08.
    let o = envelope(
        &dir,
        &[
            "--data",
            r#"{"n":3}"#,
            "--event-id",
            "a1",
            "--lifecycle",
            "anchored",
        ],
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("REFUSED ENV-08"));

    // Provenance rules at the CLI: measured without a tag, inferred without a source.
    let o = run(
        &dir,
        &[
            "rws",
            "event-envelope",
            "--data",
            "{}",
            "--source",
            "s",
            "--module",
            "m",
            "--provenance",
            "measured",
        ],
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("REFUSED ENV-06"));
    let o = run(
        &dir,
        &[
            "rws",
            "event-envelope",
            "--data",
            "{}",
            "--source",
            "s",
            "--module",
            "m",
            "--provenance",
            "inferred",
        ],
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("REFUSED ENV-06"));

    let check = run(&dir, &["rws", "check"]);
    assert!(check.status.success(), "{}", stdout(&check));
    assert!(
        stdout(&check).contains("refusals recorded: 4"),
        "{}",
        stdout(&check)
    );
    assert_eq!(
        ledger_envelopes(&dir).len(),
        1,
        "only the sealed envelope landed"
    );
    fs::remove_dir_all(dir).ok();
}

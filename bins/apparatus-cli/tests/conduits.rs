//! M4 in local mode (no daemon): ingest drop-pipeline, crash recovery,
//! signals, replica backup and restore.

use apparatus_crypto::Sha256Digest;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn tempdir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("apparatus-m4-{tag}-{}-{nanos}", std::process::id()));
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

fn ok(o: &Output) -> String {
    assert!(
        o.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn init(tag: &str) -> PathBuf {
    let dir = tempdir(tag);
    ok(&run(&dir, &["rws", "init", "--solo"]));
    dir
}

fn pipe(dir: &Path, which: &str) -> PathBuf {
    dir.join(".apparatus/in").join(which)
}

/// Payload files (not results, sidecars or dotfiles) in a pipe folder.
fn payloads(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok()?.file_name().into_string().ok())
                .filter(|n| !n.starts_with('.') && !n.ends_with(".json"))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn result(dir: &Path, name: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(dir.join(format!("{name}.json"))).unwrap()).unwrap()
}

fn head(dir: &Path) -> String {
    ok(&run(dir, &["rws", "head"])).trim().to_string()
}

#[test]
fn drop_three_files_two_ok_one_refuse() {
    let dir = init("pipe");
    ok(&run(&dir, &["rws", "ingest-dir"])); // creates the folders
    let new = pipe(&dir, "new");
    fs::write(new.join("a.md"), b"alpha\n").unwrap();
    fs::write(new.join("a-copy.md"), b"alpha\n").unwrap();
    fs::write(new.join("empty.md"), b"").unwrap();
    fs::write(
        new.join("a.md.rws.json"),
        br#"{"purpose":"corpus","tag":"B"}"#,
    )
    .unwrap();

    let text = ok(&run(&dir, &["rws", "ingest-dir"]));
    assert!(text.contains("pipe idle: new=0 work=0"), "{text}");
    assert_eq!(payloads(&pipe(&dir, "ok")), vec!["a-copy.md", "a.md"]);
    assert_eq!(payloads(&pipe(&dir, "refuse")), vec!["empty.md"]);
    assert!(payloads(&new).is_empty() && payloads(&pipe(&dir, "work")).is_empty());

    let refuse = result(&pipe(&dir, "refuse"), "empty.md");
    assert_eq!(refuse["rule"], "C-03");
    assert!(refuse["receipt"].is_string(), "reject is on the chain");
    let a = result(&pipe(&dir, "ok"), "a.md");
    let copy = result(&pipe(&dir, "ok"), "a-copy.md");
    // Whichever came second is the idempotent no-op: one artefact id, not two.
    let statuses = [
        a["status"].as_str().unwrap(),
        copy["status"].as_str().unwrap(),
    ];
    assert!(statuses.contains(&"admitted") && statuses.contains(&"already_admitted"));
    assert_eq!(
        a["digest"], copy["digest"],
        "same bytes, same digest, one artefact"
    );
    // The sidecar travelled with its file.
    assert!(pipe(&dir, "ok").join("a.md.rws.json").exists());

    let check = ok(&run(&dir, &["rws", "check"]));
    assert!(check.contains("cas ok: 1 artefacts verified"), "{check}");
    assert!(check.contains("check ok"), "{check}");
    // a.md was tagged B; if the copy was admitted first it was untagged → one ticket.
    let tickets = ok(&run(&dir, &["rws", "tickets", "--open"]));
    let untagged = tickets.matches("untagged ingest").count();
    assert!(untagged <= 1, "{tickets}");
}

#[test]
fn untagged_ingest_is_admitted_as_c_with_a_ticket() {
    let dir = init("untagged");
    ok(&run(&dir, &["rws", "ingest-dir"]));
    fs::write(pipe(&dir, "new").join("note.txt"), b"loose note").unwrap();
    ok(&run(&dir, &["rws", "ingest-dir"]));
    let r = result(&pipe(&dir, "ok"), "note.txt");
    assert_eq!(r["status"], "admitted");
    assert_eq!(r["tag"], "C");
    let tickets = ok(&run(&dir, &["rws", "tickets", "--open"]));
    assert!(tickets.contains("untagged ingest: note.txt"), "{tickets}");
}

#[test]
fn crash_leftovers_are_reoffered_once_then_quarantined() {
    let dir = init("crash");
    ok(&run(&dir, &["rws", "ingest-dir"]));
    let work = pipe(&dir, "work");

    // 1. A file the last run left in work/: offered again and admitted.
    fs::write(work.join("left.md"), b"left behind").unwrap();
    // …after its bytes already reached the CAS without a receipt (orphan).
    let digest = Sha256Digest::compute(b"left behind");
    let hex = format!("{digest:x}");
    let blob = dir
        .join(".apparatus/cas/sha256")
        .join(&hex[0..2])
        .join(&hex[2..4])
        .join(&hex);
    fs::create_dir_all(blob.parent().unwrap()).unwrap();
    fs::write(&blob, b"left behind").unwrap();
    let text = ok(&run(&dir, &["rws", "ingest-dir"]));
    assert!(text.contains("re-offering left.md"), "{text}");
    assert_eq!(result(&pipe(&dir, "ok"), "left.md")["status"], "admitted");
    assert!(
        dir.join(".apparatus/cas/orphans")
            .read_dir()
            .unwrap()
            .count()
            == 1,
        "the orphan was set aside, not reused silently"
    );
    assert!(ok(&run(&dir, &["rws", "check"])).contains("check ok"));

    // 2. A file whose retry already died once: quarantined with a ticket.
    fs::write(work.join("poison.md"), b"kills the writer").unwrap();
    fs::write(work.join(".poison.md.retry"), b"").unwrap();
    let text = ok(&run(&dir, &["rws", "ingest-dir"]));
    assert!(text.contains("quarantine poison.md"), "{text}");
    assert_eq!(payloads(&pipe(&dir, "quarantine")), vec!["poison.md"]);
    assert!(!work.join(".poison.md.retry").exists());
    let tickets = ok(&run(&dir, &["rws", "tickets", "--open"]));
    assert!(
        tickets.contains("ingest quarantined: poison.md"),
        "{tickets}"
    );
    // Nothing quarantined was admitted.
    assert!(ok(&run(&dir, &["rws", "check"])).contains("cas ok: 1 artefacts verified"));
}

#[test]
fn bad_sidecar_quarantines() {
    let dir = init("sidecar");
    ok(&run(&dir, &["rws", "ingest-dir"]));
    let new = pipe(&dir, "new");
    fs::write(new.join("x.md"), b"x").unwrap();
    fs::write(new.join("x.md.rws.json"), b"{not json").unwrap();
    ok(&run(&dir, &["rws", "ingest-dir"]));
    assert_eq!(payloads(&pipe(&dir, "quarantine")), vec!["x.md"]);
    let r = result(&pipe(&dir, "quarantine"), "x.md");
    assert!(r["reason"].as_str().unwrap().contains("bad sidecar"));
}

#[test]
fn enqueue_locally_then_ingest_dir() {
    let dir = init("enqueue");
    let src = dir.join("src.md");
    fs::write(&src, b"queued bytes").unwrap();
    let text = ok(&run(
        &dir,
        &["rws", "enqueue", src.to_str().unwrap(), "--tag", "A"],
    ));
    assert!(text.contains("queued in/new/src.md"), "{text}");
    assert!(pipe(&dir, "new").join("src.md.rws.json").exists());
    ok(&run(&dir, &["rws", "ingest-dir"]));
    assert_eq!(result(&pipe(&dir, "ok"), "src.md")["tag"], "A");
}

#[test]
fn signals_local_and_file() {
    let dir = init("signals");
    let text = ok(&run(&dir, &["rws", "signals"]));
    assert!(text.starts_with("signal ok"), "{text}");
    assert!(text.contains("source=local"), "{text}");
    let file = ok(&run(&dir, &["rws", "signals", "--file"]));
    assert!(file.starts_with("signal ok"), "{file}");
    let json: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join(".apparatus/SIGNAL")).unwrap()).unwrap();
    assert_eq!(json["status"], "ok");
    assert!(json["head_hash"].is_string());

    // Corrupt an admitted blob: the signal fails (exit 1) and names the cause.
    let src = dir.join("f.md");
    fs::write(&src, b"fragile").unwrap();
    ok(&run(
        &dir,
        &[
            "rws",
            "ingest",
            src.to_str().unwrap(),
            "--purpose",
            "t",
            "--tag",
            "C",
        ],
    ));
    let hex = format!("{:x}", Sha256Digest::compute(b"fragile"));
    let blob = dir
        .join(".apparatus/cas/sha256")
        .join(&hex[0..2])
        .join(&hex[2..4])
        .join(&hex);
    fs::write(&blob, b"tampered").unwrap();
    let o = run(&dir, &["rws", "signals"]);
    assert_eq!(o.status.code(), Some(1));
    let text = String::from_utf8_lossy(&o.stdout);
    assert!(text.starts_with("signal fail"), "{text}");
    assert!(text.contains("cause artefact"), "{text}");
}

#[test]
fn backup_restore_round_trip() {
    let dir = init("backup");
    let src = dir.join("doc.md");
    fs::write(&src, b"keep me").unwrap();
    ok(&run(
        &dir,
        &[
            "rws",
            "ingest",
            src.to_str().unwrap(),
            "--purpose",
            "t",
            "--tag",
            "B",
        ],
    ));
    let before = head(&dir);
    let bdir = tempdir("backup-dst").join("b1");
    let text = ok(&run(
        &dir,
        &["rws", "backup", "--to", bdir.to_str().unwrap()],
    ));
    assert!(text.contains("replica: true"), "{text}");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(bdir.join("MANIFEST.json")).unwrap()).unwrap();
    assert_eq!(manifest["replica"], true);
    let hash = before.split_whitespace().nth(1).unwrap();
    assert_eq!(manifest["head_hash"], hash);

    // A second backup into the same, now non-empty, directory is refused.
    assert!(
        !run(&dir, &["rws", "backup", "--to", bdir.to_str().unwrap()])
            .status
            .success()
    );

    // Mutate the source after the backup.
    ok(&run(&dir, &["rws", "ticket", "--title", "after backup"]));
    let after = head(&dir);
    assert_ne!(before, after);

    // Restore into an empty project: same head as the backup, check green.
    let target = tempdir("restore");
    let text = ok(&run(
        &target,
        &["rws", "restore", "--from", bdir.to_str().unwrap()],
    ));
    assert!(text.contains("restored"), "{text}");
    assert_eq!(head(&target), before);
    assert!(ok(&run(&target, &["rws", "check"])).contains("check ok"));
    assert!(!target.join(".apparatus-restore").exists());
    // Source unchanged by the restore.
    assert_eq!(head(&dir), after);

    // Never over a live HEAD.
    let o = run(&dir, &["rws", "restore", "--from", bdir.to_str().unwrap()]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("not empty"));
    assert_eq!(head(&dir), after);

    // An interrupted backup (no manifest) is not restorable.
    fs::remove_file(bdir.join("MANIFEST.json")).unwrap();
    let o = run(
        &tempdir("restore2"),
        &["rws", "restore", "--from", bdir.to_str().unwrap()],
    );
    assert!(!o.status.success());
}

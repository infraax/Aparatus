//! Health signals: a periodic `check` whose result lands on the chain and in
//! `.apparatus/SIGNAL`.
//!
//! A failing cause files one ticket titled `signal: <cause>`; while that ticket
//! is open the same cause files nothing more (damping). When the cause is gone
//! the ticket is closed (`report_back_resolved`), so a later relapse files a
//! fresh one. The process never exits over a signal.

use crate::api::{check, execute, ExecOptions, Op, Response};
use crate::kernel::{apparatus_dir, Kernel};
use crate::pipe::{self, file_ticket, QUEUE_FULL};
use anyhow::Result;
use apparatus_hooks::Signal;
use std::fs;
use std::path::{Path, PathBuf};

/// Title prefix of tickets owned by the signal loop.
pub const PREFIX: &str = "signal: ";

pub fn signal_path(root: &Path) -> PathBuf {
    apparatus_dir(root).join("SIGNAL")
}

fn title(cause: &str) -> String {
    let mut t = format!("{PREFIX}{cause}");
    if t.len() > 200 {
        let mut cut = 200;
        while !t.is_char_boundary(cut) {
            cut -= 1;
        }
        t.truncate(cut);
    }
    t
}

/// Snapshot the project: `check` causes plus a full ingest queue.
pub fn compute(k: &Kernel, capacity: usize, source: &str) -> Signal {
    let report = check(k);
    let mut causes: Vec<String> = report
        .lines
        .iter()
        .filter_map(|l| l.strip_prefix("FAIL ").map(String::from))
        .collect();
    let d = pipe::depth(k.root());
    if d.new >= capacity {
        causes.push(QUEUE_FULL.into());
    }
    let head = k.head();
    Signal {
        status: if causes.is_empty() { "ok" } else { "fail" }.into(),
        time_ms: k.now(),
        queue_depth: d.new,
        in_work: d.work,
        quarantined: d.quarantine,
        head_id: head.map(|(id, _)| id.to_string()),
        head_hash: head.map(|(_, h)| format!("{h:x}")),
        receipts: k.entries.len(),
        open_tickets: k.state.open_tickets().len(),
        causes,
        source: source.into(),
    }
}

/// Write `.apparatus/SIGNAL` atomically (JSON, one object).
pub fn write(root: &Path, s: &Signal) -> Result<()> {
    let path = signal_path(root);
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(s)?)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn read(root: &Path) -> Result<Option<Signal>> {
    let path = signal_path(root);
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}

/// Open ticket filed by the signal loop for `cause`, if any.
fn open_signal_ticket(k: &Kernel, cause_title: &str) -> Option<apparatus_types::ObjectId> {
    k.state
        .open_tickets()
        .iter()
        .find(|t| t.ticket.as_ref().map(|x| x.title.as_str()) == Some(cause_title))
        .map(|t| t.id)
}

/// File a ticket for `cause` unless one is already open. Returns a log line
/// when a ticket was filed. The tick closes it once the cause is gone.
pub fn ensure_ticket(k: &mut Kernel, cause: &str, body: Option<String>) -> Option<String> {
    ensure_open_ticket(k, &title(cause), body)
}

/// File a ticket titled exactly `title` unless one with that title is open.
/// Not closed automatically: for failures without a check to turn green.
pub fn ensure_open_ticket(k: &mut Kernel, title: &str, body: Option<String>) -> Option<String> {
    if open_signal_ticket(k, title).is_some() {
        return None;
    }
    Some(file_ticket(k, title.to_string(), None, body))
}

/// One tick: compute, file/close dampened tickets, write SIGNAL, call the hook.
pub fn tick(k: &mut Kernel, capacity: usize, source: &str) -> (Signal, Vec<String>) {
    let first = compute(k, capacity, source);
    let mut lines = vec![];
    let wanted: Vec<String> = first.causes.iter().map(|c| title(c)).collect();
    for cause in &first.causes {
        if let Some(l) = ensure_ticket(
            k,
            cause,
            Some(format!("detected by check at {}", first.time_ms)),
        ) {
            lines.push(l);
        }
    }
    // Causes that are green again: close their tickets.
    let stale: Vec<_> = k
        .state
        .open_tickets()
        .iter()
        .filter_map(|t| {
            let title = &t.ticket.as_ref()?.title;
            (title.starts_with(PREFIX) && !wanted.contains(title)).then(|| (t.id, title.clone()))
        })
        .collect();
    for (id, title) in stale {
        let req = crate::api::Request {
            principal: Some(k.meta.default_principal.clone()),
            role: None,
            producer_kind: None,
            op: Op::TicketClose {
                ticket: id,
                reason: Some("green again".into()),
            },
        };
        let resp = execute(k, req, ExecOptions::default());
        lines.push(if resp.ok {
            format!("closed {id} {title}")
        } else {
            format!("could not close {id}: {}", resp.error.unwrap_or_default())
        });
    }
    // Head and ticket count moved if tickets were filed or closed.
    let mut signal = first;
    signal.head_id = k.head().map(|(id, _)| id.to_string());
    signal.head_hash = k.head().map(|(_, h)| format!("{h:x}"));
    signal.receipts = k.entries.len();
    signal.open_tickets = k.state.open_tickets().len();
    if let Err(e) = write(k.root(), &signal) {
        lines.push(format!("could not write SIGNAL: {e:#}"));
    }
    k.hook().on_signal(&signal);
    (signal, lines)
}

pub fn lines(s: &Signal) -> Vec<String> {
    let mut out = vec![format!(
        "signal {} at {} source={} depth={} work={} quarantine={} receipts={} open_tickets={}",
        s.status,
        s.time_ms,
        s.source,
        s.queue_depth,
        s.in_work,
        s.quarantined,
        s.receipts,
        s.open_tickets
    )];
    out.push(format!(
        "head {} {}",
        s.head_id.as_deref().unwrap_or("-"),
        s.head_hash.as_deref().unwrap_or("-")
    ));
    out.extend(s.causes.iter().map(|c| format!("cause {c}")));
    out
}

/// `signals` op: a fresh snapshot (no tickets filed); also rewrites SIGNAL.
pub fn response(k: &Kernel, capacity: usize, source: &str) -> Response {
    let s = compute(k, capacity, source);
    let _ = write(k.root(), &s);
    let mut r = if s.is_ok() {
        Response::ok(lines(&s))
    } else {
        let mut r = Response::error(format!("signal fail: {}", s.causes.join("; ")));
        r.lines = lines(&s);
        r
    };
    r.data = serde_json::to_value(&s).ok();
    r
}

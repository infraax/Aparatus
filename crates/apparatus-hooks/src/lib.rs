//! Empty conduits: the one place later integrations attach to the runtime.
//!
//! A `Hook` is told what happened after it happened. It never decides, never
//! writes the chain and cannot veto: the chain is the source of truth and a
//! hook is a listener. Zero or one hook per process; no plugin ABI, no dlopen.
//! Future MCP notify, remote wake or object-storage upload implement this
//! trait; none of them is built here.

use apparatus_ledger::Receipt;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Health snapshot written to `.apparatus/SIGNAL` and passed to `on_signal`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signal {
    /// `ok` or `fail`.
    pub status: String,
    /// Unix milliseconds when the snapshot was taken.
    pub time_ms: u64,
    /// Files waiting in `in/new/`.
    pub queue_depth: usize,
    /// Files in `in/work/` (0 or 1 while healthy).
    pub in_work: usize,
    /// Files parked in `in/quarantine/`.
    pub quarantined: usize,
    pub head_id: Option<String>,
    pub head_hash: Option<String>,
    pub receipts: usize,
    pub open_tickets: usize,
    /// One line per failing check cause; empty when `status` is `ok`.
    pub causes: Vec<String>,
    /// `apparatusd` or `local`.
    pub source: String,
}

impl Signal {
    pub fn is_ok(&self) -> bool {
        self.status == "ok"
    }
}

/// Listener for runtime events. Every method defaults to a no-op.
pub trait Hook: Send + Sync {
    /// A receipt was appended to the chain.
    fn after_receipt(&self, _receipt: &Receipt) {}
    /// A write or a dropped file was refused under `rule`.
    fn after_refuse(&self, _rule: &str, _subject: &str) {}
    /// A health signal was computed.
    fn on_signal(&self, _signal: &Signal) {}
    /// A replica backup was completed in `dir`.
    fn after_backup(&self, _dir: &Path) {}
}

/// The default: does nothing.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullHook;

impl Hook for NullHook {}

/// Writes one line per event to stderr (for a daemon's log).
#[derive(Debug, Default, Clone, Copy)]
pub struct LogHook;

impl Hook for LogHook {
    fn after_receipt(&self, receipt: &Receipt) {
        let kind = receipt
            .operation_payload
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("?");
        eprintln!("hook: receipt {} {kind}", receipt.header.id);
    }

    fn after_refuse(&self, rule: &str, subject: &str) {
        eprintln!("hook: refused {rule} {subject}");
    }

    fn on_signal(&self, signal: &Signal) {
        eprintln!(
            "hook: signal {} depth={} receipts={} causes={}",
            signal.status,
            signal.queue_depth,
            signal.receipts,
            signal.causes.len()
        );
    }

    fn after_backup(&self, dir: &Path) {
        eprintln!("hook: backup {}", dir.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Default)]
    struct Counting(AtomicUsize);

    impl Hook for Counting {
        fn after_refuse(&self, _rule: &str, _subject: &str) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn defaults_are_no_ops_and_overrides_fire() {
        let hooks: Vec<Box<dyn Hook>> = vec![Box::new(NullHook), Box::new(LogHook)];
        for h in &hooks {
            h.after_backup(Path::new("/tmp"));
        }
        let c = Counting::default();
        c.after_refuse("C-03", "x");
        c.after_backup(Path::new("/tmp"));
        assert_eq!(c.0.load(Ordering::SeqCst), 1);
    }
}

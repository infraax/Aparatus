//! `apparatus mcp serve`: a stdio MCP server (JSON-RPC 2.0, one message per line) that is
//! only a transport. The control plane is the `mcp-gate` canister on the local replica:
//! before any tool runs, the bridge calls `check(tool, sha256(args))` with the anker key,
//! and the canister records allow/refuse as an Event. Default refuse.
//!
//! Tools are read-only: `chain_head` (apparatusd RPC `head`), `ic_status` (replica status
//! through ic-agent, nothing written), `quarantine_scan` (parse the lockfiles, count pins,
//! no network, nothing written). No install_code, no setAnker, no key export. apparatusd
//! stays the only writer of the chain.

use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// MCP protocol revision this bridge answers with.
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// Decides whether a tool call may run. The real one is the mcp-gate canister.
pub trait Gate {
    /// `Ok((allowed, reason))`; `Err` when the gate cannot be reached (treated as refuse).
    fn check(&self, tool: &str, args: &[u8]) -> Result<(bool, String)>;
}

/// The mcp-gate canister on the local replica.
pub struct CanisterGate {
    pub url: String,
    pub key: PathBuf,
    pub canister: String,
}

impl Gate for CanisterGate {
    fn check(&self, tool: &str, args: &[u8]) -> Result<(bool, String)> {
        let v = apparatus_ic::gate::check(&self.url, &self.key, &self.canister, tool, args)?;
        Ok((
            v.allowed,
            format!("{} (args sha256 {})", v.reason, v.args_sha256),
        ))
    }
}

/// Where the tools read from.
pub struct Ctx {
    pub project: PathBuf,
    pub replica_url: String,
}

struct ToolDef {
    name: &'static str,
    description: &'static str,
}

const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "chain_head",
        description: "Read the Aparatus chain head (receipt id, hash, count) from apparatusd. Read-only.",
    },
    ToolDef {
        name: "ic_status",
        description: "Read the local IC replica status (health, certified height). Read-only; records nothing.",
    },
    ToolDef {
        name: "quarantine_scan",
        description: "Parse the project's lockfiles and count the pinned packages. Read-only; no network; never rewrites a lockfile.",
    },
];

fn run_tool(ctx: &Ctx, name: &str) -> Result<String> {
    match name {
        "chain_head" => chain_head(&ctx.project),
        "ic_status" => {
            let s = apparatus_ic::status(&ctx.replica_url)?;
            Ok(format!(
                "replica {} health {} certified_height {}",
                apparatus_ic::status_url(&ctx.replica_url),
                s.health.unwrap_or_default(),
                s.certified_height
                    .map(|h| h.to_string())
                    .unwrap_or_default()
            ))
        }
        "quarantine_scan" => {
            let mut out = Vec::new();
            for f in apparatus_quarantine::default_lockfiles(&ctx.project) {
                let pins = apparatus_quarantine::parse_lockfile(&f)?;
                out.push(format!("{} pins {}", f.display(), pins.len()));
            }
            if out.is_empty() {
                out.push("no lockfiles".into());
            }
            Ok(out.join("\n"))
        }
        other => anyhow::bail!("no such tool {other}"),
    }
}

#[cfg(unix)]
fn chain_head(project: &Path) -> Result<String> {
    use apparatus_kernel::api::{Op, Request};
    let stream = apparatus_kernel::rpc::connect(project)?
        .ok_or_else(|| anyhow::anyhow!("apparatusd is not running for {}", project.display()))?;
    let mut client = apparatus_kernel::rpc::Client::new(stream)?;
    let resp = client.call(&Request {
        principal: None,
        role: None,
        producer_kind: None,
        op: Op::Head,
    })?;
    if let Some(e) = resp.error {
        anyhow::bail!(e);
    }
    Ok(resp.lines.join("\n"))
}

#[cfg(not(unix))]
fn chain_head(_: &Path) -> Result<String> {
    anyhow::bail!("apparatusd RPC needs a Unix socket")
}

fn text(s: impl Into<String>, is_error: bool) -> Value {
    json!({"content": [{"type": "text", "text": s.into()}], "isError": is_error})
}

/// Handle one JSON-RPC message. `None` for notifications (no reply).
pub fn handle(gate: &dyn Gate, ctx: &Ctx, msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned();
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    let id = id?; // notifications carry no id and get no reply
    let ok = |result: Value| json!({"jsonrpc": "2.0", "id": id, "result": result});
    Some(match method {
        "initialize" => ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "apparatus-mcp", "version": env!("CARGO_PKG_VERSION")},
            "instructions": "Read-only tools. Every call is checked by the mcp-gate canister and recorded there as an Event."
        })),
        "ping" => ok(json!({})),
        "tools/list" => ok(json!({"tools": TOOLS.iter().map(|t| json!({
            "name": t.name,
            "description": t.description,
            "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}
        })).collect::<Vec<_>>()})),
        "tools/call" => {
            let params = msg.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            // serde_json maps are sorted, so the bytes (and their hash) are canonical.
            let bytes = serde_json::to_vec(&args).unwrap_or_default();
            // Ask the gate first, for every name: an unknown tool is refused there and recorded.
            match gate.check(name, &bytes) {
                Err(e) => ok(text(format!("refused: mcp-gate unreachable: {e:#}"), true)),
                Ok((false, reason)) => ok(text(format!("refused by mcp-gate: {reason}"), true)),
                Ok((true, reason)) => {
                    if !TOOLS.iter().any(|t| t.name == name) {
                        ok(text(
                            format!("refused: the bridge has no tool {name} (gate said: {reason})"),
                            true,
                        ))
                    } else {
                        match run_tool(ctx, name) {
                            Ok(out) => ok(text(out, false)),
                            Err(e) => ok(text(format!("tool {name} failed: {e:#}"), true)),
                        }
                    }
                }
            }
        }
        _ => {
            json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("method not found: {method}")}})
        }
    })
}

/// Serve MCP on stdin/stdout until stdin closes.
pub fn serve(gate: &dyn Gate, ctx: &Ctx) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(msg) => handle(gate, ctx, &msg),
            Err(e) => Some(
                json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": format!("parse error: {e}")}}),
            ),
        };
        if let Some(r) = reply {
            writeln!(stdout, "{r}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Allows only the names it was given; records every check like the canister does.
    struct FakeGate {
        allow: Vec<&'static str>,
        seen: RefCell<Vec<(String, bool)>>,
    }
    impl Gate for FakeGate {
        fn check(&self, tool: &str, _args: &[u8]) -> Result<(bool, String)> {
            let ok = self.allow.contains(&tool);
            self.seen.borrow_mut().push((tool.to_string(), ok));
            Ok((
                ok,
                if ok {
                    "registered".into()
                } else {
                    "unknown tool: default refuse".into()
                },
            ))
        }
    }

    fn call(gate: &FakeGate, ctx: &Ctx, name: &str) -> Value {
        handle(gate, ctx, &json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": name, "arguments": {}}})).unwrap()
    }

    fn ctx(project: &Path, url: &str) -> Ctx {
        Ctx {
            project: project.to_path_buf(),
            replica_url: url.to_string(),
        }
    }

    #[test]
    fn lists_three_read_only_tools_and_answers_initialize() {
        let g = FakeGate {
            allow: vec![],
            seen: RefCell::new(vec![]),
        };
        let c = ctx(Path::new("."), "http://127.0.0.1:1");
        let init = handle(
            &g,
            &c,
            &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        )
        .unwrap();
        assert_eq!(init["result"]["protocolVersion"], PROTOCOL_VERSION);
        let list = handle(
            &g,
            &c,
            &json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        )
        .unwrap();
        let names: Vec<_> = list["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(names, ["chain_head", "ic_status", "quarantine_scan"]);
        assert!(handle(
            &g,
            &c,
            &json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
        )
        .is_none());
    }

    #[test]
    fn a_refused_tool_never_runs_and_the_check_is_recorded() {
        let g = FakeGate {
            allow: vec![],
            seen: RefCell::new(vec![]),
        };
        let c = ctx(Path::new("."), "http://127.0.0.1:1");
        let r = call(&g, &c, "install_code");
        assert_eq!(r["result"]["isError"], true);
        assert!(r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("refused by mcp-gate"));
        let r = call(&g, &c, "ic_status");
        assert_eq!(
            r["result"]["isError"], true,
            "ic_status not allowed: must not run"
        );
        assert_eq!(
            *g.seen.borrow(),
            vec![
                ("install_code".to_string(), false),
                ("ic_status".to_string(), false)
            ]
        );
    }

    #[test]
    fn an_allowed_tool_runs_after_the_check() {
        let url = apparatus_ic::testing::serve_status("healthy", 4242);
        let dir = std::env::temp_dir().join(format!("apparatus-mcp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.lock"), "version = 4\n\n[[package]]\nname = \"a\"\nversion = \"1.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"00\"\n").unwrap();
        let before = std::fs::read(dir.join("Cargo.lock")).unwrap();
        let g = FakeGate {
            allow: vec!["ic_status", "quarantine_scan"],
            seen: RefCell::new(vec![]),
        };
        let c = ctx(&dir, &url);
        let r = call(&g, &c, "ic_status");
        assert_eq!(r["result"]["isError"], false, "{r}");
        assert!(r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("certified_height 4242"));
        let r = call(&g, &c, "quarantine_scan");
        assert_eq!(r["result"]["isError"], false, "{r}");
        assert!(r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("pins 1"));
        assert_eq!(
            std::fs::read(dir.join("Cargo.lock")).unwrap(),
            before,
            "lockfile untouched"
        );
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn unknown_method_is_a_json_rpc_error() {
        let g = FakeGate {
            allow: vec![],
            seen: RefCell::new(vec![]),
        };
        let c = ctx(Path::new("."), "http://127.0.0.1:1");
        let r = handle(
            &g,
            &c,
            &json!({"jsonrpc": "2.0", "id": 3, "method": "resources/list"}),
        )
        .unwrap();
        assert_eq!(r["error"]["code"], -32601);
    }
}

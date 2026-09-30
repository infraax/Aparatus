use anyhow::{bail, Context, Result};
use apparatus_kernel::api::{ExecOptions, Op, Request, Response};
use apparatus_kernel::rpc::socket_path;
use apparatus_kernel::{execute, Kernel};
use clap::Parser;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "apparatusd", version = env!("CARGO_PKG_VERSION"), about = "Apparatus daemon: JSONL RPC around one writer")]
struct Args {
    /// Project directory (contains `.apparatus/`).
    #[arg(long, default_value = ".")]
    project: PathBuf,
    /// Requests that may wait for the writer before new ones get `busy`.
    #[arg(long, default_value_t = 64)]
    queue: usize,
    /// How long a request waits to enter a full queue before `busy`.
    #[arg(long, default_value_t = 5_000)]
    enqueue_timeout_ms: u64,
    /// How long a client waits for the writer's answer before `busy`.
    #[arg(long, default_value_t = 60_000)]
    reply_timeout_ms: u64,
}

static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_: libc::c_int) {
    STOP.store(true, Ordering::SeqCst);
}

fn install_signal_handlers() {
    let handler = on_signal as extern "C" fn(libc::c_int) as *const () as libc::sighandler_t;
    // SAFETY: the handler only stores to an atomic, which is async-signal-safe.
    unsafe {
        libc::signal(libc::SIGTERM, handler);
        libc::signal(libc::SIGINT, handler);
    }
}

enum Job {
    Request(Request, SyncSender<Response>),
    Stop,
}

pub fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("apparatusd: {e:#}");
            ExitCode::from(1)
        }
    }
}

fn log(msg: impl AsRef<str>) {
    eprintln!("apparatusd: {}", msg.as_ref());
}

fn run(args: Args) -> Result<()> {
    install_signal_handlers();
    let root = args.project.clone();
    // Taking the kernel takes `.apparatus/LOCK` for the daemon's whole life:
    // local CLI writers are excluded while it runs.
    let kernel = Kernel::open(&root).context("opening project")?;
    let sock = socket_path(&root);
    claim_socket(&sock)?;
    let listener =
        UnixListener::bind(&sock).with_context(|| format!("binding {}", sock.display()))?;
    std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    log(format!(
        "listening on {} ({} receipts)",
        sock.display(),
        kernel.entries.len()
    ));

    let (tx, rx) = mpsc::sync_channel::<Job>(args.queue.max(1));
    let allow_panic = std::env::var_os("APPARATUSD_ALLOW_DEBUG_PANIC").is_some();
    let writer = thread::Builder::new()
        .name("writer".into())
        .spawn(move || writer_loop(kernel, rx, allow_panic))?;

    let enqueue = Duration::from_millis(args.enqueue_timeout_ms);
    let reply = Duration::from_millis(args.reply_timeout_ms);
    while !STOP.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let tx = tx.clone();
                let spawned = thread::Builder::new()
                    .name("conn".into())
                    .spawn(move || serve(stream, tx, enqueue, reply));
                if let Err(e) = spawned {
                    log(format!("could not start connection thread: {e}"));
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(25)),
            Err(e) => {
                log(format!("accept failed: {e}"));
                thread::sleep(Duration::from_millis(100));
            }
        }
    }

    log("stop requested; finishing queued requests");
    std::fs::remove_file(&sock).ok();
    // Queued jobs before `Stop` are answered; later ones see a closed channel.
    let _ = tx.send(Job::Stop);
    drop(tx);
    let _ = writer.join();
    log("stopped");
    Ok(())
}

/// Refuse to start next to a live daemon; remove a stale socket file.
fn claim_socket(sock: &Path) -> Result<()> {
    if !sock.exists() {
        return Ok(());
    }
    match UnixStream::connect(sock) {
        Ok(_) => bail!(
            "another apparatusd is already listening on {}",
            sock.display()
        ),
        Err(_) => {
            log(format!("removing stale socket {}", sock.display()));
            std::fs::remove_file(sock)?;
            Ok(())
        }
    }
}

/// The only place that touches the kernel.
fn writer_loop(mut kernel: Kernel, rx: Receiver<Job>, allow_debug_panic: bool) {
    let opts = ExecOptions {
        strict_principal: true,
        allow_debug_panic,
    };
    while let Ok(job) = rx.recv() {
        let (req, reply) = match job {
            Job::Request(req, reply) => (req, reply),
            Job::Stop => break,
        };
        let op_name = req.op.name();
        let principal = req.principal.clone();
        let outcome = catch_unwind(AssertUnwindSafe(|| execute(&mut kernel, req, opts)));
        let resp = match outcome {
            Ok(resp) => {
                if resp.status == apparatus_kernel::Status::Error && !is_read(&op_name) {
                    record_ticket(
                        &mut kernel,
                        principal,
                        &op_name,
                        resp.error.as_deref(),
                        opts,
                    );
                }
                resp
            }
            Err(panic) => {
                let msg = panic
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| panic.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".into());
                log(format!(
                    "handler for {op_name} panicked: {msg}; reloading chain"
                ));
                if let Err(e) = kernel.reload() {
                    log(format!("reload after panic failed: {e:#}"));
                }
                record_ticket(&mut kernel, principal, &op_name, Some(&msg), opts);
                Response::error(format!("internal error in {op_name}: {msg}"))
            }
        };
        // A client that went away is not our problem.
        let _ = reply.send(resp);
    }
}

fn is_read(op: &str) -> bool {
    matches!(op, "head" | "check" | "show" | "list" | "tickets")
}

/// Unexpected errors become tickets on the chain, filed under the requester.
fn record_ticket(
    kernel: &mut Kernel,
    principal: Option<String>,
    op: &str,
    error: Option<&str>,
    opts: ExecOptions,
) {
    let req = Request {
        principal,
        role: None,
        producer_kind: None,
        op: Op::Ticket {
            title: format!("apparatusd: {op} failed"),
            subject: None,
            body: error.map(|e| e.chars().take(2_000).collect()),
        },
    };
    let resp = execute(kernel, req, opts);
    if !resp.ok {
        log(format!(
            "could not record ticket for failed {op}: {}",
            resp.error.unwrap_or_default()
        ));
    }
}

/// One client connection: read JSON lines, hand each to the writer, answer in order.
fn serve(stream: UnixStream, tx: SyncSender<Job>, enqueue: Duration, reply_wait: Duration) {
    if stream.set_nonblocking(false).is_err() {
        return;
    }
    let Ok(mut out) = stream.try_clone() else {
        return;
    };
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let resp = match serde_json::from_str::<Request>(&line) {
            Err(e) => Response::error(format!("invalid request: {e}")),
            Ok(req) => submit(&tx, req, enqueue, reply_wait),
        };
        let Ok(mut text) = serde_json::to_string(&resp) else {
            break;
        };
        text.push('\n');
        if out
            .write_all(text.as_bytes())
            .and_then(|_| out.flush())
            .is_err()
        {
            break; // client left; the daemon stays
        }
    }
}

fn submit(tx: &SyncSender<Job>, req: Request, enqueue: Duration, reply_wait: Duration) -> Response {
    let (reply_tx, reply_rx) = mpsc::sync_channel(1);
    let mut job = Job::Request(req, reply_tx);
    let deadline = Instant::now() + enqueue;
    loop {
        match tx.try_send(job) {
            Ok(()) => break,
            Err(TrySendError::Full(back)) => {
                if Instant::now() >= deadline {
                    return Response::busy("writer queue is full; retry later");
                }
                job = back;
                thread::sleep(Duration::from_millis(10));
            }
            Err(TrySendError::Disconnected(_)) => return Response::error("daemon is stopping"),
        }
    }
    match reply_rx.recv_timeout(reply_wait) {
        Ok(resp) => resp,
        Err(mpsc::RecvTimeoutError::Timeout) => Response::busy(
            "writer did not answer in time; the request may still be applied — check `head`",
        ),
        Err(mpsc::RecvTimeoutError::Disconnected) => Response::error("daemon is stopping"),
    }
}

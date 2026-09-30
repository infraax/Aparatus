use anyhow::{bail, Context, Result};
use apparatus_kernel::api::{ExecOptions, Op, Request, Response};
use apparatus_kernel::hooks::{Hook, LogHook, NullHook};
use apparatus_kernel::rpc::socket_path;
use apparatus_kernel::{execute, pipe, signal, Kernel};
use clap::Parser;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
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
    /// Seconds between health ticks (check → SIGNAL, dampened tickets).
    #[arg(long, default_value_t = 60)]
    signal_every: u64,
    /// How often the ingest pipe looks at `in/new/`.
    #[arg(long, default_value_t = 1_000)]
    ingest_poll_ms: u64,
    /// Files `in/new/` may hold before `ingest_enqueue` answers `busy`.
    #[arg(long, default_value_t = pipe::DEFAULT_CAPACITY)]
    ingest_capacity: usize,
    /// Log receipts, refusals, signals and backups to stderr (`LogHook`).
    #[arg(long)]
    log_hook: bool,
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
    /// Process one file of the ingest pipe; answers whether there was one.
    IngestStep(SyncSender<bool>),
    /// Health tick.
    Tick,
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
    let mut kernel = Kernel::open(&root).context("opening project")?;
    let hook: Arc<dyn Hook> = if args.log_hook {
        Arc::new(LogHook)
    } else {
        Arc::new(NullHook)
    };
    kernel.set_hook(hook);
    pipe::ensure_dirs(&root)?;
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
    let opts = ExecOptions {
        strict_principal: true,
        allow_debug_panic: std::env::var_os("APPARATUSD_ALLOW_DEBUG_PANIC").is_some(),
        ingest_capacity: Some(args.ingest_capacity),
    };
    let writer = thread::Builder::new()
        .name("writer".into())
        .spawn(move || writer_loop(kernel, rx, opts))?;
    // First tick now: SIGNAL exists as soon as the socket does.
    let _ = tx.try_send(Job::Tick);
    let every = Duration::from_secs(args.signal_every.max(1));
    let ticker = tx.clone();
    thread::Builder::new()
        .name("ticker".into())
        .spawn(move || tick_loop(ticker, every))?;
    let poll = Duration::from_millis(args.ingest_poll_ms.max(10));
    let feeder = tx.clone();
    thread::Builder::new()
        .name("ingest".into())
        .spawn(move || ingest_loop(feeder, poll))?;

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

/// Sleep up to `d`, waking early on stop. Returns false once stop is requested.
fn nap(d: Duration) -> bool {
    let end = Instant::now() + d;
    while Instant::now() < end {
        if STOP.load(Ordering::SeqCst) {
            return false;
        }
        thread::sleep(Duration::from_millis(50).min(d));
    }
    !STOP.load(Ordering::SeqCst)
}

/// Queue a health tick every `every`; a full queue skips that tick.
fn tick_loop(tx: SyncSender<Job>, every: Duration) {
    while nap(every) {
        match tx.try_send(Job::Tick) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => log("writer queue full; skipping this signal tick"),
            Err(TrySendError::Disconnected(_)) => return,
        }
    }
}

/// Feed the ingest pipe one file per job, so client requests interleave with
/// a long batch. At most one ingest job is ever queued.
fn ingest_loop(tx: SyncSender<Job>, poll: Duration) {
    loop {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        match tx.try_send(Job::IngestStep(reply_tx)) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                if !nap(poll) {
                    return;
                }
                continue;
            }
            Err(TrySendError::Disconnected(_)) => return,
        }
        match reply_rx.recv() {
            Ok(true) => {
                if STOP.load(Ordering::SeqCst) {
                    return;
                }
            }
            Ok(false) => {
                if !nap(poll) {
                    return;
                }
            }
            Err(_) => return,
        }
    }
}

fn panic_text(panic: &(dyn std::any::Any + Send)) -> String {
    panic
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".into())
}

/// Background work (ingest step, tick) under the same panic guard as requests.
fn guarded<T>(kernel: &mut Kernel, what: &str, f: impl FnOnce(&mut Kernel) -> T) -> Option<T> {
    match catch_unwind(AssertUnwindSafe(|| f(kernel))) {
        Ok(v) => Some(v),
        Err(panic) => {
            let msg = panic_text(panic.as_ref());
            log(format!("{what} panicked: {msg}; reloading chain"));
            if let Err(e) = kernel.reload() {
                log(format!("reload after panic failed: {e:#}"));
            }
            if let Some(l) = signal::ensure_open_ticket(
                kernel,
                &format!("apparatusd: {what} panicked"),
                Some(msg),
            ) {
                log(l);
            }
            None
        }
    }
}

/// The only place that touches the kernel.
fn writer_loop(mut kernel: Kernel, rx: Receiver<Job>, opts: ExecOptions) {
    while let Ok(job) = rx.recv() {
        let (req, reply) = match job {
            Job::Request(req, reply) => (req, reply),
            Job::IngestStep(reply) => {
                let more = guarded(&mut kernel, "ingest", |k| match pipe::step(k) {
                    Ok(Some(lines)) => {
                        lines.iter().for_each(|l| log(format!("ingest: {l}")));
                        true
                    }
                    Ok(None) => false,
                    Err(e) => {
                        // Folder-level failure (permissions, disk): report, back off.
                        log(format!("ingest pipe: {e:#}"));
                        if let Some(l) = signal::ensure_open_ticket(
                            k,
                            "apparatusd: ingest pipe failing",
                            Some(format!("{e:#}")),
                        ) {
                            log(l);
                        }
                        false
                    }
                });
                let _ = reply.send(more.unwrap_or(true));
                continue;
            }
            Job::Tick => {
                guarded(&mut kernel, "signal tick", |k| {
                    let (s, lines) = signal::tick(k, opts.capacity(), "apparatusd");
                    lines.iter().for_each(|l| log(format!("signal: {l}")));
                    if !s.is_ok() {
                        log(format!("signal fail: {}", s.causes.join("; ")));
                    }
                });
                continue;
            }
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
                let msg = panic_text(panic.as_ref());
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
    matches!(
        op,
        "head" | "check" | "show" | "list" | "tickets" | "signals"
    )
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

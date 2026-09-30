//! Apparatus kernel: the single-writer RWS runtime shared by `apparatus`
//! (CLI, local mode) and `apparatusd` (daemon, RPC mode).

pub mod api;
pub mod backup;
pub mod kernel;
pub mod pipe;
#[cfg(unix)]
pub mod rpc;
pub mod signal;

pub use api::{execute, ExecOptions, ListWhat, Op, Request, Response, Status};
pub use apparatus_hooks as hooks;
pub use kernel::{apparatus_dir, buffer_path, Kernel};

//! Apparatus kernel: the single-writer RWS runtime shared by `apparatus`
//! (CLI, local mode) and `apparatusd` (daemon, RPC mode).

pub mod api;
pub mod kernel;
#[cfg(unix)]
pub mod rpc;

pub use api::{execute, ExecOptions, ListWhat, Op, Request, Response, Status};
pub use kernel::{apparatus_dir, buffer_path, Kernel};

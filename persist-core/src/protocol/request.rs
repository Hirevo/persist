use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::protocol::{LogStreamSource, ProcessSpec};

/// A request to start managing a new process.
///
/// Eventually turned into a `ProcessSpec` internally, once managed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartRequest {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
}

/// A request to start managing a new process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListRequest {
    pub filters: Option<Vec<String>>,
}

/// A request to stop managed processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StopRequest {
    pub filters: Option<Vec<String>>,
}

/// A request to restart managed processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartRequest {
    pub filters: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

/// A request to get information about managed processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoRequest {
    pub name: String,
}

/// A request to stop managing processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteRequest {
    pub filters: Option<Vec<String>>,
}

/// A request to dump the `ProcessSpec` of managed processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DumpRequest {
    pub filters: Option<Vec<String>>,
}

/// A request to restore processes from a previously generated dump.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub specs: Vec<ProcessSpec>,
}

/// A request to restore processes from a previously generated dump.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogsRequest {
    pub filters: Option<Vec<String>>,
    pub source_filter: Option<LogStreamSource>,
    pub stream: bool,
    pub lines: usize,
}
/// A request to prune logs and pid files of unmanaged and/or stopped processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruneRequest {
    /// Prune files from stopped, but still managed, processes.
    pub stopped: bool,
}

/// A request (from a client to the daemon).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum Request {
    List(ListRequest),
    Start(StartRequest),
    Stop(StopRequest),
    Restart(RestartRequest),
    Info(InfoRequest),
    Delete(DeleteRequest),
    Dump(DumpRequest),
    Restore(RestoreRequest),
    Logs(LogsRequest),
    Prune(PruneRequest),
    Version,
    Kill,
}

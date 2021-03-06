use serde::{Deserialize, Serialize};

use crate::protocol::{LogStreamSource, ProcessInfo, ProcessSpec, ProcessStatus};

/// A response to list information and metrics about managed processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListResponse {
    pub name: String,
    pub pid: Option<usize>,
    pub status: ProcessStatus,
    pub cpu_usage: u32,
    pub mem_usage: u32,
}

/// A response to list information and metrics about managed processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartResponse {
    #[serde(flatten)]
    pub spec: ProcessSpec,
}

/// A response to stop managed processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopResponse {
    pub name: String,
    pub error: Option<String>,
}

/// A response to restart managed processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartResponse {
    pub name: String,
    pub error: Option<String>,
}

/// A response to get information about the current state of a managed process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoResponse {
    #[serde(flatten)]
    pub info: ProcessInfo,
}

/// A response to stop managing a process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub name: String,
    pub error: Option<String>,
}

/// A response to dump the current process specifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DumpResponse {
    #[serde(flatten)]
    pub spec: ProcessSpec,
}

/// A response to restore a previously dumped process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreResponse {
    pub name: String,
    pub error: Option<String>,
}

/// A response to get version information about the daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
}

/// A log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    /// Name of the originating process.
    pub name: String,
    /// Source stream of this log entry (stdout or stderr).
    pub source: LogStreamSource,
    /// The actual log message.
    pub msg: String,
}

/// A response for log entries (from the daemon to a client).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum LogsResponse {
    Subscribed,
    Entry(LogEntry),
    Unsubscribed,
}

/// A response to prune unused log files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruneResponse {
    /// List of the names of all the successfully pruned files.
    pub pruned_files: Vec<String>,
}

/// A response (from the daemon to a client).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum Response {
    List(Vec<ListResponse>),
    Start(StartResponse),
    Stop(Vec<StopResponse>),
    Restart(Vec<RestartResponse>),
    Info(InfoResponse),
    Delete(Vec<DeleteResponse>),
    Dump(Vec<DumpResponse>),
    Restore(Vec<RestoreResponse>),
    Version(VersionResponse),
    Logs(LogsResponse),
    Prune(PruneResponse),
    Error(String),
}

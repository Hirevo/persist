use std::collections::HashMap;
use std::fmt::{self, Display};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod request;
mod response;

pub use self::request::*;
pub use self::response::*;

/// The status of a process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessStatus {
    /// The process is running normally.
    Running,
    /// The process is stopped.
    Stopped,
}

impl Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessStatus::Running => write!(f, "running"),
            ProcessStatus::Stopped => write!(f, "stopped"),
        }
    }
}

/// A process specification.
///
/// It is a complete description of a process' environment and configuration.  
/// It can be used to save and restore processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessSpec {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub pid_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub created_at: chrono::NaiveDateTime,
}

/// Information about a current state of a process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub pid: Option<usize>,
    pub status: ProcessStatus,
    pub pid_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub created_at: chrono::NaiveDateTime,
}

impl From<ProcessInfo> for ProcessSpec {
    fn from(info: ProcessInfo) -> ProcessSpec {
        ProcessSpec {
            name: info.name,
            cmd: info.cmd,
            cwd: info.cwd,
            env: info.env,
            pid_path: info.pid_path,
            stdout_path: info.stdout_path,
            stderr_path: info.stderr_path,
            created_at: info.created_at,
        }
    }
}

/// The log stream source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogStreamSource {
    /// Standard output stream.
    Stdout,
    /// Standard error stream.
    Stderr,
}

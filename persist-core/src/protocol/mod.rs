use std::fmt::{self, Display};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessStatus {
    Running,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessSpec {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    pub pid_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub name: String,
    pub pid: Option<usize>,
    pub status: ProcessStatus,
    pub cpu_usage: u32,
    pub mem_usage: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    pub pid: Option<usize>,
    pub status: ProcessStatus,
    pub pid_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProcess {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum Request {
    List,
    Start(NewProcess),
    Stop(String),
    Restart(String),
    Info(String),
    Delete(String),
    Dump(Vec<String>),
    Kill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum Response {
    List(Vec<ProcessMetrics>),
    Start(ProcessInfo),
    Stop,
    Restart(ProcessInfo),
    Info(ProcessInfo),
    Delete,
    Dump(Vec<ProcessSpec>),
    Error(String),
}

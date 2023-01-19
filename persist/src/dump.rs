use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use persist_core::protocol::{ProcessSpec, ProcessStatus};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessDump {
    pub name: String,
    pub cmd: Vec<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub status: ProcessStatus,
    pub pid_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub created_at: chrono::NaiveDateTime,
}

impl From<ProcessSpec> for ProcessDump {
    fn from(spec: ProcessSpec) -> ProcessDump {
        ProcessDump {
            name: spec.name,
            cmd: spec.cmd,
            cwd: spec.cwd,
            env: spec.env.into_iter().collect(),
            status: spec.status,
            pid_path: spec.pid_path,
            stdout_path: spec.stdout_path,
            stderr_path: spec.stderr_path,
            created_at: spec.created_at,
        }
    }
}

impl From<ProcessDump> for ProcessSpec {
    fn from(spec: ProcessDump) -> ProcessSpec {
        ProcessSpec {
            name: spec.name,
            cmd: spec.cmd,
            cwd: spec.cwd,
            env: spec.env.into_iter().collect(),
            status: spec.status,
            pid_path: spec.pid_path,
            stdout_path: spec.stdout_path,
            stderr_path: spec.stderr_path,
            created_at: spec.created_at,
        }
    }
}

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use futures::future;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use heim::process::Process;
use heim::units::information::byte;
use heim::units::ratio;
use tokio::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio::fs::OpenOptions;
use tokio::net::process::Command;
use tokio::sync::Mutex;

use persist_core::daemon::{self, LOGS_DIR, PIDS_DIR};
use persist_core::error::{Error, PersistError};
use persist_core::protocol::{NewProcess, ProcessInfo, ProcessMetrics, ProcessSpec, ProcessStatus};

#[derive(Default)]
pub struct State {
    processes: Mutex<HashMap<String, (ProcessSpec, Option<Process>)>>,
}

impl State {
    pub fn new() -> State {
        State {
            processes: Mutex::new(HashMap::new()),
        }
    }

    pub async fn list(&self) -> Result<Vec<ProcessMetrics>, Error> {
        let processes = self.processes.lock().await;

        let mut metrics = Vec::with_capacity(processes.len());
        for (name, (_, handle)) in processes.iter() {
            let future = async move {
                let (pid, status, cpu_usage, mem_usage) = if let Some(handle) = handle {
                    let pid = handle.pid();
                    let cpu_usage = {
                        let usage1 = handle.cpu_usage().await?;
                        tokio::timer::delay_for(Duration::from_millis(200)).await;
                        let usage2 = handle.cpu_usage().await?;
                        (usage2 - usage1).get::<ratio::percent>()
                    } as u32;
                    let mem_usage = handle.memory().await?.rss().get::<byte>();
                    (
                        Some(pid as usize),
                        ProcessStatus::Running,
                        cpu_usage,
                        mem_usage as u32,
                    )
                } else {
                    (None, ProcessStatus::Stopped, 0u32, 0u32)
                };

                Ok::<ProcessMetrics, Error>(ProcessMetrics {
                    pid,
                    status,
                    cpu_usage,
                    mem_usage,
                    name: name.clone(),
                })
            };
            metrics.push(future);
        }

        let metrics = future::join_all(metrics).await;
        let metrics = metrics.into_iter().collect::<Result<Vec<_>, _>>()?;

        Ok(metrics)
    }

    pub async fn start(self: Arc<Self>, spec: NewProcess) -> Result<ProcessInfo, Error> {
        let mut processes = self.processes.lock().await;

        let name = spec.name.as_str();
        if processes.contains_key(name) {
            return Err(Error::from(PersistError::ProcessAlreadyExists));
        }

        let (cmd, args) = spec.cmd.split_first().expect("empty command");
        let envs = spec.env.clone();

        let home_dir = daemon::home_dir()?;
        let pids_dir = home_dir.join(PIDS_DIR);
        let logs_dir = home_dir.join(LOGS_DIR);

        let _ = tokio::fs::create_dir(&pids_dir).await;
        let _ = tokio::fs::create_dir(&logs_dir).await;

        let pid_path = format!("{}.pid", name);
        let pid_path = pids_dir.join(pid_path);

        let stdout_path = format!("{}-out.log", name);
        let stdout_path = logs_dir.join(stdout_path);
        let stdout_sink = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&stdout_path)
            .await?;
        let mut stdout_sink = FramedWrite::new(stdout_sink, LinesCodec::new());

        let stderr_path = format!("{}-err.log", name);
        let stderr_path = logs_dir.join(stderr_path);
        let stderr_sink = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&stderr_path)
            .await?;
        let mut stderr_sink = FramedWrite::new(stderr_sink, LinesCodec::new());

        let now = chrono::Local::now().naive_local();
        let mut child = Command::new(cmd)
            .args(args)
            .env_clear()
            .envs(envs)
            .current_dir(spec.cwd.as_path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let pid = child.id();
        let handle = heim::process::get(pid as i32).await?;

        tokio::fs::write(pid_path.clone(), pid.to_string()).await?;

        let spec = ProcessSpec {
            name: String::from(name),
            cmd: spec.cmd,
            env: spec.env,
            created_at: now,
            cwd: spec.cwd,
            pid_path: pid_path.canonicalize()?,
            stdout_path: stdout_path.canonicalize()?,
            stderr_path: stderr_path.canonicalize()?,
        };

        processes.insert(String::from(name), (spec.clone(), Some(handle)));

        let stdout = child.stdout().take().unwrap();
        let stderr = child.stderr().take().unwrap();
        let mut stdout = FramedRead::new(stdout, LinesCodec::new());
        let mut stderr = FramedRead::new(stderr, LinesCodec::new());

        tokio::spawn(async move {
            while let Some(Ok(line)) = stdout.next().await {
                let _ = stdout_sink.send(line).await;
            }
        });
        tokio::spawn(async move {
            while let Some(Ok(line)) = stderr.next().await {
                let _ = stderr_sink.send(line).await;
            }
        });

        let name = String::from(name);
        let cloned_self = self.clone();
        tokio::spawn(async move {
            let pid = child.id() as i32;
            let _ = child.await;
            let mut processes = cloned_self.processes.lock().await;
            if let Some((_, handle)) = processes.get_mut(&name) {
                match handle {
                    Some(inner) if pid == inner.pid() => {
                        let _ = handle.take();
                        // TODO: restart process ?
                    }
                    _ => {}
                }
            }
        });

        let info = ProcessInfo {
            name: spec.name,
            cmd: spec.cmd,
            cwd: spec.cwd,
            env: spec.env,
            pid: Some(pid as usize),
            status: ProcessStatus::Running,
            created_at: spec.created_at,
            pid_path: spec.pid_path,
            stdout_path: spec.stdout_path,
            stderr_path: spec.stderr_path,
        };

        Ok(info)
    }

    pub async fn stop(&self, name: String) -> Result<(), Error> {
        let mut processes = self.processes.lock().await;

        let (_, child) = processes
            .get_mut(&name)
            .ok_or_else(|| PersistError::ProcessNotFound)?;

        if let Some(child) = child.take() {
            let _ = child.terminate().await;
        }

        Ok(())
    }

    pub async fn restart(self: Arc<Self>, name: String) -> Result<ProcessInfo, Error> {
        let mut processes = self.processes.lock().await;

        let (spec, og_handle) = processes
            .get_mut(&name)
            .ok_or_else(|| PersistError::ProcessNotFound)?;

        if let Some(child) = og_handle.take() {
            let _ = child.terminate().await;
        }

        let (cmd, args) = spec.cmd.split_first().expect("empty command");
        let envs = spec.env.clone();

        let home_dir = daemon::home_dir()?;
        let pids_dir = home_dir.join(PIDS_DIR);
        let logs_dir = home_dir.join(LOGS_DIR);

        let _ = tokio::fs::create_dir(&pids_dir).await;
        let _ = tokio::fs::create_dir(&logs_dir).await;

        let stdout_sink = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&spec.stdout_path)
            .await?;
        let mut stdout_sink = FramedWrite::new(stdout_sink, LinesCodec::new());

        let stderr_sink = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&spec.stderr_path)
            .await?;
        let mut stderr_sink = FramedWrite::new(stderr_sink, LinesCodec::new());

        let mut child = Command::new(cmd)
            .args(args)
            .env_clear()
            .envs(envs)
            .current_dir(spec.cwd.as_path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let pid = child.id();
        let handle = heim::process::get(pid as i32).await?;

        tokio::fs::write(spec.pid_path.clone(), pid.to_string()).await?;

        let _ = og_handle.replace(handle);

        let stdout = child.stdout().take().unwrap();
        let stderr = child.stderr().take().unwrap();
        let mut stdout = FramedRead::new(stdout, LinesCodec::new());
        let mut stderr = FramedRead::new(stderr, LinesCodec::new());

        tokio::spawn(async move {
            while let Some(Ok(line)) = stdout.next().await {
                let _ = stdout_sink.send(line).await;
            }
        });
        tokio::spawn(async move {
            while let Some(Ok(line)) = stderr.next().await {
                let _ = stderr_sink.send(line).await;
            }
        });

        let cloned_self = self.clone();
        tokio::spawn(async move {
            let pid = child.id() as i32;
            let _ = child.await;
            let mut processes = cloned_self.processes.lock().await;
            if let Some((_, handle)) = processes.get_mut(&name) {
                match handle {
                    Some(inner) if pid == inner.pid() => {
                        let _ = handle.take();
                        // TODO: restart process ?
                    }
                    _ => {}
                }
            }
        });

        let info = ProcessInfo {
            name: spec.name.clone(),
            cmd: spec.cmd.clone(),
            cwd: spec.cwd.clone(),
            env: spec.env.clone(),
            pid: Some(pid as usize),
            status: ProcessStatus::Running,
            created_at: spec.created_at,
            pid_path: spec.pid_path.clone(),
            stdout_path: spec.stdout_path.clone(),
            stderr_path: spec.stderr_path.clone(),
        };

        Ok(info)
    }

    pub async fn delete(&self, name: String) -> Result<(), Error> {
        let mut processes = self.processes.lock().await;

        let (_, handle) = processes
            .remove(&name)
            .ok_or_else(|| PersistError::ProcessNotFound)?;

        if let Some(child) = handle {
            let _ = child.terminate().await;
        }

        Ok(())
    }

    pub async fn info(&self, name: String) -> Result<ProcessInfo, Error> {
        let processes = self.processes.lock().await;

        let (spec, handle) = processes
            .get(&name)
            .ok_or_else(|| PersistError::ProcessNotFound)?;

        let (status, pid) = match handle {
            Some(handle) => (ProcessStatus::Running, Some(handle.pid() as usize)),
            None => (ProcessStatus::Stopped, None),
        };

        let info = ProcessInfo {
            pid,
            status,
            name: spec.name.clone(),
            cmd: spec.cmd.clone(),
            cwd: spec.cwd.clone(),
            env: spec.env.clone(),
            created_at: spec.created_at,
            pid_path: spec.pid_path.clone(),
            stdout_path: spec.stdout_path.clone(),
            stderr_path: spec.stderr_path.clone(),
        };

        Ok(info)
    }

    pub async fn dump(&self, names: Vec<String>) -> Result<Vec<ProcessSpec>, Error> {
        let processes = self.processes.lock().await;

        let specs = if names.is_empty() {
            processes
                .iter()
                .map(|(_, (spec, _))| spec.clone())
                .collect()
        } else {
            processes
                .iter()
                .filter(|(name, _)| names.contains(name))
                .map(|(_, (spec, _))| spec.clone())
                .collect()
        };

        Ok(specs)
    }
}

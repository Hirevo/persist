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
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio::fs::OpenOptions;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use persist_core::error::{Error, PersistError};
use persist_core::protocol::{ProcessInfo, ProcessSpec, ProcessStatus, ListResponse};

#[derive(Default)]
pub struct State {
    processes: Mutex<HashMap<String, (ProcessSpec, Option<Process>)>>,
}

impl State {
    /// Constructs a new `State` instance, with no managed processes.
    pub fn new() -> State {
        State {
            processes: Mutex::new(HashMap::new()),
        }
    }

    /// Gets the process specification associated with the given name.
    pub async fn spec(&self, name: impl AsRef<str>) -> Result<ProcessSpec, Error> {
        let processes = self.processes.lock().await;
        processes
            .get(name.as_ref())
            .map(|(spec, _)| spec.clone())
            .ok_or_else(|| Error::from(PersistError::ProcessNotFound))
    }

    /// Executes a closure and provides it every process handles.
    ///
    /// This closure is executed while holding a lock, so avoid calling other methods on `State` inside that closure.
    pub async fn with_handles<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&HashMap<String, (ProcessSpec, Option<Process>)>) -> T,
    {
        let processes = self.processes.lock().await;

        f(&processes)
    }

    /// Executes a closure and provides it the process handle of the specified process.
    ///
    /// This closure is executed while holding a lock, so avoid calling other methods on `State` inside that closure.
    pub async fn with_handle<S, F, T>(&self, name: S, f: F) -> Result<T, Error>
    where
        S: AsRef<str>,
        F: FnOnce(&(ProcessSpec, Option<Process>)) -> T,
    {
        let processes = self.processes.lock().await;

        let handle = processes
            .get(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        Ok(f(handle))
    }

    /// Spawns a new process according to the given spec, returning the child handle.
    ///
    /// It nullifies stdin and captures stdout and stderr.
    async fn spawn(&self, spec: ProcessSpec) -> Result<Child, Error> {
        let (cmd, args) = spec.cmd.split_first().expect("empty command");
        let envs = spec.env.clone();

        let child = Command::new(cmd)
            .args(args)
            .env_clear()
            .envs(envs)
            .current_dir(spec.cwd.as_path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(child)
    }

    pub async fn list(&self) -> Result<Vec<ListResponse>, Error> {
        let processes = self.processes.lock().await;

        let mut metrics = Vec::with_capacity(processes.len());
        for (name, (_, handle)) in processes.iter() {
            let future = async move {
                let (pid, status, cpu_usage, mem_usage) = if let Some(handle) = handle {
                    let pid = handle.pid();
                    let cpu_usage = {
                        let usage1 = handle.cpu_usage().await?;
                        tokio::time::delay_for(Duration::from_millis(200)).await;
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

                Ok::<ListResponse, Error>(ListResponse {
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

    pub async fn start(self: Arc<Self>, spec: ProcessSpec) -> Result<ProcessInfo, Error> {
        let mut processes = self.processes.lock().await;

        if processes.contains_key(spec.name.as_str()) {
            return Err(Error::from(PersistError::ProcessAlreadyExists));
        }

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

        let mut child = self.spawn(spec.clone()).await?;

        let pid = child.id();
        let handle = heim::process::get(pid as i32).await?;

        tokio::fs::write(spec.pid_path.clone(), pid.to_string()).await?;

        processes.insert(spec.name.clone(), (spec.clone(), Some(handle)));

        let stdout = child.stdout.take().expect("failed to capture stdout");
        let stderr = child.stderr.take().expect("failed to capture stderr");
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

        let name = spec.name.clone();
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

    pub async fn stop(&self, name: impl AsRef<str>) -> Result<(), Error> {
        let mut processes = self.processes.lock().await;

        let (_, child) = processes
            .get_mut(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        if let Some(child) = child.take() {
            let _ = child.terminate().await;
        }

        Ok(())
    }

    pub async fn restart(self: Arc<Self>, spec: ProcessSpec) -> Result<ProcessInfo, Error> {
        let mut processes = self.processes.lock().await;

        let (_, og_handle) = processes
            .get_mut(spec.name.as_str())
            .ok_or(PersistError::ProcessNotFound)?;

        if let Some(child) = og_handle.take() {
            let _ = child.terminate().await;
        }

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

        let mut child = self.spawn(spec.clone()).await?;

        let pid = child.id();
        let handle = heim::process::get(pid as i32).await?;

        tokio::fs::write(spec.pid_path.clone(), pid.to_string()).await?;

        let _ = og_handle.replace(handle);

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
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

        let name = spec.name.clone();
        let cloned_self = self.clone();
        tokio::spawn(async move {
            let pid = child.id() as i32;
            let _ = child.await;
            let mut processes = cloned_self.processes.lock().await;
            if let Some((_, handle)) = processes.get_mut(name.as_str()) {
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

    pub async fn delete(&self, name: impl AsRef<str>) -> Result<(), Error> {
        let mut processes = self.processes.lock().await;

        let (_, handle) = processes
            .remove(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        if let Some(child) = handle {
            let _ = child.terminate().await;
        }

        Ok(())
    }

    pub async fn dump(&self, filters: Option<Vec<String>>) -> Result<Vec<ProcessSpec>, Error> {
        let processes = self.processes.lock().await;

        let specs = match filters {
            Some(filters) => processes
                .iter()
                .filter(|(name, _)| filters.contains(name))
                .map(|(_, (spec, _))| spec.clone())
                .collect(),
            None => processes
                .iter()
                .map(|(_, (spec, _))| spec.clone())
                .collect(),
        };

        Ok(specs)
    }
}

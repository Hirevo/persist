use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::future;
use futures::stream::{Stream, StreamExt};
use heim::units::information::byte;
use heim::units::ratio;
use tokio::sync::Mutex;

use persist_core::error::{Error, PersistError};
use persist_core::protocol::{
    ListResponse, LogEntry, LogStreamSource, ProcessInfo, ProcessSpec, ProcessStatus,
};

use crate::server::handle::ProcessHandle;

#[derive(Default)]
pub struct State {
    processes: Mutex<HashMap<String, ProcessHandle>>,
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
            .map(|handle| handle.spec().clone())
            .ok_or_else(|| Error::from(PersistError::ProcessNotFound))
    }

    /// Executes a closure and provides it every process handles.
    ///
    /// This closure is executed while holding a lock, so avoid calling other methods on `State` inside that closure.
    pub async fn with_handles<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&HashMap<String, ProcessHandle>) -> T,
    {
        let processes = self.processes.lock().await;

        f(&processes)
    }

    /// Executes a closure and provides it the process handle of the specified process.
    ///
    /// This closure is executed while holding a lock, so avoid calling other methods on `State` inside that closure.
    pub async fn with_handle<S, F, T>(&self, name: S, func: F) -> Result<T, Error>
    where
        S: AsRef<str>,
        F: FnOnce(&ProcessHandle) -> T,
    {
        let processes = self.processes.lock().await;

        let handle = processes
            .get(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        Ok(func(handle))
    }

    pub async fn list(&self) -> Result<Vec<ListResponse>, Error> {
        let processes = self.processes.lock().await;

        let futures = processes.iter().map(|(name, handle)| async move {
            let (pid, status, cpu_usage, mem_usage) = handle
                .with_process(|handle| async move {
                    let pid = handle.pid();
                    let cpu_usage = {
                        let usage1 = handle.cpu_usage().await?;
                        tokio::time::delay_for(Duration::from_millis(200)).await;
                        let usage2 = handle.cpu_usage().await?;
                        (usage2 - usage1).get::<ratio::percent>()
                    } as u32;
                    let mem_usage = handle.memory().await?.rss().get::<byte>();

                    Ok::<_, Error>((
                        Some(pid as usize),
                        ProcessStatus::Running,
                        cpu_usage,
                        mem_usage as u32,
                    ))
                })
                .await
                .unwrap_or_else(|| Ok((None, ProcessStatus::Stopped, 0u32, 0u32)))?;

            Ok::<ListResponse, Error>(ListResponse {
                pid,
                status,
                cpu_usage,
                mem_usage,
                name: name.clone(),
            })
        });

        let metrics = future::join_all(futures).await;
        let mut metrics = metrics.into_iter().collect::<Result<Vec<_>, _>>()?;
        metrics.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(metrics)
    }

    pub async fn start(self: Arc<Self>, spec: ProcessSpec) -> Result<ProcessInfo, Error> {
        let mut processes = self.processes.lock().await;

        if processes.contains_key(spec.name.as_str()) {
            return Err(Error::from(PersistError::ProcessAlreadyExists));
        }

        processes.insert(spec.name.clone(), ProcessHandle::new(spec.clone()));
        let handle = processes.get_mut(&spec.name).unwrap();

        let future = handle.start().await?;

        let name = handle.name().to_string();
        let pid = handle.pid().unwrap();
        let cloned_self = self.clone();
        tokio::spawn(async move {
            let _ = future.await;
            let mut processes = cloned_self.processes.lock().await;
            if let Some(handle) = processes.get_mut(name.as_str()) {
                match &mut handle.process {
                    Some(inner) if pid == (inner.pid() as usize) => {
                        let _ = handle.process.take();
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
            pid: Some(pid),
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

        let handle = processes
            .get_mut(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        handle.stop().await?;

        Ok(())
    }

    pub async fn restart(self: Arc<Self>, spec: ProcessSpec) -> Result<ProcessInfo, Error> {
        let mut processes = self.processes.lock().await;

        let handle = processes
            .get_mut(spec.name.as_str())
            .ok_or(PersistError::ProcessNotFound)?;

        let future = handle.restart_with_spec(spec.clone()).await?;

        let name = handle.name().to_string();
        let pid = handle.pid().unwrap();
        let cloned_self = self.clone();
        tokio::spawn(async move {
            let _ = future.await;
            let mut processes = cloned_self.processes.lock().await;
            if let Some(handle) = processes.get_mut(name.as_str()) {
                match &mut handle.process {
                    Some(inner) if pid == (inner.pid() as usize) => {
                        let _ = handle.process.take();
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
            pid: Some(pid),
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

        let mut handle = processes
            .remove(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        handle.stop().await?;

        Ok(())
    }

    pub async fn dump(&self, filters: Option<Vec<String>>) -> Result<Vec<ProcessSpec>, Error> {
        let processes = self.processes.lock().await;

        let specs = match filters {
            Some(filters) => processes
                .iter()
                .filter(|(name, _)| filters.contains(name))
                .map(|(_, handle)| handle.spec().clone())
                .collect(),
            None => processes
                .iter()
                .map(|(_, handle)| handle.spec().clone())
                .collect(),
        };

        Ok(specs)
    }

    pub async fn logs(
        &self,
        names: Vec<String>,
        lines: usize,
        stream: bool,
    ) -> Result<impl Stream<Item = LogEntry>, Error> {
        let processes = self.processes.lock().await;

        let streams = future::try_join_all(
            names
                .into_iter()
                .filter_map(|name| processes.get(&name))
                .map(|handle| async move {
                    let stdout_init = match lines {
                        0 => futures::stream::iter(Vec::new().into_iter().rev()),
                        lines => {
                            let contents = tokio::fs::read_to_string(handle.stdout_file()).await?;
                            let lines = contents
                                .split('\n')
                                .rev()
                                .skip(1)
                                .take(lines)
                                .map(String::from)
                                .collect::<Vec<_>>();
                            futures::stream::iter(lines.into_iter().rev())
                        }
                    };

                    let stderr_init = match lines {
                        0 => futures::stream::iter(Vec::new().into_iter().rev()),
                        lines => {
                            let contents = tokio::fs::read_to_string(handle.stderr_file()).await?;
                            let lines = contents
                                .split('\n')
                                .rev()
                                .skip(1)
                                .take(lines)
                                .map(String::from)
                                .collect::<Vec<_>>();
                            futures::stream::iter(lines.into_iter().rev())
                        }
                    };

                    if stream {
                        let name = handle.name().to_string();
                        let stdout = stdout_init.chain(handle.stdout()).map(move |msg| LogEntry {
                            msg,
                            name: name.clone(),
                            source: LogStreamSource::Stdout,
                        });

                        let name = handle.name().to_string();
                        let stderr = stderr_init.chain(handle.stderr()).map(move |msg| LogEntry {
                            msg,
                            name: name.clone(),
                            source: LogStreamSource::Stderr,
                        });

                        Ok::<Pin<Box<dyn Stream<Item = LogEntry> + Send>>, Error>(Box::pin(
                            futures::stream::select(stdout, stderr),
                        ))
                    } else {
                        let name = handle.name().to_string();
                        let stdout = stdout_init.map(move |msg| LogEntry {
                            msg,
                            name: name.clone(),
                            source: LogStreamSource::Stdout,
                        });

                        let name = handle.name().to_string();
                        let stderr = stderr_init.map(move |msg| LogEntry {
                            msg,
                            name: name.clone(),
                            source: LogStreamSource::Stderr,
                        });

                        Ok::<Pin<Box<dyn Stream<Item = LogEntry> + Send>>, Error>(Box::pin(
                            futures::stream::select(stdout, stderr),
                        ))
                    }
                }),
        )
        .await?;

        Ok(futures::stream::select_all(streams))
    }
}

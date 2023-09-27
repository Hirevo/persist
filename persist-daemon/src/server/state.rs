use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::future;
use futures::stream::{Stream, StreamExt};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use tokio::sync::Mutex;

use persist_core::daemon::{self, LOGS_DIR, PIDS_DIR};
use persist_core::error::{Error, PersistError};
use persist_core::protocol::{
    ListResponse, LogEntry, LogStreamSource, ProcessInfo, ProcessSpec, ProcessStatus,
};

use crate::server::handle::ProcessHandle;

struct Inner {
    system: System,
    processes: HashMap<String, ProcessHandle>,
}

pub struct State {
    inner: Mutex<Inner>,
}

impl State {
    /// Constructs a new `State` instance, with no managed processes.
    pub fn new() -> State {
        State {
            inner: Mutex::new(Inner {
                system: System::new(),
                processes: HashMap::default(),
            }),
        }
    }

    /// Gets the process specification associated with the given name.
    pub async fn spec(&self, name: impl AsRef<str>) -> Result<ProcessSpec, Error> {
        let locked = self.inner.lock().await;
        locked
            .processes
            .get(name.as_ref())
            .map(|handle| handle.spec().clone())
            .ok_or_else(|| Error::from(PersistError::ProcessNotFound))
    }

    /// Executes a closure and provides it every process handles.
    ///
    /// This closure is executed while holding a lock, so avoid calling other methods on `State` inside that closure.
    pub async fn with_handles<F, T>(&self, func: F) -> T
    where
        F: FnOnce(&HashMap<String, ProcessHandle>) -> T,
    {
        let locked = self.inner.lock().await;

        func(&locked.processes)
    }

    /// Executes a closure and provides it the process handle of the specified process.
    ///
    /// This closure is executed while holding a lock, so avoid calling other methods on `State` inside that closure.
    pub async fn with_handle<S, F, T>(&self, name: S, func: F) -> Result<T, Error>
    where
        S: AsRef<str>,
        F: FnOnce(&ProcessHandle) -> T,
    {
        let locked = self.inner.lock().await;

        let handle = locked
            .processes
            .get(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        Ok(func(handle))
    }

    pub async fn list(&self) -> Result<Vec<ListResponse>, Error> {
        let mut locked = self.inner.lock().await;
        let locked = &mut *locked;

        for handle in locked.processes.values() {
            let Some(pid) = handle.pid() else {
                continue;
            };
            let pid = sysinfo::Pid::from_u32(pid as _);
            //? this first refresh adds the process to the list if it isn't there yet.
            locked.system.refresh_process(pid);
            //? first refresh since added, CPU usage will be diff-ed with the next one.
            locked.system.refresh_process(pid);
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
        for handle in locked.processes.values() {
            let Some(pid) = handle.pid() else {
                continue;
            };
            let pid = sysinfo::Pid::from_u32(pid as _);
            //? second refresh since added, the CPU usage diffs are now ready.
            locked.system.refresh_process(pid);
        }

        let mut metrics = locked
            .processes
            .iter()
            .map(|(name, handle)| {
                let (pid, status, cpu_usage, mem_usage) = handle
                    .with_process_sync(|handle| {
                        let pid = sysinfo::Pid::from_u32(handle.pid.as_raw() as _);

                        let Some(handle) = locked.system.process(pid) else {
                            return Ok((
                                Some(handle.pid.as_raw() as usize),
                                ProcessStatus::Running,
                                0,
                                0,
                            ));
                        };

                        let cpu_usage = handle.cpu_usage().round() as u32;
                        let mem_usage = handle.memory();

                        Ok::<_, Error>((
                            Some(pid.as_u32() as usize),
                            ProcessStatus::Running,
                            cpu_usage,
                            mem_usage as u32,
                        ))
                    })
                    .unwrap_or_else(|| Ok((None, ProcessStatus::Stopped, 0u32, 0u32)))?;

                Ok::<ListResponse, Error>(ListResponse {
                    pid,
                    status,
                    cpu_usage,
                    mem_usage,
                    name: name.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        metrics.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(metrics)
    }

    pub async fn start(self: Arc<Self>, mut spec: ProcessSpec) -> Result<ProcessInfo, Error> {
        let mut locked = self.inner.lock().await;

        if locked.processes.contains_key(spec.name.as_str()) {
            return Err(Error::from(PersistError::ProcessAlreadyExists));
        }

        //? get dirs paths
        let home_dir = daemon::home_dir()?;
        let pids_dir = home_dir.join(PIDS_DIR);
        let logs_dir = home_dir.join(LOGS_DIR);

        //? ensure they exist
        let _ = future::join(
            tokio::fs::create_dir(&pids_dir),
            tokio::fs::create_dir(&logs_dir),
        )
        .await;

        //? get PID file path
        let pid_path = format!("{}.pid", spec.name);
        let pid_path = pids_dir.join(pid_path);

        //? get stdout file path
        let stdout_path = format!("{}-out.log", spec.name);
        let stdout_path = logs_dir.join(stdout_path);

        //? get stderr file path
        let stderr_path = format!("{}-err.log", spec.name);
        let stderr_path = logs_dir.join(stderr_path);

        //? ensure they exist
        let _ = future::join3(
            tokio::fs::File::create(pid_path.as_path()),
            tokio::fs::File::create(stdout_path.as_path()),
            tokio::fs::File::create(stderr_path.as_path()),
        )
        .await;

        let now = chrono::Local::now().naive_local();

        spec.created_at = now;
        spec.pid_path = pid_path.canonicalize()?;
        spec.stdout_path = stdout_path.canonicalize()?;
        spec.stderr_path = stderr_path.canonicalize()?;

        locked
            .processes
            .insert(spec.name.clone(), ProcessHandle::new(spec.clone()));

        let info = match spec.status {
            ProcessStatus::Running => {
                let handle = locked.processes.get_mut(&spec.name).unwrap();

                let future = handle.start().await?;

                let name = handle.name().to_string();
                let pid = handle.pid().unwrap();
                let cloned_self = self.clone();
                tokio::spawn(async move {
                    let _ = future.await;
                    let mut locked = cloned_self.inner.lock().await;
                    if let Some(handle) = locked.processes.get_mut(name.as_str()) {
                        if matches!(handle.pid(), Some(inner_pid) if pid == inner_pid) {
                            let _ = handle.process.take();
                            // TODO: restart process ?
                        }
                    }
                });

                ProcessInfo {
                    name: spec.name,
                    cmd: spec.cmd,
                    cwd: spec.cwd,
                    env: spec.env,
                    pid: Some(pid),
                    status: spec.status,
                    created_at: spec.created_at,
                    pid_path: spec.pid_path,
                    stdout_path: spec.stdout_path,
                    stderr_path: spec.stderr_path,
                }
            }
            ProcessStatus::Stopped => ProcessInfo {
                name: spec.name,
                cmd: spec.cmd,
                cwd: spec.cwd,
                env: spec.env,
                pid: None,
                status: spec.status,
                created_at: spec.created_at,
                pid_path: spec.pid_path,
                stdout_path: spec.stdout_path,
                stderr_path: spec.stderr_path,
            },
        };

        Ok(info)
    }

    pub async fn stop(&self, name: impl AsRef<str>) -> Result<(), Error> {
        let mut locked = self.inner.lock().await;

        let handle = locked
            .processes
            .get_mut(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        handle.stop().await?;

        Ok(())
    }

    pub async fn restart(self: Arc<Self>, spec: ProcessSpec) -> Result<ProcessInfo, Error> {
        let mut locked = self.inner.lock().await;

        let handle = locked
            .processes
            .get_mut(spec.name.as_str())
            .ok_or(PersistError::ProcessNotFound)?;

        let future = handle.restart_with_spec(spec.clone()).await?;

        let name = handle.name().to_string();
        let pid = handle.pid().unwrap();
        let cloned_self = self.clone();
        tokio::spawn(async move {
            let _ = future.await;
            let mut locked = cloned_self.inner.lock().await;
            if let Some(handle) = locked.processes.get_mut(name.as_str()) {
                if matches!(handle.pid(), Some(inner_pid) if pid == inner_pid) {
                    let _ = handle.process.take();
                    // TODO: restart process ?
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
        let mut locked = self.inner.lock().await;

        let mut handle = locked
            .processes
            .remove(name.as_ref())
            .ok_or(PersistError::ProcessNotFound)?;

        handle.stop().await?;

        Ok(())
    }

    pub async fn dump(&self, filters: Option<Vec<String>>) -> Result<Vec<ProcessSpec>, Error> {
        let locked = self.inner.lock().await;

        let specs = match filters {
            Some(filters) => locked
                .processes
                .iter()
                .filter(|(name, _)| filters.contains(name))
                .map(|(_, handle)| {
                    let mut spec = handle.spec().clone();
                    spec.status = handle.status();
                    spec
                })
                .collect(),
            None => locked
                .processes
                .iter()
                .map(|(_, handle)| {
                    let mut spec = handle.spec().clone();
                    spec.status = handle.status();
                    spec
                })
                .collect(),
        };

        Ok(specs)
    }

    pub async fn logs(
        &self,
        filters: Option<Vec<String>>,
        lines: usize,
        stream: bool,
        source_filter: Option<LogStreamSource>,
    ) -> Result<impl Stream<Item = LogEntry>, Error> {
        let locked = self.inner.lock().await;

        let streams = future::try_join_all(
            locked
                .processes
                .iter()
                .filter(|(name, _)| filters.as_ref().map_or(true, |names| names.contains(name)))
                .map(|(_, handle)| async move {
                    let stdout_init = match (source_filter, lines) {
                        (Some(LogStreamSource::Stderr), _) | (_, 0) => {
                            futures::stream::empty().right_stream()
                        }
                        (_, lines) => {
                            let contents = tokio::fs::read_to_string(handle.stdout_file()).await?;
                            let lines = contents
                                .split('\n')
                                .rev()
                                .skip(1)
                                .take(lines)
                                .map(String::from)
                                .collect::<Vec<_>>();
                            futures::stream::iter(lines.into_iter().rev()).left_stream()
                        }
                    };

                    let stderr_init = match (source_filter, lines) {
                        (Some(LogStreamSource::Stdout), _) | (_, 0) => {
                            futures::stream::empty().right_stream()
                        }
                        (_, lines) => {
                            let contents = tokio::fs::read_to_string(handle.stderr_file()).await?;
                            let lines = contents
                                .split('\n')
                                .rev()
                                .skip(1)
                                .take(lines)
                                .map(String::from)
                                .collect::<Vec<_>>();
                            futures::stream::iter(lines.into_iter().rev()).left_stream()
                        }
                    };

                    if stream {
                        let name = handle.name().to_string();
                        let stdout = match source_filter {
                            Some(LogStreamSource::Stderr) => {
                                futures::stream::empty().right_stream()
                            }
                            _ => handle.stdout().left_stream(),
                        };
                        let stdout = stdout_init.chain(stdout).map(move |msg| LogEntry {
                            msg,
                            name: name.clone(),
                            source: LogStreamSource::Stdout,
                        });

                        let name = handle.name().to_string();
                        let stderr = match source_filter {
                            Some(LogStreamSource::Stdout) => {
                                futures::stream::empty().right_stream()
                            }
                            _ => handle.stderr().left_stream(),
                        };
                        let stderr = stderr_init.chain(stderr).map(move |msg| LogEntry {
                            msg,
                            name: name.clone(),
                            source: LogStreamSource::Stderr,
                        });

                        Ok::<_, Error>(Box::pin(
                            futures::stream::select(stdout, stderr).right_stream(),
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

                        Ok::<_, Error>(Box::pin(
                            futures::stream::select(stdout, stderr).left_stream(),
                        ))
                    }
                }),
        )
        .await?;

        Ok(futures::stream::select_all(streams))
    }

    pub async fn prune(&self, stopped: bool) -> Result<Vec<String>, Error> {
        let locked = self.inner.lock().await;

        let mut pruned_files = Vec::new();
        let expected_files: Vec<PathBuf> = locked
            .processes
            .values()
            .filter(|handle| !stopped || (stopped && handle.status() == ProcessStatus::Running))
            .flat_map(|handle| {
                let fst = std::iter::once(PathBuf::from(handle.pid_file()));
                let snd = std::iter::once(PathBuf::from(handle.stdout_file()));
                let trd = std::iter::once(PathBuf::from(handle.stderr_file()));
                fst.chain(snd).chain(trd)
            })
            .collect();

        let (logs, pids) =
            future::join(tokio::fs::read_dir(LOGS_DIR), tokio::fs::read_dir(PIDS_DIR)).await;

        // We kind-of ignore errors because the logs and pids directories are only created
        // upon the first process start-up, so it can legitimately not be there yet.
        if let Ok(mut logs) = logs {
            while let Some(dirent) = logs.next_entry().await? {
                // Ignore non-regular files (like directories).
                let kind = dirent.file_type().await?;
                if !kind.is_file() {
                    continue;
                }

                let path = dirent.path().canonicalize()?;
                if !expected_files.contains(&path) {
                    if let Ok(_) = tokio::fs::remove_file(&path).await {
                        pruned_files.push(path.display().to_string());
                    }
                }
            }
        }

        if let Ok(mut pids) = pids {
            while let Some(dirent) = pids.next_entry().await? {
                // Ignore non-regular files (like directories).
                let kind = dirent.file_type().await?;
                if !kind.is_file() {
                    continue;
                }

                let path = dirent.path().canonicalize()?;
                if !expected_files.contains(&path) {
                    if let Ok(_) = tokio::fs::remove_file(&path).await {
                        pruned_files.push(path.display().to_string());
                    }
                }
            }
        }

        Ok(pruned_files)
    }
}

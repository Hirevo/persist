use std::future::Future;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use futures::sink::SinkExt;
use futures::stream::{self, Stream, StreamExt};
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use tokio::fs::OpenOptions;
use tokio::process::Command;
use tokio::sync::{broadcast, Notify};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{ProcessSpec, ProcessStatus};

use crate::server::codec::LogDecoder;

pub struct Inner {
    pub pid: Pid,
    pub ended: Arc<Notify>,
}

pub struct ProcessHandle {
    pub(crate) spec: ProcessSpec,
    pub(crate) process: Option<Inner>,
    pub(crate) stdout: broadcast::Sender<String>,
    pub(crate) stderr: broadcast::Sender<String>,
}

impl ProcessHandle {
    /// Creates a new process handle (but does not spawn the process).
    pub fn new(spec: ProcessSpec) -> Self {
        let (stdout, _) = broadcast::channel(15);
        let (stderr, _) = broadcast::channel(15);
        Self {
            spec,
            stdout,
            stderr,
            process: None,
        }
    }

    pub fn name(&self) -> &str {
        self.spec.name.as_str()
    }

    pub fn spec(&self) -> &ProcessSpec {
        &self.spec
    }

    pub fn status(&self) -> ProcessStatus {
        self.process
            .as_ref()
            .map_or(ProcessStatus::Stopped, |_| ProcessStatus::Running)
    }

    pub fn pid(&self) -> Option<usize> {
        self.process
            .as_ref()
            .map(|handle| handle.pid.as_raw() as usize)
    }

    pub fn pid_file(&self) -> &Path {
        self.spec.pid_path.as_path()
    }

    pub fn stdout_file(&self) -> &Path {
        self.spec.stdout_path.as_path()
    }

    pub fn stderr_file(&self) -> &Path {
        self.spec.stderr_path.as_path()
    }

    pub fn stdout(&self) -> impl Stream<Item = String> {
        stream::unfold(self.stdout.subscribe(), |mut stdout| async move {
            loop {
                match stdout.recv().await {
                    Ok(item) => return Some((item, stdout)),
                    Err(broadcast::error::RecvError::Closed) => return None,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        })
    }

    pub fn stderr(&self) -> impl Stream<Item = String> {
        stream::unfold(self.stderr.subscribe(), |mut stderr| async move {
            loop {
                match stderr.recv().await {
                    Ok(item) => return Some((item, stderr)),
                    Err(broadcast::error::RecvError::Closed) => return None,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        })
    }

    pub async fn start(&mut self) -> Result<impl Future<Output = ()>, Error> {
        let stdout_sink = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.spec.stdout_path)
            .await?;
        let mut stdout_sink = FramedWrite::new(stdout_sink, LinesCodec::new());

        let stderr_sink = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.spec.stderr_path)
            .await?;
        let mut stderr_sink = FramedWrite::new(stderr_sink, LinesCodec::new());

        let (cmd, args) = self.spec.cmd.split_first().expect("empty command");

        let mut child = {
            let mut command = Command::new(cmd);

            command
                .args(args)
                .env_clear()
                .envs(self.spec.env.iter())
                .current_dir(self.spec.cwd.as_path())
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            unsafe {
                command.pre_exec(|| {
                    let pid = nix::unistd::getpid();
                    nix::unistd::setpgid(pid, pid)?;
                    Ok(())
                });
            }

            command.spawn()?
        };

        let pid = match child.id() {
            Some(pid) => pid,
            None => {
                return Err(Error::Other(
                    "process terminated instantly after spawning".into(),
                ));
            }
        };

        let ended = Arc::new(Notify::new());

        let inner = Inner {
            pid: Pid::from_raw(pid as _),
            ended: Arc::clone(&ended),
        };

        tokio::fs::write(self.spec.pid_path.clone(), pid.to_string()).await?;

        let stdout = child.stdout.take().expect("failed to capture stdout");
        let stderr = child.stderr.take().expect("failed to capture stderr");
        let mut stdout = FramedRead::new(stdout, LogDecoder::new());
        let mut stderr = FramedRead::new(stderr, LogDecoder::new());

        let sender = self.stdout.clone();
        tokio::spawn(async move {
            while let Some(item) = stdout.next().await {
                if let Ok(item) = item {
                    let _ = stdout_sink.send(item.clone()).await;
                    let _ = sender.send(item);
                }
            }
        });

        let sender = self.stderr.clone();
        tokio::spawn(async move {
            while let Some(item) = stderr.next().await {
                if let Ok(item) = item {
                    let _ = stderr_sink.send(item.clone()).await;
                    let _ = sender.send(item);
                }
            }
        });

        self.process.replace(inner);

        tokio::spawn({
            let ended = Arc::clone(&ended);
            async move {
                let _ = child.wait().await;
                ended.notify_waiters();
            }
        });

        let future = async move {
            ended.notified().await;
        };

        Ok(future)
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        if let Some(child) = self.process.take() {
            let future = child.ended.notified();
            nix::sys::signal::killpg(child.pid, Signal::SIGTERM)?;
            let _ = future.await;
        }
        Ok(())
    }

    pub async fn restart(&mut self) -> Result<impl Future<Output = ()>, Error> {
        self.stop().await?;
        self.start().await
    }

    pub async fn restart_with_spec(
        &mut self,
        spec: ProcessSpec,
    ) -> Result<impl Future<Output = ()>, Error> {
        self.spec = spec;
        self.restart().await
    }

    pub async fn with_process<'a, F, Fut, T>(&'a self, func: F) -> Option<T>
    where
        F: FnOnce(&'a Inner) -> Fut,
        Fut: Future<Output = T> + 'a,
    {
        if let Some(process) = self.process.as_ref() {
            Some(func(process).await)
        } else {
            None
        }
    }

    pub fn with_process_sync<'a, F, T>(&'a self, func: F) -> Option<T>
    where
        F: FnOnce(&'a Inner) -> T,
    {
        if let Some(process) = self.process.as_ref() {
            Some(func(process))
        } else {
            None
        }
    }
}

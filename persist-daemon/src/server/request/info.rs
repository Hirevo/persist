use std::sync::Arc;

use futures::sink::SinkExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{InfoRequest, InfoResponse, ProcessInfo, ProcessStatus, Response};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    req: InfoRequest,
) -> Result<(), Error> {
    let info = state
        .with_handle(req.name, |(spec, handle)| {
            let (status, pid) = match handle {
                Some(handle) => (ProcessStatus::Running, Some(handle.pid() as usize)),
                None => (ProcessStatus::Stopped, None),
            };

            ProcessInfo {
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
            }
        })
        .await?;

    let response = Response::Info(InfoResponse { info });
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

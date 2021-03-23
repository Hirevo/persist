use std::path::PathBuf;
use std::sync::Arc;

use futures::sink::SinkExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{ProcessSpec, Response, StartRequest, StartResponse};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    spec: StartRequest,
) -> Result<(), Error> {
    let now = chrono::Local::now().naive_local();

    let spec = ProcessSpec {
        name: spec.name,
        cmd: spec.cmd,
        env: spec.env,
        cwd: spec.cwd,
        created_at: now,
        pid_path: PathBuf::new(),
        stdout_path: PathBuf::new(),
        stderr_path: PathBuf::new(),
    };

    //? start the process according to that spec
    let info = state.start(spec).await?;

    let response = Response::Start(StartResponse { spec: info.into() });
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

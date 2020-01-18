use std::sync::Arc;

use futures::sink::SinkExt;
use tokio_util::codec::{Framed, LinesCodec};
use tokio::net::UnixStream;

use persist_core::daemon::{self, LOGS_DIR, PIDS_DIR};
use persist_core::error::Error;
use persist_core::protocol::{ProcessSpec, Response, StartRequest, StartResponse};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    spec: StartRequest,
) -> Result<(), Error> {
    //? get dirs paths
    let home_dir = daemon::home_dir()?;
    let pids_dir = home_dir.join(PIDS_DIR);
    let logs_dir = home_dir.join(LOGS_DIR);

    //? ensure they exists
    let future = futures::future::join(
        tokio::fs::create_dir(&pids_dir),
        tokio::fs::create_dir(&logs_dir),
    );
    let _ = future.await;

    //? get PID file path
    let pid_path = format!("{}.pid", spec.name);
    let pid_path = pids_dir.join(pid_path);

    //? get stdout file path
    let stdout_path = format!("{}-out.log", spec.name);
    let stdout_path = logs_dir.join(stdout_path);

    //? get stderr file path
    let stderr_path = format!("{}-err.log", spec.name);
    let stderr_path = logs_dir.join(stderr_path);

    //? ensure they exists
    let future = futures::future::join3(
        tokio::fs::File::create(pid_path.as_path()),
        tokio::fs::File::create(stdout_path.as_path()),
        tokio::fs::File::create(stderr_path.as_path()),
    );
    let _ = future.await;

    //? construct process spec
    let now = chrono::Local::now().naive_local();
    let spec = ProcessSpec {
        name: spec.name,
        cmd: spec.cmd,
        env: spec.env,
        cwd: spec.cwd,
        created_at: now,
        pid_path: pid_path.canonicalize()?,
        stdout_path: stdout_path.canonicalize()?,
        stderr_path: stderr_path.canonicalize()?,
    };

    //? start the process according to that spec
    state.start(spec.clone()).await?;

    let response = Response::Start(StartResponse { spec });
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

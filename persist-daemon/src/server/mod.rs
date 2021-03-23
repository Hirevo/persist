use std::sync::Arc;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{Framed, LinesCodec};

pub mod codec;
pub mod handle;
pub mod request;
pub mod state;

use persist_core::daemon::{PID_FILE, SOCK_FILE};
use persist_core::error::Error;
use persist_core::protocol::{Request, Response};

use crate::server::request::*;
use crate::server::state::State;

pub async fn handle_conn(state: Arc<State>, conn: UnixStream) -> Result<(), Error> {
    let mut framed = Framed::new(conn, LinesCodec::new());

    while let Some(frame) = framed.next().await {
        let frame = frame?;
        let request = json::from_str::<Request>(frame.as_str());
        let request = match request {
            Ok(request) => request,
            Err(err) => {
                let response = Response::Error(err.to_string());
                let serialized = json::to_string(&response)?;
                let _ = framed.send(serialized).await;
                continue;
            }
        };

        let outcome = match request {
            Request::List(request) => list::handle(state.clone(), &mut framed, request).await,
            Request::Start(request) => start::handle(state.clone(), &mut framed, request).await,
            Request::Stop(request) => stop::handle(state.clone(), &mut framed, request).await,
            Request::Restart(request) => restart::handle(state.clone(), &mut framed, request).await,
            Request::Info(request) => info::handle(state.clone(), &mut framed, request).await,
            Request::Logs(request) => logs::handle(state.clone(), &mut framed, request).await,
            Request::Delete(request) => delete::handle(state.clone(), &mut framed, request).await,
            Request::Dump(request) => dump::handle(state.clone(), &mut framed, request).await,
            Request::Restore(request) => restore::handle(state.clone(), &mut framed, request).await,
            Request::Prune(request) => prune::handle(state.clone(), &mut framed, request).await,
            Request::Version => daemon::version::handle(&mut framed).await,
            Request::Kill => std::process::exit(0),
        };

        if let Err(err) = outcome {
            let response = Response::Error(err.to_string());
            let serialized = json::to_string(&response)?;
            let _ = framed.send(serialized).await;
        }
    }

    Ok(())
}

pub async fn start() -> Result<(), Error> {
    let state = Arc::new(State::new());
    let _ = tokio::fs::remove_file(SOCK_FILE).await;
    let listener = UnixListener::bind(SOCK_FILE)?;

    let pid = std::process::id();
    tokio::fs::write(PID_FILE, pid.to_string()).await?;

    loop {
        if let Ok((conn, _)) = listener.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                if let Err(err) = handle_conn(state, conn).await {
                    // TODO: do something about this error ?
                    eprintln!("conn error: {}", err);
                }
            });
        }
    }
}

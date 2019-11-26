use std::sync::Arc;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::codec::{Framed, LinesCodec};
use tokio::net::{UnixListener, UnixStream};

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
        let request = json::from_str::<Request>(frame.as_str())?;

        match request {
            Request::List => {
                if let Err(err) = list::handle(state.clone(), &mut framed).await {
                    let response = Response::Error(err.to_string());
                    let serialized = json::to_string(&response)?;
                    let _ = framed.send(serialized).await;
                }
            }
            Request::Start(spec) => {
                if let Err(err) = start::handle(state.clone(), &mut framed, spec).await {
                    let response = Response::Error(err.to_string());
                    let serialized = json::to_string(&response)?;
                    let _ = framed.send(serialized).await;
                }
            }
            Request::Stop(name) => {
                if let Err(err) = stop::handle(state.clone(), &mut framed, name).await {
                    let response = Response::Error(err.to_string());
                    let serialized = json::to_string(&response)?;
                    let _ = framed.send(serialized).await;
                }
            }
            Request::Restart(name) => {
                if let Err(err) = restart::handle(state.clone(), &mut framed, name).await {
                    let response = Response::Error(err.to_string());
                    let serialized = json::to_string(&response)?;
                    let _ = framed.send(serialized).await;
                }
            }
            Request::Info(name) => {
                if let Err(err) = info::handle(state.clone(), &mut framed, name).await {
                    let response = Response::Error(err.to_string());
                    let serialized = json::to_string(&response)?;
                    let _ = framed.send(serialized).await;
                }
            }
            Request::Delete(name) => {
                if let Err(err) = delete::handle(state.clone(), &mut framed, name).await {
                    let response = Response::Error(err.to_string());
                    let serialized = json::to_string(&response)?;
                    let _ = framed.send(serialized).await;
                }
            }
            Request::Kill => std::process::exit(0),
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

    let mut incoming = listener.incoming();
    while let Some(conn) = incoming.next().await {
        if let Ok(conn) = conn {
            let state = state.clone();
            tokio::spawn(
                #[allow(clippy::redundant_pattern_matching)]
                async move {
                    if let Err(err) = handle_conn(state, conn).await {
                        // TODO: do something about this error ?
                        eprintln!("conn error: {}", err);
                    }
                },
            );
        }
    }

    Ok(())
}

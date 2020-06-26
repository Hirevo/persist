use std::sync::Arc;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{LogsRequest, LogsResponse, Response};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    req: LogsRequest,
) -> Result<(), Error> {
    let mut logs = state.logs(req.processes, req.lines, req.stream).await?;

    let response = Response::Logs(LogsResponse::Subscribed);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    while let Some(item) = logs.next().await {
        let response = Response::Logs(LogsResponse::Entry(item));
        let serialized = json::to_string(&response)?;
        conn.send(serialized).await?;
    }

    let response = Response::Logs(LogsResponse::Unsubscribed);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

use std::sync::Arc;

use futures::sink::SinkExt;
use tokio::codec::{Framed, LinesCodec};
use tokio::net::UnixStream;

use persist_core::error::Error;
use persist_core::protocol::Response;

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
) -> Result<(), Error> {
    let metrics = state.list().await?;

    let response = Response::List(metrics);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

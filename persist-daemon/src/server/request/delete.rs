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
    name: String,
) -> Result<(), Error> {
    state.delete(name).await?;

    let response = Response::Delete;
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

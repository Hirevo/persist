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
    names: Vec<String>,
) -> Result<(), Error> {
    let specs = state.dump(names).await?;

    let response = Response::Dump(specs);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

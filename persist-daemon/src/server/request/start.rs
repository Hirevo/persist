use std::sync::Arc;

use futures::sink::SinkExt;
use tokio::codec::{Framed, LinesCodec};
use tokio::net::UnixStream;

use persist_core::error::Error;
use persist_core::protocol::{NewProcess, Response};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    new_spec: NewProcess,
) -> Result<(), Error> {
    let spec = state.start(new_spec).await?;

    let response = Response::Start(spec);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

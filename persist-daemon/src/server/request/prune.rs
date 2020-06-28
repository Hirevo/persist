use std::sync::Arc;

use futures::sink::SinkExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{PruneRequest, PruneResponse, Response};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    req: PruneRequest,
) -> Result<(), Error> {
    let pruned_files = state.prune(req.stopped).await?;

    let response = PruneResponse { pruned_files };
    let response = Response::Prune(response);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

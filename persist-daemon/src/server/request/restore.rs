use std::sync::Arc;

use futures::future;
use futures::sink::SinkExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{Response, RestoreRequest, RestoreResponse};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    req: RestoreRequest,
) -> Result<(), Error> {
    let futures = req.specs.into_iter().map(|spec| async {
        let name = spec.name.clone();
        let res = state.clone().start(spec).await;
        let error = res.err().map(|err| err.to_string());
        RestoreResponse { name, error }
    });

    let responses = future::join_all(futures).await;
    let response = Response::Restore(responses);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

use std::sync::Arc;

use futures::sink::SinkExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{DumpRequest, DumpResponse, Response};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    req: DumpRequest,
) -> Result<(), Error> {
    let specs = state.dump(req.filters).await?;

    let responses = specs
        .into_iter()
        .map(|spec| DumpResponse { spec })
        .collect();
    let response = Response::Dump(responses);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

use std::sync::Arc;

use futures::future;
use futures::sink::SinkExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{Response, StopRequest, StopResponse};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    request: StopRequest,
) -> Result<(), Error> {
    let names = match request.filters {
        Some(names) => names,
        None => {
            state
                .with_handles(|handles| handles.keys().cloned().collect())
                .await
        }
    };

    let futures = names.into_iter().map(|name| {
        async {
            let res = state.stop(name.as_str()).await;
            let error = res.err().map(|err| err.to_string());
            StopResponse { name, error }
        }
    });

    let responses = future::join_all(futures).await;
    let response = Response::Stop(responses);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

use std::sync::Arc;

use futures::sink::SinkExt;
use tokio_util::codec::{Framed, LinesCodec};
use tokio::net::UnixStream;

use persist_core::error::Error;
use persist_core::protocol::{DeleteRequest, DeleteResponse, Response};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    req: DeleteRequest,
) -> Result<(), Error> {
    let names = match req.filters {
        Some(filters) => filters,
        None => {
            let future = state.with_handles(|handles| handles.keys().cloned().collect());
            future.await
        }
    };
    let future = futures::future::join_all(names.into_iter().map(|name| async {
        let res = state.delete(name.as_str()).await;
        let error = res.err().map(|err| err.to_string());
        DeleteResponse { name, error }
    }));

    let responses = future.await;
    let response = Response::Delete(responses);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

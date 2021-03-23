use std::sync::Arc;

use futures::future;
use futures::sink::SinkExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{Response, RestartRequest, RestartResponse};

use crate::server::State;

pub async fn handle(
    state: Arc<State>,
    conn: &mut Framed<UnixStream, LinesCodec>,
    request: RestartRequest,
) -> Result<(), Error> {
    let names = match request.filters {
        Some(names) => names,
        None => {
            state
                .with_handles(|handles| handles.keys().cloned().collect())
                .await
        }
    };
    let updated_env = request.env;

    let futures = names.iter().cloned().map(|name| async {
        let mut spec = state.spec(name).await?;
        if let Some(ref env) = updated_env {
            spec.env = env.clone();
        }
        state.clone().restart(spec).await
    });

    let responses = future::join_all(futures).await;
    let responses = responses
        .into_iter()
        .zip(names.into_iter())
        .map(|(res, name)| {
            let error = res.err().map(|err| err.to_string());
            RestartResponse { name, error }
        })
        .collect::<Vec<_>>();

    let response = Response::Restart(responses);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

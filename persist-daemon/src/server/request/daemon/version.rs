use futures::sink::SinkExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::{Response, VersionResponse};

pub async fn handle(conn: &mut Framed<UnixStream, LinesCodec>) -> Result<(), Error> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let response = VersionResponse { version };
    let response = Response::Version(response);
    let serialized = json::to_string(&response)?;
    conn.send(serialized).await?;

    Ok(())
}

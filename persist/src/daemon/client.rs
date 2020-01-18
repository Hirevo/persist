use std::path::Path;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use persist_core::error::Error;
use persist_core::protocol::*;

pub struct DaemonClient {
    socket: Framed<UnixStream, LinesCodec>,
}

impl DaemonClient {
    pub async fn new(socket_path: impl AsRef<Path>) -> Result<DaemonClient, Error> {
        let socket = UnixStream::connect(socket_path).await?;
        let framed = Framed::new(socket, LinesCodec::new());

        Ok(DaemonClient { socket: framed })
    }

    pub async fn kill(&mut self) -> Result<(), Error> {
        let request = Request::Kill;
        let serialized = json::to_string(&request)?;

        self.socket.send(serialized).await?;

        Ok(())
    }

    pub async fn list(&mut self, request: ListRequest) -> Result<Vec<ListResponse>, Error> {
        let request = Request::List(request);
        let serialized = json::to_string(&request)?;

        self.socket.send(serialized).await?;

        let response = if let Some(response) = self.socket.next().await {
            let response = response?;
            json::from_str::<Response>(response.as_str())?
        } else {
            return Err(Error::from(String::from(
                "daemon closed connection without responding",
            )));
        };

        let responses = match response {
            Response::List(responses) => responses,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(responses)
    }

    pub async fn start(&mut self, request: StartRequest) -> Result<StartResponse, Error> {
        let request = Request::Start(request);
        let serialized = json::to_string(&request)?;

        self.socket.send(serialized).await?;

        let response = if let Some(response) = self.socket.next().await {
            let response = response?;
            json::from_str::<Response>(response.as_str())?
        } else {
            return Err(Error::from(String::from(
                "daemon closed connection without responding",
            )));
        };

        let response = match response {
            Response::Start(response) => response,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(response)
    }

    pub async fn stop(&mut self, request: StopRequest) -> Result<Vec<StopResponse>, Error> {
        let request = Request::Stop(request);
        let serialized = json::to_string(&request)?;

        self.socket.send(serialized).await?;

        let response = if let Some(response) = self.socket.next().await {
            let response = response?;
            json::from_str::<Response>(response.as_str())?
        } else {
            return Err(Error::from(String::from(
                "daemon closed connection without responding",
            )));
        };

        let responses = match response {
            Response::Stop(responses) => responses,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(responses)
    }

    pub async fn restart(
        &mut self,
        request: RestartRequest,
    ) -> Result<Vec<RestartResponse>, Error> {
        let request = Request::Restart(request);
        let serialized = json::to_string(&request)?;

        self.socket.send(serialized).await?;

        let response = if let Some(response) = self.socket.next().await {
            let response = response?;
            json::from_str::<Response>(response.as_str())?
        } else {
            return Err(Error::from(String::from(
                "daemon closed connection without responding",
            )));
        };

        let responses = match response {
            Response::Restart(responses) => responses,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(responses)
    }

    pub async fn delete(&mut self, request: DeleteRequest) -> Result<Vec<DeleteResponse>, Error> {
        let request = Request::Delete(request);
        let serialized = json::to_string(&request)?;

        self.socket.send(serialized).await?;

        let response = if let Some(response) = self.socket.next().await {
            let response = response?;
            json::from_str::<Response>(response.as_str())?
        } else {
            return Err(Error::from(String::from(
                "daemon closed connection without responding",
            )));
        };

        let responses = match response {
            Response::Delete(responses) => responses,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(responses)
    }

    pub async fn info(&mut self, request: InfoRequest) -> Result<InfoResponse, Error> {
        let request = Request::Info(request);
        let serialized = json::to_string(&request)?;

        self.socket.send(serialized).await?;

        let response = if let Some(response) = self.socket.next().await {
            let response = response?;
            json::from_str::<Response>(response.as_str())?
        } else {
            return Err(Error::from(String::from(
                "daemon closed connection without responding",
            )));
        };

        let response = match response {
            Response::Info(response) => response,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(response)
    }

    pub async fn dump(&mut self, request: DumpRequest) -> Result<Vec<DumpResponse>, Error> {
        let request = Request::Dump(request);
        let serialized = json::to_string(&request)?;

        self.socket.send(serialized).await?;

        let response = if let Some(response) = self.socket.next().await {
            let response = response?;
            json::from_str::<Response>(response.as_str())?
        } else {
            return Err(Error::from(String::from(
                "daemon closed connection without responding",
            )));
        };

        let responses = match response {
            Response::Dump(responses) => responses,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(responses)
    }
}

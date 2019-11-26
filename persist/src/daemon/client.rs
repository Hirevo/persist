use std::path::Path;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::codec::{Framed, LinesCodec};
use tokio::net::UnixStream;

use persist_core::error::Error;
use persist_core::protocol::{NewProcess, ProcessInfo, ProcessMetrics, Request, Response};

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

    pub async fn list(&mut self) -> Result<Vec<ProcessMetrics>, Error> {
        let request = Request::List;
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

        let metrics = match response {
            Response::List(metrics) => metrics,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(metrics)
    }

    pub async fn start(&mut self, spec: NewProcess) -> Result<ProcessInfo, Error> {
        let request = Request::Start(spec);
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

        let spec = match response {
            Response::Start(spec) => spec,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(spec)
    }

    pub async fn stop(&mut self, name: String) -> Result<(), Error> {
        let request = Request::Stop(name);
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

        match response {
            Response::Stop => (),
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(())
    }

    pub async fn restart(&mut self, name: String) -> Result<ProcessInfo, Error> {
        let request = Request::Restart(name);
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

        let spec = match response {
            Response::Restart(spec) => spec,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(spec)
    }

    pub async fn delete(&mut self, name: String) -> Result<(), Error> {
        let request = Request::Delete(name);
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

        match response {
            Response::Delete => (),
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(())
    }

    pub async fn info(&mut self, name: String) -> Result<ProcessInfo, Error> {
        let request = Request::Info(name);
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

        let spec = match response {
            Response::Info(spec) => spec,
            Response::Error(err) => return Err(Error::from(err)),
            _ => return Err(Error::from(String::from("unexpected response from daemon"))),
        };

        Ok(spec)
    }
}

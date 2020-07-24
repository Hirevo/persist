use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistError {
    #[error("process already exists")]
    ProcessAlreadyExists,
    #[error("process not found")]
    ProcessNotFound,
    #[error("could not find home directory")]
    HomeDirNotFound,
    #[error("could not find any running daemon")]
    DaemonNotFound,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{0}")]
    Heim(#[from] heim::Error),
    #[error("{0}")]
    HeimProcess(#[from] heim::process::ProcessError),
    #[error("{0}")]
    JSON(#[from] json::Error),
    #[error("{0}")]
    Codec(#[from] tokio_util::codec::LinesCodecError),
    #[error("{0}")]
    Nix(#[from] nix::Error),
    #[error("{0}")]
    Persist(#[from] PersistError),
    #[error("{0}")]
    Other(String),
}

impl From<String> for Error {
    fn from(msg: String) -> Error {
        Error::Other(msg)
    }
}

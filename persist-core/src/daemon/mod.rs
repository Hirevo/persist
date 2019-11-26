use std::env;
use std::path::PathBuf;

use crate::error::{Error, PersistError};

pub static PID_FILE: &str = "daemon.pid";
pub static SOCK_FILE: &str = "daemon.sock";

pub static PIDS_DIR: &str = "pids";
pub static LOGS_DIR: &str = "logs";

pub fn home_dir() -> Result<PathBuf, Error> {
    env::var("PERSIST_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".persist")))
        .ok_or_else(|| Error::from(PersistError::HomeDirNotFound))
}

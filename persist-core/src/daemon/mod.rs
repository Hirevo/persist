use std::env;
use std::path::PathBuf;

use crate::error::{Error, PersistError};

pub static PID_FILE: &str = "daemon.pid";
pub static SOCK_FILE: &str = "daemon.sock";

pub static PIDS_DIR: &str = "pids";
pub static LOGS_DIR: &str = "logs";

pub fn home_dir() -> Result<PathBuf, Error> {
    fn recursive_search() -> Result<PathBuf, Error> {
        let current_dir = env::current_dir()?;

        let found = current_dir
            .ancestors()
            .map(|path| path.join(".persist"))
            .find(|path| path.is_dir())
            .ok_or(PersistError::DaemonNotFound)?;

        Ok(found)
    }

    env::var("PERSIST_HOME")
        .map(PathBuf::from)
        .or_else(|_| recursive_search())
}

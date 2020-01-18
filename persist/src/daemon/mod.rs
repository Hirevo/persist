use std::time::Duration;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tokio::process::Command;

pub mod client;

use persist_core::daemon::SOCK_FILE;
use persist_core::error::Error;

use crate::daemon::client::DaemonClient;
use crate::format;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub enum Opts {
    /// Kill the current daemon (will stop all managed processes)
    Kill,
    /// Get version information about the current daemon
    Version,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    match opts {
        Opts::Kill => {
            let mut daemon = self::connect().await?;
            daemon.kill().await?;
            format::success("daemon successfully killed.");
        }
        Opts::Version => {
            let mut daemon = self::connect().await?;
            let response = daemon.version().await?;

            let message = format!("current daemon's version is {}", response.version);
            format::success(message);
        }
    }

    Ok(())
}

pub async fn connect() -> Result<DaemonClient, Error> {
    let home_dir = persist_core::daemon::home_dir()?;
    let socket_path = home_dir.join(SOCK_FILE);

    // if daemon doesn't exists, spawn it.
    let client = match DaemonClient::new(&socket_path).await {
        Ok(client) => client,
        Err(_) => {
            format::info("daemon is not running, spawning it...");
            let mut cur_exe = std::env::current_exe()?;
            cur_exe.set_file_name("persist-daemon");

            // Spawn the daemon.
            // (it is ok to await on it, because it should fork to daemonize early anyway).
            let _ = Command::new(cur_exe).arg("start").spawn()?.await?;

            // Let some time to the daemon to fully initialize its environment.
            tokio::time::delay_for(Duration::from_millis(250)).await;

            let client = DaemonClient::new(&socket_path).await?;
            format::info("daemon spawned and connected.");
            client
        }
    };

    Ok(client)
}

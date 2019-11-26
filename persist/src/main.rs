#[macro_use]
extern crate prettytable;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub mod commands;
pub mod daemon;

use persist_core::error::Error;

use crate::commands::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
#[structopt(about, author)]
pub enum Opts {
    /// Commands to control the daemon (advanced)
    Daemon(daemon::Opts),
    /// Start a new process
    Start(start::Opts),
    /// Stop a running process
    Stop(stop::Opts),
    /// Restart a process
    Restart(restart::Opts),
    /// Get information about a process
    Info(info::Opts),
    /// Delete an existing process
    Delete(delete::Opts),
    /// List all managed processes
    #[structopt(name = "list", alias = "ls")]
    List(list::Opts),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Opts::from_args();

    let outcome = match config {
        Opts::Start(config) => commands::start::handle(config).await,
        Opts::Stop(config) => commands::stop::handle(config).await,
        Opts::Restart(config) => commands::restart::handle(config).await,
        Opts::Info(config) => commands::info::handle(config).await,
        Opts::Delete(config) => commands::delete::handle(config).await,
        Opts::List(config) => commands::list::handle(config).await,
        Opts::Daemon(config) => daemon::handle(config).await,
    };

    if let Err(err) = outcome {
        eprintln!("error: {}", err);
    }

    Ok(())
}

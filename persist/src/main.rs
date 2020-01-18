#[macro_use]
extern crate prettytable;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub mod commands;
pub mod daemon;
pub mod dump;
pub mod format;

use persist_core::error::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
#[structopt(about, author)]
pub enum Opts {
    /// Commands to control the daemon (advanced)
    Daemon(daemon::Opts),
    /// Start a new process
    Start(commands::start::Opts),
    /// Stop a running process
    Stop(commands::stop::Opts),
    /// Restart a process
    #[structopt(alias = "reload")]
    Restart(commands::restart::Opts),
    /// Get information about a process
    Info(commands::info::Opts),
    /// Delete an existing process
    Delete(commands::delete::Opts),
    /// List all managed processes
    #[structopt(alias = "ls")]
    List(commands::list::Opts),
    /// Dump configurations of currently managed processes
    Dump(commands::dump::Opts),
    /// Restore previously dumped processes
    Restore(commands::restore::Opts),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Opts::from_args();

    let outcome = match config {
        Opts::Daemon(config) => daemon::handle(config).await,
        Opts::Start(config) => commands::start::handle(config).await,
        Opts::Stop(config) => commands::stop::handle(config).await,
        Opts::Restart(config) => commands::restart::handle(config).await,
        Opts::Info(config) => commands::info::handle(config).await,
        Opts::Delete(config) => commands::delete::handle(config).await,
        Opts::List(config) => commands::list::handle(config).await,
        Opts::Dump(config) => commands::dump::handle(config).await,
        Opts::Restore(config) => commands::restore::handle(config).await,
    };

    if let Err(err) = outcome {
        format::error(format!("{}.", err));
    }

    Ok(())
}

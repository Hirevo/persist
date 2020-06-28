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
    /// Access process logs
    Logs(commands::logs::Opts),
    /// Dump configurations of currently managed processes
    Dump(commands::dump::Opts),
    /// Restore previously dumped processes
    Restore(commands::restore::Opts),
    /// Prune outdated process logs and pid files
    Prune(commands::prune::Opts),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = Opts::from_args();

    let outcome = match opts {
        Opts::Daemon(opts) => daemon::handle(opts).await,
        Opts::Start(opts) => commands::start::handle(opts).await,
        Opts::Stop(opts) => commands::stop::handle(opts).await,
        Opts::Restart(opts) => commands::restart::handle(opts).await,
        Opts::Info(opts) => commands::info::handle(opts).await,
        Opts::Delete(opts) => commands::delete::handle(opts).await,
        Opts::List(opts) => commands::list::handle(opts).await,
        Opts::Logs(opts) => commands::logs::handle(opts).await,
        Opts::Dump(opts) => commands::dump::handle(opts).await,
        Opts::Restore(opts) => commands::restore::handle(opts).await,
        Opts::Prune(opts) => commands::prune::handle(opts).await,
    };

    if let Err(err) = outcome {
        format::error(format!("{}.", err));
    }

    Ok(())
}

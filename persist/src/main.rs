use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod daemon;
pub mod dump;
pub mod format;

use persist_core::error::Error;

use crate::format::TableStyle;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
pub struct GlobalOpts {
    #[arg(long, env = "PERSIST_TABLE_STYLE", default_value = "unicode")]
    pub table_style: TableStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
#[command(about, author)]
pub struct Opts {
    #[command(flatten)]
    pub global: GlobalOpts,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Subcommand)]
pub enum Command {
    /// Commands to control the daemon (advanced)
    #[command(subcommand)]
    Daemon(daemon::Opts),
    /// Start a new process
    Start(commands::start::Opts),
    /// Stop a running process
    Stop(commands::stop::Opts),
    /// Restart a process
    #[command(alias = "reload")]
    Restart(commands::restart::Opts),
    /// Get information about a process
    Info(commands::info::Opts),
    /// Initialize a new `persist` workspace in the current directory
    Init(commands::init::Opts),
    /// Delete an existing process
    Delete(commands::delete::Opts),
    /// List all managed processes
    #[command(alias = "ls")]
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
    let opts = Opts::parse();

    let global_opts = opts.global;

    let outcome = match opts.command {
        Command::Daemon(opts) => daemon::handle(global_opts, opts).await,
        Command::Start(opts) => commands::start::handle(global_opts, opts).await,
        Command::Stop(opts) => commands::stop::handle(global_opts, opts).await,
        Command::Restart(opts) => commands::restart::handle(global_opts, opts).await,
        Command::Info(opts) => commands::info::handle(global_opts, opts).await,
        Command::Init(opts) => commands::init::handle(global_opts, opts).await,
        Command::Delete(opts) => commands::delete::handle(global_opts, opts).await,
        Command::List(opts) => commands::list::handle(global_opts, opts).await,
        Command::Logs(opts) => commands::logs::handle(global_opts, opts).await,
        Command::Dump(opts) => commands::dump::handle(global_opts, opts).await,
        Command::Restore(opts) => commands::restore::handle(global_opts, opts).await,
        Command::Prune(opts) => commands::prune::handle(global_opts, opts).await,
    };

    if let Err(err) = outcome {
        format::error(format!("{}.", err));
    }

    Ok(())
}

use clap::Parser;
use serde::{Deserialize, Serialize};

use persist_core::error::Error;
use persist_core::protocol::RestartRequest;

use crate::format;
use crate::{daemon, GlobalOpts};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
pub struct Opts {
    /// Restart all processes
    #[arg(long)]
    pub all: bool,
    /// The names of the processes to restart
    #[arg(name = "PROCESS_NAMES")]
    pub processes: Vec<String>,
    /// Update the processes' environments with the current one
    #[arg(long)]
    pub update_env: bool,
}

pub async fn handle(_: GlobalOpts, opts: Opts) -> Result<(), Error> {
    let filters = match (opts.all, opts.processes) {
        (false, processes) if processes.is_empty() => {
            return Err(Error::from(String::from(
                "you must specify at least one process name or --all",
            )));
        }
        (false, processes) => Some(processes),
        (true, _) => None,
    };
    let env = if opts.update_env {
        Some(std::env::vars().collect())
    } else {
        None
    };
    let request = RestartRequest { filters, env };

    let mut daemon = daemon::connect().await?;
    let responses = daemon.restart(request).await?;
    for response in responses {
        if let Some(error) = response.error {
            let msg = format!(
                "process '{}' could not be restarted: {}",
                response.name, error
            );
            format::error(msg);
        } else {
            let msg = format!("process '{}' successfully restarted.", response.name);
            format::success(msg);
        }
    }

    Ok(())
}

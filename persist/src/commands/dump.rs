use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;

use crate::daemon;
use crate::dump::ProcessDump;
use crate::format;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// Dump the configuration of all the processes
    #[structopt(long)]
    pub all: bool,
    /// The name of the processes to dump configurations for
    #[structopt(name = "process-name")]
    pub processes: Vec<String>,
    /// The path of the file to store the dump into
    #[structopt(short, long, default_value = "persist-save.json")]
    pub out: PathBuf,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    let processes = match (opts.all, opts.processes) {
        (false, processes) if processes.is_empty() => {
            return Err(Error::from(String::from(
                "you must specify at least one process name or --all",
            )));
        }
        (false, processes) => processes,
        (true, _) => Vec::new(),
    };

    let mut daemon = daemon::connect().await?;
    let specs = daemon.dump(processes).await?;

    let dumps = specs.into_iter().map(ProcessDump::from).collect::<Vec<_>>();

    let serialized = json::to_string_pretty(&dumps)?;
    tokio::fs::write(opts.out, serialized).await?;

    format::success("successfully dumped process configurations.");

    Ok(())
}

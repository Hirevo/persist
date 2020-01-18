use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;
use persist_core::protocol::DumpRequest;

use crate::daemon;
use crate::dump::ProcessDump;
use crate::format;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// Dump the configuration of all the processes
    #[structopt(long)]
    pub all: bool,
    /// The names of the processes to dump configurations for
    #[structopt(name = "process-name")]
    pub processes: Vec<String>,
    /// The path of the file to store the dump into
    #[structopt(short, long, default_value = "persist-dump.json")]
    pub out: PathBuf,
}

pub async fn handle(opts: Opts) -> Result<(), Error> {
    let filters = match (opts.all, opts.processes) {
        (false, processes) if processes.is_empty() => {
            return Err(Error::from(String::from(
                "you must specify at least one process name or --all",
            )));
        }
        (false, processes) => Some(processes),
        (true, _) => None,
    };

    let mut daemon = daemon::connect().await?;
    let responses = daemon.dump(DumpRequest { filters }).await?;

    let dumps = responses
        .into_iter()
        .map(|res| ProcessDump::from(res.spec))
        .collect::<Vec<_>>();

    let serialized = json::to_string_pretty(&dumps)?;
    tokio::fs::write(opts.out, serialized).await?;

    format::success("successfully dumped process configurations.");

    Ok(())
}

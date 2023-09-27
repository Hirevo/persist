use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tokio::fs;

use persist_core::error::Error;
use persist_core::protocol::{ProcessSpec, ProcessStatus, RestoreRequest};

use crate::daemon;
use crate::format;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// The path of the file to restore processes from
    #[structopt(short, long, default_value = "persist-dump.json")]
    pub from: PathBuf,
    /// Restore all processes
    #[structopt(long)]
    pub all: bool,
    /// Don't start any of the restored processes
    #[structopt(long)]
    pub stopped: bool,
    /// The names of the processes to restore
    #[structopt(name = "process-name")]
    pub processes: Vec<String>,
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

    let contents = fs::read(opts.from).await?;
    let specs: Vec<ProcessSpec> = json::from_slice(contents.as_slice())?;
    let mut specs = if let Some(filters) = filters {
        specs
            .into_iter()
            .filter(|spec| filters.contains(&spec.name))
            .collect()
    } else {
        specs
    };

    if opts.stopped {
        for spec in &mut specs {
            spec.status = ProcessStatus::Stopped;
        }
    }

    let request = RestoreRequest { specs };
    let mut daemon = daemon::connect().await?;
    let responses = daemon.restore(request).await?;

    for response in responses {
        if let Some(error) = response.error {
            let msg = format!(
                "process '{}' could not be restored: {}",
                response.name, error
            );
            format::error(msg);
        } else {
            let msg = format!("process '{}' successfully restored.", response.name);
            format::success(msg);
        }
    }

    Ok(())
}

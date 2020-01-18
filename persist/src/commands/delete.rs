use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use persist_core::error::Error;
use persist_core::protocol::DeleteRequest;

use crate::daemon;
use crate::format;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StructOpt)]
pub struct Opts {
    /// Delete all processes
    #[structopt(long)]
    pub all: bool,
    /// The names of the processes to stop managing
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

    let mut daemon = daemon::connect().await?;
    let responses = daemon.delete(DeleteRequest { filters }).await?;
    for response in responses {
        if let Some(error) = response.error {
            let msg = format!("process '{}' could not be deleted: {}", response.name, error);
            format::error(msg);
        } else {
            let msg = format!("process '{}' successfully deleted.", response.name);
            format::success(msg);
        }
    }

    Ok(())
}
